//! OS-specific routing layer для split tunneling.

mod linux;
mod macos;
mod windows;

use crate::error::AppResult;
use crate::split_tunnel::log::NetworkChangeLog;
use crate::split_tunnel::model::{SplitRule, VpnBackend};
use std::path::PathBuf;

/// Информация о VPN-интерфейсе, обнаруженном в ОС.
#[derive(Debug, Clone)]
pub struct VpnInterfaceInfo {
    pub name: String,
    pub index: Option<u32>,
    pub address: Option<String>,
}

/// Отслеживаемое изменение для rollback.
#[derive(Debug, Clone)]
pub enum AppliedChange {
    Route {
        destination: String,
        mask: String,
        gateway: Option<String>,
        interface_index: Option<u32>,
    },
    FirewallRule {
        name: String,
    },
    IpRule {
        priority: u32,
        table: u32,
        mark: Option<u32>,
    },
    IpRoute {
        table: u32,
        destination: String,
        device: String,
    },
}

pub struct PlatformRouter {
    applied: Vec<AppliedChange>,
    pub(crate) log: NetworkChangeLog,
}

impl PlatformRouter {
    pub fn new(log: NetworkChangeLog) -> Self {
        Self {
            applied: Vec::new(),
            log,
        }
    }

    pub fn is_applied(&self) -> bool {
        !self.applied.is_empty()
    }

    pub fn applied_count(&self) -> usize {
        self.applied.len()
    }

    pub fn log_path(&self) -> &PathBuf {
        self.log.path()
    }

    /// Применяет OS-правила для активного VPN-бэкенда.
    pub fn apply_rules(
        &mut self,
        rules: &[SplitRule],
        backend: &VpnBackend,
    ) -> AppResult<VpnInterfaceInfo> {
        self.remove_rules()?;

        let iface = detect_vpn_interface(backend)?;

        #[cfg(target_os = "windows")]
        windows::apply_rules(self, rules, &iface, backend)?;

        #[cfg(target_os = "linux")]
        linux::apply_rules(self, rules, &iface, backend)?;

        #[cfg(target_os = "macos")]
        macos::apply_rules(self, rules, &iface, backend)?;

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            let _ = (rules, backend);
            self.log.write("apply", "platform not supported — sing-box rules only");
        }

        Ok(iface)
    }

    /// Полностью откатывает все OS-изменения.
    pub fn remove_rules(&mut self) -> AppResult<()> {
        while let Some(change) = self.applied.pop() {
            let result = rollback_change(&change, &self.log);
            if let Err(e) = result {
                self.log
                    .write("rollback-error", &format!("{change:?}: {e}"));
            }
        }
        Ok(())
    }
}

fn rollback_change(change: &AppliedChange, log: &NetworkChangeLog) -> AppResult<()> {
    match change {
        AppliedChange::Route {
            destination,
            mask,
            gateway,
            interface_index,
        } => {
            #[cfg(target_os = "windows")]
            windows::delete_route(destination, mask, gateway.as_deref(), *interface_index)?;
            #[cfg(target_os = "linux")]
            linux::delete_route(destination, mask, gateway.as_deref())?;
            #[cfg(target_os = "macos")]
            macos::delete_route(destination, mask, gateway.as_deref())?;
            #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
            let _ = (destination, mask, gateway, interface_index);
            log.write("rollback-route", &format!("{destination}/{mask}"));
        }
        AppliedChange::FirewallRule { name } => {
            #[cfg(target_os = "windows")]
            windows::delete_firewall_rule(name)?;
            #[cfg(not(target_os = "windows"))]
            let _ = name;
            log.write("rollback-firewall", name);
        }
        AppliedChange::IpRule {
            priority,
            table,
            mark,
        } => {
            #[cfg(target_os = "linux")]
            linux::delete_ip_rule(*priority, *table, *mark)?;
            #[cfg(not(target_os = "linux"))]
            let _ = (priority, table, mark);
            log.write("rollback-ip-rule", &format!("prio={priority} table={table}"));
        }
        AppliedChange::IpRoute {
            table,
            destination,
            device,
        } => {
            #[cfg(target_os = "linux")]
            linux::delete_ip_route(*table, destination, device)?;
            #[cfg(not(target_os = "linux"))]
            let _ = (table, destination, device);
            log.write(
                "rollback-ip-route",
                &format!("table={table} {destination} dev {device}"),
            );
        }
    }
    Ok(())
}

pub fn detect_vpn_interface(backend: &VpnBackend) -> AppResult<VpnInterfaceInfo> {
    #[cfg(target_os = "windows")]
    return windows::detect_interface(backend);

    #[cfg(target_os = "linux")]
    return linux::detect_interface(backend);

    #[cfg(target_os = "macos")]
    return macos::detect_interface(backend);

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        let _ = backend;
        Ok(VpnInterfaceInfo {
            name: "tun0".to_string(),
            index: None,
            address: None,
        })
    }
}

pub(crate) fn push_change(router: &mut PlatformRouter, change: AppliedChange) {
    router.applied.push(change);
}

pub(crate) fn log_apply(router: &PlatformRouter, msg: &str) {
    router.log.write("apply", msg);
}
