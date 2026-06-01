//! Linux: ip rule / ip route + iptables owner (где доступно).

use std::process::Command;

use crate::error::{AppError, AppResult};
use crate::split_tunnel::model::{SplitMode, SplitRule, VpnBackend};
use crate::split_tunnel::platform::{
    log_apply, push_change, AppliedChange, PlatformRouter, VpnInterfaceInfo,
};

const SPLIT_TABLE: u32 = 100;
const SPLIT_MARK: u32 = 0x4e4f; // "NO"

pub fn detect_interface(backend: &VpnBackend) -> AppResult<VpnInterfaceInfo> {
    let name = match backend {
        VpnBackend::SingBox { .. } => "tun0".to_string(),
        VpnBackend::WireGuard { interface_name } => interface_name.clone(),
    };

    let output = Command::new("ip")
        .args(["-4", "addr", "show", "dev", &name])
        .output()
        .map_err(|e| AppError::Other(format!("ip addr: {e}")))?;

    let text = String::from_utf8_lossy(&output.stdout);
    let address = text
        .lines()
        .find(|l| l.contains("inet "))
        .and_then(|l| l.split_whitespace().nth(1))
        .map(|s| s.split('/').next().unwrap_or(s).to_string());

    Ok(VpnInterfaceInfo {
        name,
        index: None,
        address,
    })
}

pub fn apply_rules(
    router: &mut PlatformRouter,
    rules: &[SplitRule],
    iface: &VpnInterfaceInfo,
    backend: &VpnBackend,
) -> AppResult<()> {
    let device = &iface.name;

    for rule in rules.iter().filter(|r| r.enabled) {
        match rule.mode {
            SplitMode::IncludeVpn => {
                if let Some(ref path) = rule.app_path {
                    add_iptables_owner_mark(router, path, SPLIT_MARK)?;
                } else if let Some(ref name) = rule.process_name {
                    add_iptables_cmd_owner(router, name, SPLIT_MARK)?;
                }
            }
            SplitMode::ExcludeVpn => {
                if let Some(ref path) = rule.app_path {
                    add_iptables_owner_bypass(router, path)?;
                }
            }
        }
    }

    // Policy routing: marked packets → VPN table
    if rules.iter().any(|r| r.enabled && r.mode == SplitMode::IncludeVpn) {
        run_ip(&["rule", "add", "fwmark", &format!("0x{SPLIT_MARK:x}"), "table", &SPLIT_TABLE.to_string()])?;
        push_change(
            router,
            AppliedChange::IpRule {
                priority: 0,
                table: SPLIT_TABLE,
                mark: Some(SPLIT_MARK),
            },
        );

        let gateway = match backend {
            VpnBackend::SingBox { .. } | VpnBackend::WireGuard { .. } => {
                run_ip(&[
                    "route",
                    "add",
                    "default",
                    "dev",
                    device,
                    "table",
                    &SPLIT_TABLE.to_string(),
                ])?;
                push_change(
                    router,
                    AppliedChange::IpRoute {
                        table: SPLIT_TABLE,
                        destination: "default".to_string(),
                        device: device.clone(),
                    },
                );
            }
        };
        let _ = gateway;
    }

    log_apply(
        router,
        &format!("linux: {} rules on dev {device}", rules.len()),
    );
    Ok(())
}

fn add_iptables_owner_mark(router: &mut PlatformRouter, path: &str, mark: u32) -> AppResult<()> {
    let status = Command::new("iptables")
        .args([
            "-t", "mangle", "-A", "OUTPUT",
            "-m", "owner", "--cmd-owner", path,
            "-j", "MARK", "--set-mark", &mark.to_string(),
        ])
        .status()
        .map_err(|e| AppError::Other(format!("iptables: {e}")))?;

    if status.success() {
        router.log.write("iptables-mark", path);
    }
    Ok(())
}

fn add_iptables_cmd_owner(router: &mut PlatformRouter, name: &str, mark: u32) -> AppResult<()> {
    let status = Command::new("iptables")
        .args([
            "-t", "mangle", "-A", "OUTPUT",
            "-m", "owner", "--cmd-owner", name,
            "-j", "MARK", "--set-mark", &mark.to_string(),
        ])
        .status()
        .map_err(|e| AppError::Other(format!("iptables: {e}")))?;

    if status.success() {
        router.log.write("iptables-mark-cmd", name);
    }
    Ok(())
}

fn add_iptables_owner_bypass(router: &mut PlatformRouter, path: &str) -> AppResult<()> {
    let status = Command::new("iptables")
        .args([
            "-t", "mangle", "-A", "OUTPUT",
            "-m", "owner", "--cmd-owner", path,
            "-j", "ACCEPT",
        ])
        .status()
        .map_err(|e| AppError::Other(format!("iptables bypass: {e}")))?;

    if status.success() {
        router.log.write("iptables-bypass", path);
    }
    Ok(())
}

pub fn delete_route(destination: &str, mask: &str, gateway: Option<&str>) -> AppResult<()> {
    let mut args = vec!["route", "del", destination];
    if mask != "255.255.255.255" {
        args.push("via");
        if let Some(gw) = gateway {
            args.push(gw);
        }
    }
    let _ = Command::new("ip").args(&args).status();
    Ok(())
}

pub fn delete_ip_rule(priority: u32, table: u32, mark: Option<u32>) -> AppResult<()> {
    if let Some(m) = mark {
        let _ = run_ip(&[
            "rule",
            "del",
            "fwmark",
            &format!("0x{m:x}"),
            "table",
            &table.to_string(),
        ]);
    } else {
        let _ = run_ip(&["rule", "del", "priority", &priority.to_string()]);
    }
    Ok(())
}

pub fn delete_ip_route(table: u32, destination: &str, device: &str) -> AppResult<()> {
    let _ = run_ip(&[
        "route",
        "del",
        destination,
        "dev",
        device,
        "table",
        &table.to_string(),
    ]);
    Ok(())
}

fn run_ip(args: &[&str]) -> AppResult<()> {
    let output = Command::new("ip")
        .args(args)
        .output()
        .map_err(|e| AppError::Other(format!("ip {}: {e}", args.join(" "))))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(AppError::Other(String::from_utf8_lossy(&output.stderr).trim().to_string()))
    }
}
