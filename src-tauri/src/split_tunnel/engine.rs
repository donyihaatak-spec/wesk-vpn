//! Split tunneling engine: rule evaluation + OS apply/remove/refresh.

use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use crate::error::{AppError, AppResult};
use crate::split_tunnel::detector::{detect_processes, match_rules};
use crate::split_tunnel::log::NetworkChangeLog;
use crate::split_tunnel::model::{
    ProcessInfo, SplitMode, SplitRule, SplitTunnelStatus, VpnBackend, VpnInterface,
};
use crate::split_tunnel::platform::PlatformRouter;
use crate::split_tunnel::store::SplitTunnelStore;

pub struct SplitTunnelEngine {
    store: SplitTunnelStore,
    platform: PlatformRouter,
    active_backend: Option<VpnBackend>,
}

impl SplitTunnelEngine {
    pub fn new(data_dir: PathBuf) -> AppResult<Self> {
        std::fs::create_dir_all(&data_dir)?;
        let log = NetworkChangeLog::new(data_dir.join("network.log"));
        let store = SplitTunnelStore::new(data_dir)?;
        let platform = PlatformRouter::new(log);
        Ok(Self {
            store,
            platform,
            active_backend: None,
        })
    }

    pub fn list_rules(&self) -> Vec<SplitRule> {
        self.store.list()
    }

    pub fn add_rule(
        &mut self,
        app_path: Option<String>,
        process_name: Option<String>,
        domain: Option<String>,
        domain_suffix: Option<String>,
        mode: SplitMode,
        interface: VpnInterface,
        priority: i32,
    ) -> AppResult<SplitRule> {
        self.store.add(
            app_path,
            process_name,
            domain,
            domain_suffix,
            mode,
            interface,
            priority,
        )
    }

    pub fn remove_rule(&mut self, id: &str) -> AppResult<()> {
        self.store.remove(id)
    }

    pub fn set_rule_enabled(&mut self, id: &str, enabled: bool) -> AppResult<SplitRule> {
        self.store.set_enabled(id, enabled)
    }

    pub fn detect_processes(&self) -> Vec<ProcessInfo> {
        detect_processes()
    }

    pub fn status(&self) -> SplitTunnelStatus {
        let rules = self.store.list();
        SplitTunnelStatus {
            rules_count: rules.len(),
            applied: self.platform.is_applied(),
            active_backend: self.active_backend.as_ref().map(backend_label),
            matched_processes: match_rules(&rules),
            network_log_path: self.network_log_display_path(),
        }
    }

    /// Применяет правила при активном VPN. Вызывается после успешного connect.
    pub fn apply_rules(&mut self, backend: VpnBackend) -> AppResult<()> {
        thread::sleep(Duration::from_millis(500));

        let rules = self.store.enabled_rules();
        if rules.is_empty() {
            self.active_backend = Some(backend);
            return Ok(());
        }

        self.platform.apply_rules(&rules, &backend)?;
        self.active_backend = Some(backend);
        Ok(())
    }

    /// Снимает все OS-изменения. Вызывается при disconnect.
    pub fn remove_rules(&mut self) -> AppResult<()> {
        self.platform.remove_rules()?;
        self.active_backend = None;
        Ok(())
    }

    /// Перечитывает правила из store и переприменяет OS-слой.
    pub fn refresh_rules(&mut self) -> AppResult<()> {
        let backend = self
            .active_backend
            .clone()
            .ok_or_else(|| AppError::Other("VPN не подключён — нечего обновлять".to_string()))?;
        self.platform.remove_rules()?;
        let rules = self.store.enabled_rules();
        if !rules.is_empty() {
            self.platform.apply_rules(&rules, &backend)?;
        }
        Ok(())
    }

    pub fn enabled_rules(&self) -> Vec<SplitRule> {
        self.store.enabled_rules()
    }

    pub fn network_log_path(&self) -> PathBuf {
        self.platform.log_path().clone()
    }

    pub fn network_log_display_path(&self) -> String {
        self.platform.log.display_path()
    }
}

fn backend_label(backend: &VpnBackend) -> String {
    match backend {
        VpnBackend::SingBox { tun_address } => format!("sing-box ({tun_address})"),
        VpnBackend::WireGuard { interface_name } => format!("wireguard ({interface_name})"),
    }
}
