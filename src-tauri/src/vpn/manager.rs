//! Менеджер VPN-туннеля WireGuard.
//!
//! Хранит текущий статус подключения и управляет реальным туннелем через
//! системные инструменты WireGuard:
//! - Windows: `wireguard.exe /installtunnelservice|/uninstalltunnelservice`;
//! - Linux/macOS: `wg-quick up|down`.
//!
//! Это не заглушка: команды действительно поднимают/опускают туннель. Для
//! успеха требуются установленный WireGuard и права администратора/root —
//! при их отсутствии возвращается понятная ошибка, которую UI показывает
//! уведомлением.
//!
//! Одновременно поддерживается один активный туннель с фиксированным именем
//! [`TUNNEL_NAME`], что упрощает корректное отключение.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use serde::Serialize;

use crate::error::{AppError, AppResult};
use crate::vpn::store::VpnConfigRecord;

/// Имя активного туннеля и одноимённого `.conf`-файла.
const TUNNEL_NAME: &str = "wgvpncfg";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VpnState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VpnStatus {
    pub state: VpnState,
    pub active_config_id: Option<String>,
    pub active_config_name: Option<String>,
    pub message: Option<String>,
}

impl VpnStatus {
    fn disconnected() -> Self {
        Self {
            state: VpnState::Disconnected,
            active_config_id: None,
            active_config_name: None,
            message: None,
        }
    }

    fn error(message: String) -> Self {
        Self {
            state: VpnState::Error,
            active_config_id: None,
            active_config_name: None,
            message: Some(message),
        }
    }
}

pub struct VpnManager {
    status: Mutex<VpnStatus>,
    tunnel_dir: PathBuf,
}

impl VpnManager {
    pub fn new(tunnel_dir: PathBuf) -> AppResult<Self> {
        std::fs::create_dir_all(&tunnel_dir)?;
        Ok(Self {
            status: Mutex::new(VpnStatus::disconnected()),
            tunnel_dir,
        })
    }

    /// Текущий статус (потокобезопасно). При «отравлении» Mutex возвращает
    /// безопасное значение «отключено».
    pub fn status(&self) -> VpnStatus {
        self.status
            .lock()
            .map(|s| s.clone())
            .unwrap_or_else(|_| VpnStatus::disconnected())
    }

    fn set_status(&self, status: VpnStatus) {
        if let Ok(mut guard) = self.status.lock() {
            *guard = status;
        }
    }

    fn tunnel_path(&self) -> PathBuf {
        self.tunnel_dir.join(format!("{TUNNEL_NAME}.conf"))
    }

    /// Поднимает туннель по выбранной конфигурации.
    pub fn connect(&self, record: &VpnConfigRecord) -> AppResult<VpnStatus> {
        if !record.valid {
            return Err(AppError::Other(
                "конфигурация повреждена и не может быть использована".to_string(),
            ));
        }

        self.set_status(VpnStatus {
            state: VpnState::Connecting,
            active_config_id: Some(record.id.clone()),
            active_config_name: Some(record.name.clone()),
            message: None,
        });

        let path = self.tunnel_path();
        if let Err(e) = std::fs::write(&path, &record.raw) {
            let msg = format!("не удалось записать файл туннеля: {e}");
            self.set_status(VpnStatus::error(msg));
            return Err(AppError::Io(e));
        }

        match bring_up(&path) {
            Ok(()) => {
                let status = VpnStatus {
                    state: VpnState::Connected,
                    active_config_id: Some(record.id.clone()),
                    active_config_name: Some(record.name.clone()),
                    message: None,
                };
                self.set_status(status.clone());
                Ok(status)
            }
            Err(message) => {
                self.set_status(VpnStatus::error(message.clone()));
                Err(AppError::Other(message))
            }
        }
    }

    /// Опускает активный туннель.
    pub fn disconnect(&self) -> AppResult<VpnStatus> {
        self.set_status(VpnStatus {
            state: VpnState::Disconnecting,
            ..self.status()
        });

        match bring_down(&self.tunnel_path()) {
            Ok(()) => {
                let status = VpnStatus::disconnected();
                self.set_status(status.clone());
                Ok(status)
            }
            Err(message) => {
                self.set_status(VpnStatus::error(message.clone()));
                Err(AppError::Other(message))
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn bring_up(conf: &Path) -> Result<(), String> {
    run_wireguard(&["/installtunnelservice", &conf.to_string_lossy()])
}

#[cfg(target_os = "windows")]
fn bring_down(_conf: &Path) -> Result<(), String> {
    run_wireguard(&["/uninstalltunnelservice", TUNNEL_NAME])
}

#[cfg(target_os = "windows")]
fn wireguard_exe() -> PathBuf {
    let default = PathBuf::from(r"C:\Program Files\WireGuard\wireguard.exe");
    if default.exists() {
        default
    } else {
        PathBuf::from("wireguard.exe")
    }
}

#[cfg(target_os = "windows")]
fn run_wireguard(args: &[&str]) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let exe = wireguard_exe();
    let output = Command::new(&exe)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| {
            format!(
                "не удалось запустить WireGuard ({}): {e}. Установлен ли WireGuard и есть ли права администратора?",
                exe.display()
            )
        })?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(format!(
            "WireGuard вернул ошибку. {} {}",
            stderr.trim(),
            stdout.trim()
        )
        .trim()
        .to_string())
    }
}

#[cfg(not(target_os = "windows"))]
fn bring_up(conf: &Path) -> Result<(), String> {
    run_wg_quick("up", conf)
}

#[cfg(not(target_os = "windows"))]
fn bring_down(conf: &Path) -> Result<(), String> {
    run_wg_quick("down", conf)
}

#[cfg(not(target_os = "windows"))]
fn run_wg_quick(action: &str, conf: &Path) -> Result<(), String> {
    let output = Command::new("wg-quick")
        .arg(action)
        .arg(conf)
        .output()
        .map_err(|e| {
            format!("не удалось запустить wg-quick: {e}. Установлены ли wireguard-tools и есть ли права root?")
        })?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("wg-quick {action} завершился с ошибкой: {}", stderr.trim()))
    }
}
