//! Tauri-команды — единственная IPC-граница между React-фронтендом и
//! доменной логикой. Здесь нет бизнес-правил: команды берут блокировку
//! состояния и делегируют работу `ConfigStore`.

use tauri::{Manager, State};

use crate::error::{AppError, AppResult};
use crate::proxy::manager::ProxyStatus;
use crate::proxy::singbox::build_tun_config;
use crate::proxy::store::ProfileRecord;
use crate::split_tunnel::model::{ProcessInfo, SplitMode, SplitRule, SplitTunnelStatus, VpnBackend, VpnInterface};
use crate::vpn::manager::VpnStatus;
use crate::vpn::store::VpnConfigRecord;
use crate::AppState;

const SINGBOX_TUN_ADDR: &str = "172.19.0.1";
const WG_INTERFACE: &str = "wgvpncfg";

/// Сообщение об ошибке при «отравлении» Mutex (паника в другом потоке).
fn locked_err() -> AppError {
    AppError::Other("состояние хранилища заблокировано".to_string())
}

#[tauri::command]
pub fn list_configs(state: State<AppState>) -> AppResult<Vec<VpnConfigRecord>> {
    let store = state.store.lock().map_err(|_| locked_err())?;
    store.list()
}

#[tauri::command]
pub fn get_config(state: State<AppState>, id: String) -> AppResult<VpnConfigRecord> {
    let store = state.store.lock().map_err(|_| locked_err())?;
    store.get(&id)
}

/// Импорт `.conf` по абсолютному пути (путь приходит из диалога выбора файла).
#[tauri::command]
pub fn import_config_from_path(state: State<AppState>, path: String) -> AppResult<VpnConfigRecord> {
    let store = state.store.lock().map_err(|_| locked_err())?;
    store.import_from_path(&path)
}

/// Импорт `.conf` из переданного текста (например, вставка из буфера обмена).
#[tauri::command]
pub fn import_config_from_text(
    state: State<AppState>,
    name: String,
    content: String,
) -> AppResult<VpnConfigRecord> {
    let store = state.store.lock().map_err(|_| locked_err())?;
    store.import_from_text(&name, &content)
}

#[tauri::command]
pub fn rename_config(
    state: State<AppState>,
    id: String,
    name: String,
) -> AppResult<VpnConfigRecord> {
    let store = state.store.lock().map_err(|_| locked_err())?;
    store.rename(&id, &name)
}

#[tauri::command]
pub fn delete_config(state: State<AppState>, id: String) -> AppResult<()> {
    let store = state.store.lock().map_err(|_| locked_err())?;
    store.delete(&id)
}

/// Подключение VPN по идентификатору конфигурации.
#[tauri::command]
pub fn connect_vpn(state: State<AppState>, id: String) -> AppResult<VpnStatus> {
    let _ = state.proxy.disconnect();
    {
        let mut st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
        st.remove_rules()?;
    }

    let record = {
        let store = state.store.lock().map_err(|_| locked_err())?;
        store.get(&id)?
    };

    let status = state.vpn.connect(&record)?;

    if status.state == crate::vpn::manager::VpnState::Connected {
        let mut st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
        st.apply_rules(VpnBackend::WireGuard {
            interface_name: WG_INTERFACE.to_string(),
        })?;
    }

    Ok(status)
}

/// Отключение активного VPN-туннеля.
#[tauri::command]
pub fn disconnect_vpn(state: State<AppState>) -> AppResult<VpnStatus> {
    {
        let mut st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
        st.remove_rules()?;
    }
    state.vpn.disconnect()
}

/// Текущий статус VPN-подключения.
#[tauri::command]
pub fn get_vpn_status(state: State<AppState>) -> AppResult<VpnStatus> {
    Ok(state.vpn.status())
}

// ─── Proxy-профили (ключи Happ: vless/vmess/...) ───────────────────────────

fn profiles_locked_err() -> AppError {
    AppError::Other("состояние хранилища профилей заблокировано".to_string())
}

#[tauri::command]
pub fn list_profiles(state: State<AppState>) -> AppResult<Vec<ProfileRecord>> {
    let store = state.profiles.lock().map_err(|_| profiles_locked_err())?;
    store.list()
}

/// Импорт ключа из текста/буфера обмена (`vless://`, `vmess://`, ...).
#[tauri::command]
pub fn import_profile_from_text(state: State<AppState>, text: String) -> AppResult<ProfileRecord> {
    let store = state.profiles.lock().map_err(|_| profiles_locked_err())?;
    store.import_from_text(&text)
}

/// Импорт подписки по URL (как в Happ).
#[tauri::command]
pub async fn import_subscription(
    state: State<'_, AppState>,
    url: String,
) -> AppResult<Vec<ProfileRecord>> {
    let profiles = crate::proxy::subscription::fetch_subscription(&url).await?;
    let store = state.profiles.lock().map_err(|_| profiles_locked_err())?;
    store.import_many(profiles)
}

#[tauri::command]
pub fn rename_profile(
    state: State<AppState>,
    id: String,
    name: String,
) -> AppResult<ProfileRecord> {
    let store = state.profiles.lock().map_err(|_| profiles_locked_err())?;
    store.rename(&id, &name)
}

#[tauri::command]
pub fn delete_profile(state: State<AppState>, id: String) -> AppResult<()> {
    let store = state.profiles.lock().map_err(|_| profiles_locked_err())?;
    store.delete(&id)
}

#[tauri::command]
pub fn connect_profile(state: State<AppState>, id: String) -> AppResult<ProxyStatus> {
    let _ = state.vpn.disconnect();
    {
        let mut st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
        st.remove_rules()?;
    }

    let record = {
        let store = state.profiles.lock().map_err(|_| profiles_locked_err())?;
        store.get(&id)?
    };

    let split_rules = {
        let st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
        st.enabled_rules()
    };

    let parsed = crate::proxy::uri::parse_uri(&record.raw_uri)?;
    let config = build_tun_config(&parsed, &split_rules)?;

    let status = state.proxy.connect_with_config(&record, config)?;

    if status.state == crate::proxy::manager::ProxyState::Connected {
        let mut st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
        st.apply_rules(VpnBackend::SingBox {
            tun_address: SINGBOX_TUN_ADDR.to_string(),
        })?;
    }

    Ok(status)
}

#[tauri::command]
pub fn disconnect_profile(state: State<AppState>) -> AppResult<ProxyStatus> {
    {
        let mut st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
        st.remove_rules()?;
    }
    state.proxy.disconnect()
}

#[tauri::command]
pub fn get_proxy_status(state: State<AppState>) -> AppResult<ProxyStatus> {
    Ok(state.proxy.status())
}

/// Проверка: установлен ли sing-box. `null` = не найден, иначе путь к exe.
#[tauri::command]
pub fn check_singbox(state: State<AppState>) -> Option<String> {
    state
        .proxy
        .singbox_available()
        .ok()
        .map(|p| p.display().to_string())
}

/// Сброс «застрявшего» TUN-адаптера (Windows Wintun).
#[tauri::command]
pub fn reset_tun_adapter(state: State<AppState>) -> AppResult<()> {
    state.proxy.reset_tun()
}

// ─── Split tunneling ───────────────────────────────────────────────────────

fn split_locked_err() -> AppError {
    AppError::Other("состояние split tunneling заблокировано".to_string())
}

#[tauri::command]
pub fn list_split_rules(state: State<AppState>) -> AppResult<Vec<SplitRule>> {
    let st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
    Ok(st.list_rules())
}

#[tauri::command]
pub fn add_split_rule(
    state: State<AppState>,
    app_path: Option<String>,
    process_name: Option<String>,
    domain: Option<String>,
    domain_suffix: Option<String>,
    mode: SplitMode,
    interface: VpnInterface,
    priority: Option<i32>,
) -> AppResult<SplitRule> {
    let mut st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
    st.add_rule(
        app_path,
        process_name,
        domain,
        domain_suffix,
        mode,
        interface,
        priority.unwrap_or(0),
    )
}

#[tauri::command]
pub fn remove_split_rule(state: State<AppState>, id: String) -> AppResult<()> {
    let mut st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
    st.remove_rule(&id)
}

#[tauri::command]
pub fn set_split_rule_enabled(
    state: State<AppState>,
    id: String,
    enabled: bool,
) -> AppResult<SplitRule> {
    let mut st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
    st.set_rule_enabled(&id, enabled)
}

#[tauri::command]
pub fn detect_processes(state: State<AppState>) -> AppResult<Vec<ProcessInfo>> {
    let st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
    Ok(st.detect_processes())
}

#[tauri::command]
pub fn get_split_tunnel_status(state: State<AppState>) -> AppResult<SplitTunnelStatus> {
    let st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
    Ok(st.status())
}

/// Переприменить split tunneling: OS-правила + перезапуск sing-box если подключён.
#[tauri::command]
pub fn apply_split_rules(state: State<AppState>) -> AppResult<SplitTunnelStatus> {
    let split_rules = {
        let st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
        st.enabled_rules()
    };

    let proxy_status = state.proxy.status();
    let vpn_status = state.vpn.status();

    if proxy_status.state == crate::proxy::manager::ProxyState::Connected {
        if let Some(profile_id) = proxy_status.active_profile_id.clone() {
            let record = {
                let store = state.profiles.lock().map_err(|_| profiles_locked_err())?;
                store.get(&profile_id)?
            };
            let parsed = crate::proxy::uri::parse_uri(&record.raw_uri)?;
            let config = build_tun_config(&parsed, &split_rules)?;
            state.proxy.reconnect_with_config(&record, config)?;
        }

        let mut st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
        if st.status().active_backend.is_some() {
            st.refresh_rules()?;
        } else {
            st.apply_rules(VpnBackend::SingBox {
                tun_address: SINGBOX_TUN_ADDR.to_string(),
            })?;
        }
        return Ok(st.status());
    }

    if vpn_status.state == crate::vpn::manager::VpnState::Connected {
        let mut st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
        if st.status().active_backend.is_some() {
            st.refresh_rules()?;
        } else {
            st.apply_rules(VpnBackend::WireGuard {
                interface_name: WG_INTERFACE.to_string(),
            })?;
        }
        return Ok(st.status());
    }

    let st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
    Ok(st.status())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDiagnostics {
    pub version: String,
    pub data_dir: String,
    pub singbox_path: Option<String>,
    pub network_log_path: String,
}

/// Системная информация для экрана настроек.
#[tauri::command]
pub fn get_app_diagnostics(app: tauri::AppHandle, state: State<AppState>) -> AppResult<AppDiagnostics> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| AppError::NoDataDir)?;
    let st = state.split_tunnel.lock().map_err(|_| split_locked_err())?;
    Ok(AppDiagnostics {
        version: env!("CARGO_PKG_VERSION").to_string(),
        data_dir: data_dir.display().to_string(),
        singbox_path: state
            .proxy
            .singbox_available()
            .ok()
            .map(|p| p.display().to_string()),
        network_log_path: st.status().network_log_path,
    })
}
