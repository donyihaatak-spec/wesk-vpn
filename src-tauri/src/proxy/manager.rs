//! Запуск/остановка sing-box и отслеживание статуса подключения.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde::Serialize;

use crate::error::{AppError, AppResult};
use crate::proxy::singbox::build_tun_config;
use crate::proxy::singbox_paths::find_singbox_exe;
use crate::proxy::store::ProfileRecord;
use crate::proxy::tun_platform;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyStatus {
    pub state: ProxyState,
    pub active_profile_id: Option<String>,
    pub active_profile_name: Option<String>,
    pub message: Option<String>,
}

impl ProxyStatus {
    fn disconnected() -> Self {
        Self {
            state: ProxyState::Disconnected,
            active_profile_id: None,
            active_profile_name: None,
            message: None,
        }
    }
}

struct ManagerInner {
    status: ProxyStatus,
    child: Option<Child>,
    config_path: PathBuf,
    log_path: PathBuf,
}

pub struct ProxyManager {
    inner: Mutex<ManagerInner>,
    search_paths: Vec<PathBuf>,
}

impl ProxyManager {
    pub fn new(runtime_dir: PathBuf, search_paths: Vec<PathBuf>) -> AppResult<Self> {
        std::fs::create_dir_all(&runtime_dir)?;
        Ok(Self {
            inner: Mutex::new(ManagerInner {
                status: ProxyStatus::disconnected(),
                child: None,
                config_path: runtime_dir.join("sing-box.json"),
                log_path: runtime_dir.join("sing-box.log"),
            }),
            search_paths,
        })
    }

    /// Проверка наличия sing-box до подключения.
    pub fn singbox_available(&self) -> Result<PathBuf, String> {
        find_singbox_exe(&self.search_paths)
    }

    pub fn log_path(&self) -> AppResult<PathBuf> {
        Ok(self
            .inner
            .lock()
            .map_err(|_| lock_err())?
            .log_path
            .clone())
    }

    pub fn status(&self) -> ProxyStatus {
        let mut guard = match self.inner.lock() {
            Ok(g) => g,
            Err(_) => return ProxyStatus::disconnected(),
        };
        sync_child_state(&mut guard);
        guard.status.clone()
    }

    pub fn connect_with_config(
        &self,
        profile: &ProfileRecord,
        config: serde_json::Value,
    ) -> AppResult<ProxyStatus> {
        let mut guard = self.inner.lock().map_err(|_| lock_err())?;

        stop_child(&mut guard.child);

        guard.status = ProxyStatus {
            state: ProxyState::Connecting,
            active_profile_id: Some(profile.id.clone()),
            active_profile_name: Some(profile.name.clone()),
            message: None,
        };

        let json = serde_json::to_string_pretty(&config)?;
        std::fs::write(&guard.config_path, &json)?;

        let exe = find_singbox_exe(&self.search_paths).map_err(AppError::Other)?;

        match start_singbox_with_retry(&exe, &guard.config_path, &guard.log_path, profile) {
            Ok(child) => {
                guard.child = Some(child);
            }
            Err(msg) => {
                guard.status = ProxyStatus {
                    state: ProxyState::Error,
                    active_profile_id: Some(profile.id.clone()),
                    active_profile_name: Some(profile.name.clone()),
                    message: Some(msg.clone()),
                };
                return Err(AppError::Other(msg));
            }
        }

        thread::sleep(Duration::from_millis(900));
        if let Some(ref mut child) = guard.child {
            if let Ok(Some(code)) = child.try_wait() {
                let log_tail = read_log_tail(&guard.log_path, 8);
                guard.child = None;

                if is_tun_conflict(&log_tail) {
                    let _ = tun_platform::cleanup_stale_tun();
                    thread::sleep(Duration::from_millis(400));
                    match start_singbox_with_retry(&exe, &guard.config_path, &guard.log_path, profile)
                    {
                        Ok(child) => {
                            guard.child = Some(child);
                            thread::sleep(Duration::from_millis(900));
                            if let Some(ref mut child) = guard.child {
                                if let Ok(Some(code)) = child.try_wait() {
                                    let log_tail = read_log_tail(&guard.log_path, 8);
                                    guard.child = None;
                                    let msg = tun_error_message(code, &log_tail, &guard.log_path);
                                    guard.status = ProxyStatus {
                                        state: ProxyState::Error,
                                        active_profile_id: Some(profile.id.clone()),
                                        active_profile_name: Some(profile.name.clone()),
                                        message: Some(msg.clone()),
                                    };
                                    return Err(AppError::Other(msg));
                                }
                            }
                        }
                        Err(msg) => {
                            guard.status = ProxyStatus {
                                state: ProxyState::Error,
                                active_profile_id: Some(profile.id.clone()),
                                active_profile_name: Some(profile.name.clone()),
                                message: Some(msg.clone()),
                            };
                            return Err(AppError::Other(msg));
                        }
                    }
                } else {
                    let msg = tun_error_message(code, &log_tail, &guard.log_path);
                    guard.status = ProxyStatus {
                        state: ProxyState::Error,
                        active_profile_id: Some(profile.id.clone()),
                        active_profile_name: Some(profile.name.clone()),
                        message: Some(msg.clone()),
                    };
                    return Err(AppError::Other(msg));
                }
            }
        }

        guard.status = ProxyStatus {
            state: ProxyState::Connected,
            active_profile_id: Some(profile.id.clone()),
            active_profile_name: Some(profile.name.clone()),
            message: None,
        };

        Ok(guard.status.clone())
    }

    pub fn connect(&self, profile: &ProfileRecord) -> AppResult<ProxyStatus> {
        let parsed = crate::proxy::uri::parse_uri(&profile.raw_uri)?;
        let config = build_tun_config(&parsed, &[])?;
        self.connect_with_config(profile, config)
    }

    pub fn is_connected(&self) -> bool {
        matches!(
            self.status().state,
            ProxyState::Connected | ProxyState::Connecting
        )
    }

    pub fn active_profile_id(&self) -> Option<String> {
        self.status().active_profile_id
    }

    /// Перезапуск sing-box с обновлённым конфигом (для refresh split rules).
    pub fn reconnect_with_config(
        &self,
        profile: &ProfileRecord,
        config: serde_json::Value,
    ) -> AppResult<ProxyStatus> {
        self.connect_with_config(profile, config)
    }

    pub fn disconnect(&self) -> AppResult<ProxyStatus> {
        let mut guard = self.inner.lock().map_err(|_| lock_err())?;
        guard.status.state = ProxyState::Disconnecting;
        stop_child(&mut guard.child);
        guard.status = ProxyStatus::disconnected();
        Ok(guard.status.clone())
    }

    /// Принудительный сброс TUN (orphan sing-box + stale Wintun).
    pub fn reset_tun(&self) -> AppResult<()> {
        let mut guard = self.inner.lock().map_err(|_| lock_err())?;
        stop_child(&mut guard.child);
        guard.status = ProxyStatus::disconnected();
        tun_platform::cleanup_stale_tun().map_err(AppError::Other)
    }
}

fn sync_child_state(guard: &mut ManagerInner) {
    if let Some(ref mut child) = guard.child {
        if let Ok(Some(code)) = child.try_wait() {
            let log_tail = read_log_tail(&guard.log_path, 5);
            guard.status = ProxyStatus {
                state: ProxyState::Error,
                active_profile_id: guard.status.active_profile_id.clone(),
                active_profile_name: guard.status.active_profile_name.clone(),
                message: Some(format!(
                    "sing-box неожиданно остановился ({code}). {log_tail}"
                )),
            };
            guard.child = None;
        }
    }
}

fn is_tun_conflict(log_tail: &str) -> bool {
    let lower = log_tail.to_lowercase();
    lower.contains("object already exists")
        || lower.contains("configure tun interface")
        || lower.contains("set ipv4 address")
}

fn tun_error_message(code: std::process::ExitStatus, log_tail: &str, log_path: &Path) -> String {
    let code = code.code().unwrap_or(-1);
    let hint = if is_tun_conflict(log_tail) {
        "Старый TUN-адаптер не освободился. Настройки → Система → «Сбросить TUN», затем подключитесь снова."
    } else {
        "Запустите приложение от имени администратора."
    };
    format!(
        "sing-box завершился с кодом {code}. {log_tail}\n{hint} Лог: {}",
        log_path.display()
    )
}

fn start_singbox_with_retry(
    exe: &Path,
    config_path: &Path,
    log_path: &Path,
    _profile: &ProfileRecord,
) -> Result<Child, String> {
    tun_platform::cleanup_stale_tun()?;
    let log_file = std::fs::File::create(log_path).map_err(|e| e.to_string())?;
    spawn_singbox(exe, config_path, log_file)
}

fn read_log_tail(path: &Path, lines: usize) -> String {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let tail: Vec<&str> = text.lines().rev().take(lines).collect();
    if tail.is_empty() {
        String::new()
    } else {
        format!("Лог: {}", tail.into_iter().rev().collect::<Vec<_>>().join(" | "))
    }
}

fn lock_err() -> AppError {
    AppError::Other("состояние proxy-менеджера заблокировано".to_string())
}

fn stop_child(child: &mut Option<Child>) {
    if let Some(mut c) = child.take() {
        let _ = c.kill();
        let _ = c.wait();
        thread::sleep(Duration::from_millis(300));
        let _ = tun_platform::cleanup_stale_tun();
    }
}

#[cfg(target_os = "windows")]
fn spawn_singbox(
    exe: &Path,
    config: &Path,
    log_file: std::fs::File,
) -> Result<Child, String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    Command::new(exe)
        .args(["run", "-c", &config.to_string_lossy()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(log_file))
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| {
            format!(
                "не удалось запустить sing-box ({exe:?}): {e}. \
                 Запустите Cursor/терминал от имени администратора."
            )
        })
}

#[cfg(not(target_os = "windows"))]
fn spawn_singbox(
    exe: &Path,
    config: &Path,
    log_file: std::fs::File,
) -> Result<Child, String> {
    Command::new(exe)
        .args(["run", "-c", &config.to_string_lossy()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(log_file))
        .spawn()
        .map_err(|e| format!("не удалось запустить sing-box: {e}"))
}
