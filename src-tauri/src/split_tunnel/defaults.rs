//! Предустановленные правила для быстрой проверки split tunneling.

use chrono::Utc;
use uuid::Uuid;

use crate::split_tunnel::model::{SplitMode, SplitRule, VpnInterface};

const MARKER: &str = ".defaults_applied_v4";
const MARKER_V3: &str = ".defaults_applied_v3";
const MARKER_V2: &str = ".defaults_applied_v2";
const MARKER_V1: &str = ".defaults_applied";

pub fn marker_path(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join(MARKER)
}

pub fn defaults_already_applied(dir: &std::path::Path) -> bool {
    marker_path(dir).exists()
}

pub fn should_seed_defaults(dir: &std::path::Path, rules_empty: bool) -> bool {
    if defaults_already_applied(dir) {
        return false;
    }
    rules_empty
        || dir.join(MARKER_V3).exists()
        || dir.join(MARKER_V2).exists()
        || dir.join(MARKER_V1).exists()
}

pub fn mark_defaults_applied(dir: &std::path::Path) -> std::io::Result<()> {
    std::fs::write(marker_path(dir), "v4")
}

/// Telegram Web/Desktop — VPN. Госуслуги — direct + DNS через Яндекс.
pub fn default_rules() -> Vec<SplitRule> {
    let now = Utc::now().to_rfc3339();
    vec![
        domain_suffix("telegram.org", SplitMode::IncludeVpn, 100, &now),
        domain_suffix("t.me", SplitMode::IncludeVpn, 100, &now),
        domain("web.telegram.org", SplitMode::IncludeVpn, 100, &now),
        domain_suffix("telesco.pe", SplitMode::IncludeVpn, 90, &now),
        process("Telegram.exe", SplitMode::IncludeVpn, 80, &now),
        domain_suffix("gosuslugi.ru", SplitMode::ExcludeVpn, 120, &now),
        domain_suffix("gu-st.ru", SplitMode::ExcludeVpn, 120, &now),
        domain("esia.gosuslugi.ru", SplitMode::ExcludeVpn, 120, &now),
    ]
}

fn process(process_name: &str, mode: SplitMode, priority: i32, now: &str) -> SplitRule {
    SplitRule {
        id: Uuid::new_v4().to_string(),
        app_path: None,
        process_name: Some(process_name.to_string()),
        domain: None,
        domain_suffix: None,
        mode,
        interface: VpnInterface::Tun0,
        priority,
        enabled: true,
        created_at: now.to_string(),
        updated_at: now.to_string(),
    }
}

fn domain(domain: &str, mode: SplitMode, priority: i32, now: &str) -> SplitRule {
    SplitRule {
        id: Uuid::new_v4().to_string(),
        app_path: None,
        process_name: None,
        domain: Some(domain.to_string()),
        domain_suffix: None,
        mode,
        interface: VpnInterface::Tun0,
        priority,
        enabled: true,
        created_at: now.to_string(),
        updated_at: now.to_string(),
    }
}

fn domain_suffix(suffix: &str, mode: SplitMode, priority: i32, now: &str) -> SplitRule {
    SplitRule {
        id: Uuid::new_v4().to_string(),
        app_path: None,
        process_name: None,
        domain: None,
        domain_suffix: Some(suffix.to_string()),
        mode,
        interface: VpnInterface::Tun0,
        priority,
        enabled: true,
        created_at: now.to_string(),
        updated_at: now.to_string(),
    }
}
