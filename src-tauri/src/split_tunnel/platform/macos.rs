//! macOS: route-based filtering + sing-box process rules.

use std::process::Command;

use crate::error::AppResult;
use crate::split_tunnel::model::{SplitRule, VpnBackend};
use crate::split_tunnel::platform::{
    log_apply, push_change, AppliedChange, PlatformRouter, VpnInterfaceInfo,
};

pub fn detect_interface(backend: &VpnBackend) -> AppResult<VpnInterfaceInfo> {
    let name = match backend {
        VpnBackend::SingBox { .. } => "utun".to_string(),
        VpnBackend::WireGuard { interface_name } => interface_name.clone(),
    };

    Ok(VpnInterfaceInfo {
        name,
        index: None,
        address: match backend {
            VpnBackend::SingBox { tun_address } => Some(tun_address.clone()),
            VpnBackend::WireGuard { .. } => None,
        },
    })
}

pub fn apply_rules(
    router: &mut PlatformRouter,
    rules: &[SplitRule],
    iface: &VpnInterfaceInfo,
    _backend: &VpnBackend,
) -> AppResult<()> {
    // Host routes for split: private nets via physical gateway when excluding apps.
    for rule in rules.iter().filter(|r| r.enabled) {
        if let Some(ref path) = rule.app_path {
            router.log.write("macos-app", path);
        }
    }

    // Добавляем host route для TUN address (anti-leak helper).
    if let Some(ref addr) = iface.address {
        let dest = addr.split('.').take(3).collect::<Vec<_>>().join(".");
        if !dest.is_empty() {
            let route_dest = format!("{dest}.0/24");
            if add_route(&route_dest, &iface.name).is_ok() {
                push_change(
                    router,
                    AppliedChange::Route {
                        destination: route_dest.clone(),
                        mask: "255.255.255.0".to_string(),
                        gateway: None,
                        interface_index: None,
                    },
                );
            }
        }
    }

    log_apply(
        router,
        &format!("macos: {} rules on {}", rules.len(), iface.name),
    );
    Ok(())
}

pub fn delete_route(destination: &str, _mask: &str, gateway: Option<&str>) -> AppResult<()> {
    let mut args = vec!["delete".to_string(), "-net".to_string(), destination.to_string()];
    if let Some(gw) = gateway {
        args.push(gw.to_string());
    }
    let _ = Command::new("route").args(&args).status();
    Ok(())
}

fn add_route(destination: &str, iface: &str) -> Result<(), String> {
    let output = Command::new("route")
        .args(["add", "-net", destination, "-interface", iface])
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
