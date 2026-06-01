//! Генерация конфигурации sing-box из proxy-ключей.
//!
//! sing-box — ядро, аналогичное Xray (как в Happ). Приложение запускает
//! `sing-box run -c config.json` в режиме TUN для системного VPN.

use std::collections::HashMap;

use serde_json::{json, Value};
use url::Url;

use crate::error::{AppError, AppResult};
use crate::proxy::uri::{parse_query, ProxyProfile, ProxyProtocol};

const PROXY_TAG: &str = "proxy";

/// Полный конфиг sing-box для системного TUN-туннеля (Windows/Linux/macOS).
pub fn build_tun_config(profile: &ProxyProfile, split_rules: &[crate::split_tunnel::model::SplitRule]) -> AppResult<Value> {
    let outbound = build_outbound(profile)?;
    let bypass = crate::split_tunnel::singbox_rules::has_direct_bypass(split_rules);
    let (route_rules, final_outbound) =
        crate::split_tunnel::singbox_rules::build_route_section(split_rules);
    let dns_rules = crate::split_tunnel::singbox_rules::build_dns_rules(split_rules);
    let tun_inbound = crate::split_tunnel::singbox_rules::build_tun_inbound(split_rules);
    let rule_sets = crate::split_tunnel::singbox_rules::build_route_rule_sets(bypass);

    let mut route = json!({
        "rules": route_rules,
        "final": final_outbound,
        "auto_detect_interface": true,
        "default_domain_resolver": "dns-ru"
    });
    if !rule_sets.is_empty() {
        route["rule_set"] = json!(rule_sets);
    }

    Ok(json!({
        "log": { "level": "warn", "timestamp": true },
        "dns": {
            "servers": [
                {
                    "type": "udp",
                    "tag": "dns-remote",
                    "server": "8.8.8.8",
                    "detour": PROXY_TAG
                },
                {
                    "type": "udp",
                    "tag": "dns-ru",
                    "server": "77.88.8.8"
                },
                {
                    "type": "local",
                    "tag": "dns-local"
                }
            ],
            "rules": dns_rules,
            "final": "dns-remote",
            "strategy": "prefer_ipv4"
        },
        "inbounds": [tun_inbound],
        "outbounds": [
            outbound,
            { "type": "direct", "tag": "direct" }
        ],
        "route": route
    }))
}

fn build_outbound(profile: &ProxyProfile) -> AppResult<Value> {
    match profile.protocol {
        ProxyProtocol::Vless => build_vless(&profile.raw_uri),
        ProxyProtocol::Vmess => build_vmess(&profile.raw_uri),
        ProxyProtocol::Trojan => build_trojan(&profile.raw_uri),
        ProxyProtocol::Shadowsocks => build_shadowsocks(&profile.raw_uri),
        ProxyProtocol::Socks => build_socks(&profile.raw_uri),
        ProxyProtocol::Hysteria2 => build_hysteria2(&profile.raw_uri),
    }
}

fn build_vless(uri: &str) -> AppResult<Value> {
    let url = Url::parse(uri).map_err(|e| AppError::InvalidConfig(e.to_string()))?;
    let uuid = url.username().to_string();
    if uuid.is_empty() {
        return Err(AppError::InvalidConfig("vless: отсутствует UUID".to_string()));
    }
    let server = url.host_str().unwrap_or("").to_string();
    let port = url.port().unwrap_or(443);
    let q = parse_query(&url);

    let mut outbound = json!({
        "type": "vless",
        "tag": PROXY_TAG,
        "server": server,
        "server_port": port,
        "uuid": uuid
    });

    if let Some(flow) = q.get("flow") {
        outbound["flow"] = json!(flow);
    }

    let security = q.get("security").map(|s| s.as_str()).unwrap_or("none");
    if security == "tls" || security == "reality" {
        let mut tls = json!({ "enabled": true });
        if let Some(sni) = q.get("sni").or(q.get("host")) {
            tls["server_name"] = json!(sni);
        }
        if let Some(fp) = q.get("fp") {
            tls["utls"] = json!({ "enabled": true, "fingerprint": fp });
        }
        if security == "reality" {
            tls["reality"] = json!({
                "enabled": true,
                "public_key": q.get("pbk").cloned().unwrap_or_default(),
                "short_id": q.get("sid").cloned().unwrap_or_default()
            });
        }
        outbound["tls"] = tls;
    }

    if let Some(transport) = build_transport(&q) {
        outbound["transport"] = transport;
    }

    Ok(outbound)
}

fn build_vmess(uri: &str) -> AppResult<Value> {
    let payload = uri.trim_start_matches("vmess://").split('#').next().unwrap_or("");
    let decoded = crate::proxy::uri::decode_base64(payload)?;
    let v: Value = serde_json::from_str(&decoded).map_err(|e| AppError::InvalidConfig(e.to_string()))?;

    let server = v
        .get("add")
        .or(v.get("host"))
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let port = v.get("port").and_then(|x| x.as_u64()).unwrap_or(443) as u16;
    let uuid = v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();

    let mut outbound = json!({
        "type": "vmess",
        "tag": PROXY_TAG,
        "server": server,
        "server_port": port,
        "uuid": uuid,
        "security": v.get("scy").and_then(|x| x.as_str()).unwrap_or("auto")
    });

    if v.get("tls").and_then(|x| x.as_str()) == Some("tls") {
        outbound["tls"] = json!({
            "enabled": true,
            "server_name": v.get("sni").or(v.get("host")).and_then(|x| x.as_str()).unwrap_or(&server)
        });
    }

    let net = v.get("net").and_then(|x| x.as_str()).unwrap_or("tcp");
    if net == "ws" {
        outbound["transport"] = json!({
            "type": "ws",
            "path": v.get("path").and_then(|x| x.as_str()).unwrap_or("/"),
            "headers": { "Host": v.get("host").and_then(|x| x.as_str()).unwrap_or(&server) }
        });
    } else if net == "grpc" {
        outbound["transport"] = json!({
            "type": "grpc",
            "service_name": v.get("path").and_then(|x| x.as_str()).unwrap_or("")
        });
    }

    Ok(outbound)
}

fn build_trojan(uri: &str) -> AppResult<Value> {
    let url = Url::parse(uri).map_err(|e| AppError::InvalidConfig(e.to_string()))?;
    let password = url.username().to_string();
    let server = url.host_str().unwrap_or("").to_string();
    let port = url.port().unwrap_or(443);
    let q = parse_query(&url);

    let mut outbound = json!({
        "type": "trojan",
        "tag": PROXY_TAG,
        "server": server,
        "server_port": port,
        "password": password
    });

    let mut tls = json!({ "enabled": true });
    if let Some(sni) = q.get("sni").or(q.get("host")) {
        tls["server_name"] = json!(sni);
    }
    outbound["tls"] = tls;

    if let Some(transport) = build_transport(&q) {
        outbound["transport"] = transport;
    }

    Ok(outbound)
}

fn build_shadowsocks(uri: &str) -> AppResult<Value> {
    let rest = uri.trim_start_matches("ss://");
    let main = rest.split('#').next().unwrap_or(rest);

    let (method, password, server, port) = if let Some(at) = main.rfind('@') {
        let cred = &main[..at];
        let host = &main[at + 1..];
        let creds = crate::proxy::uri::decode_base64(cred).unwrap_or_else(|_| cred.to_string());
        let (m, p) = creds.split_once(':').unwrap_or(("aes-256-gcm", creds.as_str()));
        let (s, pt) = parse_host_port(host, 8388)?;
        (m.to_string(), p.to_string(), s, pt)
    } else {
        let decoded = crate::proxy::uri::decode_base64(main)?;
        let (creds, host) = decoded
            .rsplit_once('@')
            .ok_or_else(|| AppError::InvalidConfig("ss: bad format".to_string()))?;
        let (m, p) = creds.split_once(':').unwrap_or(("aes-256-gcm", creds));
        let (s, pt) = parse_host_port(host, 8388)?;
        (m.to_string(), p.to_string(), s, pt)
    };

    Ok(json!({
        "type": "shadowsocks",
        "tag": PROXY_TAG,
        "server": server,
        "server_port": port,
        "method": method,
        "password": password
    }))
}

fn build_socks(uri: &str) -> AppResult<Value> {
    let url = Url::parse(uri).map_err(|e| AppError::InvalidConfig(e.to_string()))?;
    let server = url.host_str().unwrap_or("").to_string();
    let port = url.port().unwrap_or(1080);
    let version = if url.username().is_empty() { "5" } else { "5" };

    let mut outbound = json!({
        "type": "socks",
        "tag": PROXY_TAG,
        "server": server,
        "server_port": port,
        "version": version
    });

    if !url.username().is_empty() {
        outbound["username"] = json!(url.username());
        outbound["password"] = json!(url.password().unwrap_or(""));
    }

    Ok(outbound)
}

fn build_hysteria2(uri: &str) -> AppResult<Value> {
    let normalized = uri.replace("hy2://", "hysteria2://");
    let url = Url::parse(&normalized).map_err(|e| AppError::InvalidConfig(e.to_string()))?;
    let password = url.username().to_string();
    let server = url.host_str().unwrap_or("").to_string();
    let port = url.port().unwrap_or(443);
    let q = parse_query(&url);

    let mut outbound = json!({
        "type": "hysteria2",
        "tag": PROXY_TAG,
        "server": server,
        "server_port": port,
        "password": password
    });

    if q.get("security").map(|s| s.as_str()) == Some("tls") || port == 443 {
        outbound["tls"] = json!({
            "enabled": true,
            "server_name": q.get("sni").cloned().unwrap_or(server.clone())
        });
    }

    Ok(outbound)
}

fn build_transport(q: &HashMap<String, String>) -> Option<Value> {
    let t = q.get("type").map(|s| s.as_str()).unwrap_or("tcp");
    match t {
        "ws" => Some(json!({
            "type": "ws",
            "path": q.get("path").cloned().unwrap_or_else(|| "/".to_string()),
            "headers": { "Host": q.get("host").cloned().unwrap_or_default() }
        })),
        "grpc" => Some(json!({
            "type": "grpc",
            "service_name": q.get("serviceName").or(q.get("path")).cloned().unwrap_or_default()
        })),
        "http" | "h2" => Some(json!({
            "type": "http",
            "host": [q.get("host").cloned().unwrap_or_default()],
            "path": q.get("path").cloned().unwrap_or_else(|| "/".to_string())
        })),
        _ => None,
    }
}

fn parse_host_port(host_part: &str, default: u16) -> AppResult<(String, u16)> {
    crate::proxy::uri::parse_host_port(host_part, default)
}
