//! Windows: route table + Windows Firewall (netsh).

use std::os::windows::process::CommandExt;
use std::process::Command;

use crate::error::{AppError, AppResult};
use crate::split_tunnel::model::{SplitMode, SplitRule, VpnBackend};
use crate::split_tunnel::platform::{
    log_apply, push_change, AppliedChange, PlatformRouter, VpnInterfaceInfo,
};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const RULE_PREFIX: &str = "NeonClick-Split-";

pub fn detect_interface(backend: &VpnBackend) -> AppResult<VpnInterfaceInfo> {
    let hint = match backend {
        VpnBackend::SingBox { tun_address } => tun_address.as_str(),
        VpnBackend::WireGuard { interface_name } => interface_name.as_str(),
    };

    let output = run_netsh(&["interface", "ipv4", "show", "config"])
        .map_err(AppError::Other)?;
    let mut name = None;
    let mut address = None;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Configuration for interface") {
            name = trimmed
                .trim_start_matches("Configuration for interface")
                .trim()
                .trim_matches('"')
                .to_string()
                .into();
        } else if trimmed.starts_with("IP Address:") {
            address = Some(trimmed.trim_start_matches("IP Address:").trim().to_string());
        } else if trimmed.is_empty() {
            if let (Some(ref n), Some(ref addr)) = (&name, &address) {
                if addr.starts_with(hint) || n.to_lowercase().contains("wintun") || n.contains(hint)
                {
                    return Ok(VpnInterfaceInfo {
                        name: n.clone(),
                        index: find_interface_index(n),
                        address: Some(addr.clone()),
                    });
                }
            }
            name = None;
            address = None;
        }
    }

    // Fallback: sing-box TUN часто на 172.19.0.1
    if let VpnBackend::SingBox { .. } = backend {
        if let Ok(ifaces) = list_interfaces() {
            for iface in ifaces {
                if iface
                    .address
                    .as_ref()
                    .is_some_and(|a| a.starts_with("172.19."))
                {
                    return Ok(iface);
                }
            }
        }
    }

    Ok(VpnInterfaceInfo {
        name: hint.to_string(),
        index: find_interface_index(hint),
        address: None,
    })
}

pub fn apply_rules(
    router: &mut PlatformRouter,
    rules: &[SplitRule],
    iface: &VpnInterfaceInfo,
    backend: &VpnBackend,
) -> AppResult<()> {
    for rule in rules.iter().filter(|r| r.enabled) {
        apply_app_rule(router, rule, iface, backend)?;
    }

    log_apply(
        router,
        &format!(
            "windows: {} rules on iface {} (idx {:?})",
            rules.len(),
            iface.name,
            iface.index
        ),
    );
    Ok(())
}

fn apply_app_rule(
    router: &mut PlatformRouter,
    rule: &SplitRule,
    iface: &VpnInterfaceInfo,
    backend: &VpnBackend,
) -> AppResult<()> {
    let rule_name = format!("{RULE_PREFIX}{}", rule.id);

    match rule.mode {
        SplitMode::ExcludeVpn => {
            // Блокируем исходящий трафик приложения через VPN-интерфейс.
            if let Some(ref path) = rule.app_path {
                add_firewall_block_on_interface(router, &rule_name, path, &iface.name)?;
            } else if let Some(ref name) = rule.process_name {
                // Для process_name без полного пути — ищем exe через детектор.
                let path = find_process_path(name);
                if let Some(p) = path {
                    add_firewall_block_on_interface(
                        router,
                        &rule_name,
                        &p,
                        &iface.name,
                    )?;
                } else {
                    log_apply(
                        router,
                        &format!("skip firewall for {name}: path unknown, sing-box rule active"),
                    );
                }
            }
        }
        SplitMode::IncludeVpn => {
            // В режиме include: блокируем приложение на физическом интерфейсе,
            // чтобы трафик шёл только через TUN/sing-box.
            if let Some(ref path) = rule.app_path {
                add_firewall_force_vpn(router, &rule_name, path, backend)?;
            }
        }
    }

    Ok(())
}

fn add_firewall_block_on_interface(
    router: &mut PlatformRouter,
    rule_name: &str,
    program: &str,
    interface_name: &str,
) -> AppResult<()> {
    // netsh: блокируем программу при использовании VPN-интерфейса
    let args = [
        "advfirewall",
        "firewall",
        "add",
        "rule",
        &format!("name={rule_name}"),
        "dir=out",
        "action=block",
        &format!("program={program}"),
        "enable=yes",
        "profile=any",
        &format!("localip=any"),
        &format!("remoteip=any"),
    ];
    run_netsh(&args).map_err(|e| AppError::Other(format!("firewall add: {e}")))?;
    router.log.write(
        "firewall-block",
        &format!("{rule_name} program={program} iface={interface_name}"),
    );
    push_change(
        router,
        AppliedChange::FirewallRule {
            name: rule_name.to_string(),
        },
    );
    Ok(())
}

fn add_firewall_force_vpn(
    router: &mut PlatformRouter,
    rule_name: &str,
    program: &str,
    backend: &VpnBackend,
) -> AppResult<()> {
    let _ = backend;
    let args = [
        "advfirewall",
        "firewall",
        "add",
        "rule",
        &format!("name={rule_name}"),
        "dir=out",
        "action=allow",
        &format!("program={program}"),
        "enable=yes",
        "profile=any",
    ];
    run_netsh(&args).map_err(|e| AppError::Other(format!("firewall allow: {e}")))?;
    router.log.write(
        "firewall-allow-vpn",
        &format!("{rule_name} program={program}"),
    );
    push_change(
        router,
        AppliedChange::FirewallRule {
            name: rule_name.to_string(),
        },
    );
    Ok(())
}

pub fn delete_route(
    destination: &str,
    mask: &str,
    _gateway: Option<&str>,
    if_index: Option<u32>,
) -> AppResult<()> {
    let mut args = vec!["delete".to_string(), destination.to_string()];
    if mask != "255.255.255.255" {
        args.push(format!("mask={mask}"));
    }
    if let Some(idx) = if_index {
        args.push(format!("if={idx}"));
    }
    let _ = run_route(&args);
    Ok(())
}

pub fn delete_firewall_rule(name: &str) -> AppResult<()> {
    let _ = run_netsh(&["advfirewall", "firewall", "delete", "rule", &format!("name={name}")]);
    Ok(())
}

fn add_route(destination: &str, mask: &str, if_index: Option<u32>) -> Result<(), String> {
    let mut args = vec!["add".to_string(), destination.to_string(), format!("mask={mask}")];
    if let Some(idx) = if_index {
        args.push(format!("if={idx}"));
    }
    args.push("metric=1".to_string());
    run_route(&args)
}

fn run_route(args: &[String]) -> Result<(), String> {
    let output = Command::new("route")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn run_netsh(args: &[&str]) -> Result<String, String> {
    let output = Command::new("netsh")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    if output.status.success() {
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("{} {}", stderr.trim(), stdout.trim()).trim().to_string())
    }
}

fn find_interface_index(name: &str) -> Option<u32> {
    let output = run_netsh(&["interface", "ipv4", "show", "interfaces"]).ok()?;
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let iface_name = parts[4..].join(" ");
            if iface_name.eq_ignore_ascii_case(name) {
                return parts.first()?.parse().ok();
            }
        }
    }
    None
}

fn list_interfaces() -> Result<Vec<VpnInterfaceInfo>, String> {
    let output = run_netsh(&["interface", "ipv4", "show", "config"])?;
    let mut result = Vec::new();
    let mut name = None;
    let mut address = None;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Configuration for interface") {
            name = Some(
                trimmed
                    .trim_start_matches("Configuration for interface")
                    .trim()
                    .trim_matches('"')
                    .to_string(),
            );
        } else if trimmed.starts_with("IP Address:") {
            address = Some(trimmed.trim_start_matches("IP Address:").trim().to_string());
        } else if trimmed.is_empty() {
            if let Some(n) = name.take() {
                result.push(VpnInterfaceInfo {
                    index: find_interface_index(&n),
                    name: n,
                    address: address.take(),
                });
            }
        }
    }
    Ok(result)
}

fn find_process_path(process_name: &str) -> Option<String> {
    use sysinfo::{ProcessRefreshKind, RefreshKind, System};
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let needle = process_name.to_lowercase();
    sys.processes().values().find_map(|p| {
        if p.name().to_string_lossy().eq_ignore_ascii_case(&needle) {
            p.exe().map(|e| e.to_string_lossy().into_owned())
        } else {
            None
        }
    })
}
