//! Генерация route/dns-правил sing-box из split tunneling policy.

use std::collections::{BTreeMap, HashSet};

use serde_json::{json, Value};

use crate::split_tunnel::model::{SplitMode, SplitRule};

const PROXY_TAG: &str = "proxy";
const DIRECT_TAG: &str = "direct";
pub const DIRECT_DNS_TAG: &str = "dns-ru";

const GEOIP_RU_URL: &str =
    "https://raw.githubusercontent.com/SagerNet/sing-geoip/rule-set/geoip-ru.srs";

/// IP DNS-серверов — исключаем из TUN-маршрута (Windows strict_route иначе ломает DNS).
pub const DIRECT_DNS_IPS: &[&str] = &[
    "77.88.8.8/32",
    "77.88.8.1/32",
    "223.5.5.5/32",
    "223.6.6.6/32",
    "195.208.4.2/32",
    "195.208.4.3/32",
];

pub fn has_direct_bypass(rules: &[SplitRule]) -> bool {
    rules
        .iter()
        .any(|r| r.enabled && r.mode == SplitMode::ExcludeVpn)
}

/// TUN inbound с исключениями для split tunneling.
pub fn build_tun_inbound(rules: &[SplitRule]) -> Value {
    let bypass = has_direct_bypass(rules);
    let mut tun = json!({
        "type": "tun",
        "tag": "tun-in",
        "interface_name": crate::proxy::tun_platform::TUN_INTERFACE_NAME,
        "address": ["172.19.0.1/30", "fdfe:dcba:9876::1/126"],
        "auto_route": true,
        "strict_route": !bypass,
        "stack": "mixed"
    });
    if bypass {
        tun["route_exclude_address"] = json!(DIRECT_DNS_IPS);
    }
    tun
}

pub fn build_route_rule_sets(bypass: bool) -> Vec<Value> {
    if !bypass {
        return Vec::new();
    }
    vec![json!({
        "type": "remote",
        "tag": "geoip-ru",
        "format": "binary",
        "url": GEOIP_RU_URL,
        "download_detour": DIRECT_TAG
    })]
}

/// Строит route.rules и route.final для sing-box TUN.
pub fn build_route_section(rules: &[SplitRule]) -> (Vec<Value>, String) {
    let enabled: Vec<_> = rules.iter().filter(|r| r.enabled).collect();
    let has_exclude = enabled.iter().any(|r| r.mode == SplitMode::ExcludeVpn);

    let mut route_rules = vec![
        json!({ "action": "sniff" }),
        json!({ "protocol": "dns", "action": "hijack-dns" }),
    ];

    if has_exclude {
        route_rules.push(json!({
            "action": "route",
            "ip_cidr": DIRECT_DNS_IPS,
            "outbound": DIRECT_TAG
        }));
        route_rules.push(json!({
            "action": "route",
            "ip_is_private": true,
            "outbound": DIRECT_TAG
        }));
    }

    let has_include = enabled.iter().any(|r| r.mode == SplitMode::IncludeVpn);
    let final_outbound = if has_include && !has_exclude {
        DIRECT_TAG
    } else {
        PROXY_TAG
    };

    // Telegram / include — до geoip-ru, чтобы не ушло в direct по RU IP.
    push_consolidated_rule(&mut route_rules, &enabled, SplitMode::IncludeVpn, PROXY_TAG);
    push_consolidated_rule(&mut route_rules, &enabled, SplitMode::ExcludeVpn, DIRECT_TAG);

    if has_exclude {
        route_rules.push(json!({
            "action": "route",
            "rule_set": "geoip-ru",
            "outbound": DIRECT_TAG
        }));
    }

    let mut process_rules: Vec<_> = enabled
        .iter()
        .filter(|r| r.process_name.is_some() || r.app_path.is_some())
        .copied()
        .collect();
    process_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

    for rule in process_rules {
        let outbound = match rule.mode {
            SplitMode::IncludeVpn => PROXY_TAG,
            SplitMode::ExcludeVpn => DIRECT_TAG,
        };
        let mut entry = json!({ "action": "route", "outbound": outbound });
        if let Some(ref name) = rule.process_name {
            entry["process_name"] = json!([name]);
        }
        if let Some(ref path) = rule.app_path {
            entry["process_path"] = json!([path]);
        }
        route_rules.push(entry);
    }

    (route_rules, final_outbound.to_string())
}

/// DNS: исключённые домены → dns-ru (sing-box 1.13).
pub fn build_dns_rules(rules: &[SplitRule]) -> Vec<Value> {
    let mut domains = HashSet::new();
    let mut suffixes = HashSet::new();

    for rule in rules.iter().filter(|r| r.enabled && r.mode == SplitMode::ExcludeVpn) {
        if let Some(ref d) = rule.domain {
            domains.insert(d.clone());
        }
        if let Some(ref s) = rule.domain_suffix {
            suffixes.insert(normalize_suffix(s));
        }
    }

    if domains.is_empty() && suffixes.is_empty() {
        return Vec::new();
    }

    let mut entry = json!({
        "action": "route",
        "server": DIRECT_DNS_TAG,
        "strategy": "ipv4_only"
    });
    if !domains.is_empty() {
        let mut list: Vec<_> = domains.into_iter().collect();
        list.sort();
        entry["domain"] = json!(list);
    }
    if !suffixes.is_empty() {
        let mut list: Vec<_> = suffixes.into_iter().collect();
        list.sort();
        entry["domain_suffix"] = json!(list);
    }

    vec![entry]
}

fn normalize_suffix(s: &str) -> String {
    if s.starts_with('.') {
        s.to_string()
    } else {
        format!(".{s}")
    }
}

fn push_consolidated_rule(
    route_rules: &mut Vec<Value>,
    enabled: &[&SplitRule],
    mode: SplitMode,
    outbound: &str,
) {
    let matching: Vec<_> = enabled
        .iter()
        .filter(|r| r.mode == mode && (r.domain.is_some() || r.domain_suffix.is_some()))
        .copied()
        .collect();

    if matching.is_empty() {
        return;
    }

    let mut domains = HashSet::new();
    let mut suffixes = HashSet::new();
    for rule in matching {
        if let Some(ref d) = rule.domain {
            domains.insert(d.clone());
        }
        if let Some(ref s) = rule.domain_suffix {
            suffixes.insert(normalize_suffix(s));
        }
    }

    let mut entry = json!({ "action": "route", "outbound": outbound });
    if !domains.is_empty() {
        let mut list: Vec<_> = domains.into_iter().collect();
        list.sort();
        entry["domain"] = json!(list);
    }
    if !suffixes.is_empty() {
        let mut list: Vec<_> = suffixes.into_iter().collect();
        list.sort();
        entry["domain_suffix"] = json!(list);
    }
    route_rules.push(entry);
}

#[allow(dead_code)]
pub fn excluded_domains(rules: &[SplitRule]) -> BTreeMap<String, Vec<String>> {
    let mut domains = Vec::new();
    let mut suffixes = Vec::new();
    for rule in rules.iter().filter(|r| r.enabled && r.mode == SplitMode::ExcludeVpn) {
        if let Some(ref d) = rule.domain {
            domains.push(d.clone());
        }
        if let Some(ref s) = rule.domain_suffix {
            suffixes.push(s.clone());
        }
    }
    let mut map = BTreeMap::new();
    map.insert("domain".to_string(), domains);
    map.insert("domain_suffix".to_string(), suffixes);
    map
}
