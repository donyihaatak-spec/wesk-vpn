//! Модель правил split tunneling.

use serde::{Deserialize, Serialize};

/// Режим правила: приложение идёт через VPN или обходит его.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SplitMode {
    /// Только перечисленные приложения используют VPN (остальной трафик — direct).
    IncludeVpn,
    /// Перечисленные приложения обходят VPN.
    ExcludeVpn,
}

/// Целевой сетевой интерфейс VPN.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VpnInterface {
    /// sing-box TUN (wintun/tun0).
    Tun0,
    /// WireGuard-интерфейс.
    Wg0,
    /// Физический адаптер (обход VPN).
    System,
}

/// Активный VPN-бэкенд для применения OS-правил.
#[derive(Debug, Clone)]
pub enum VpnBackend {
    SingBox { tun_address: String },
    WireGuard { interface_name: String },
}

/// Правило split tunneling для одного приложения.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitRule {
    pub id: String,
    /// Полный путь к исполняемому файлу (предпочтительно для Windows firewall).
    pub app_path: Option<String>,
    /// Имя процесса, например `chrome.exe`.
    pub process_name: Option<String>,
    /// Точный домен, например `web.telegram.org`.
    #[serde(default)]
    pub domain: Option<String>,
    /// Суффикс домена, например `telegram.org`.
    #[serde(default)]
    pub domain_suffix: Option<String>,
    pub mode: SplitMode,
    pub interface: VpnInterface,
    /// Большее значение = выше приоритет при конфликте.
    pub priority: i32,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl SplitRule {
    pub fn validate(&self) -> Result<(), String> {
        if self.app_path.is_none()
            && self.process_name.is_none()
            && self.domain.is_none()
            && self.domain_suffix.is_none()
        {
            return Err("укажите appPath, processName, domain или domainSuffix".to_string());
        }
        Ok(())
    }

    pub fn match_key(&self) -> String {
        self.app_path
            .clone()
            .or_else(|| self.process_name.clone())
            .or_else(|| self.domain.clone())
            .or_else(|| self.domain_suffix.clone())
            .unwrap_or_default()
            .to_lowercase()
    }
}

/// Информация о запущенном процессе (для UI / refresh).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub exe_path: Option<String>,
}

/// Снимок состояния split tunneling.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitTunnelStatus {
    pub rules_count: usize,
    pub applied: bool,
    pub active_backend: Option<String>,
    pub matched_processes: Vec<ProcessInfo>,
    pub network_log_path: String,
}
