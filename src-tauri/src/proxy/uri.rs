//! Парсинг VPN/Proxy-ключей в форматах Happ и совместимых клиентов.
//!
//! Поддерживаемые схемы: `vless://`, `vmess://`, `trojan://`, `ss://`,
//! `socks://`, `hysteria2://`, `hy2://`.

use std::collections::HashMap;

use base64::Engine;
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyProtocol {
    Vless,
    Vmess,
    Trojan,
    Shadowsocks,
    Socks,
    Hysteria2,
}

impl ProxyProtocol {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Vless => "VLESS",
            Self::Vmess => "VMess",
            Self::Trojan => "Trojan",
            Self::Shadowsocks => "Shadowsocks",
            Self::Socks => "SOCKS",
            Self::Hysteria2 => "Hysteria2",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyProfile {
    pub name: String,
    pub protocol: ProxyProtocol,
    pub server: String,
    pub port: u16,
    /// Исходный ключ/URI — сохраняется для повторного подключения.
    pub raw_uri: String,
}

/// Определяет, похожа ли строка на поддерживаемый ключ.
pub fn looks_like_proxy_uri(input: &str) -> bool {
    let s = input.trim();
    s.starts_with("vless://")
        || s.starts_with("vmess://")
        || s.starts_with("trojan://")
        || s.starts_with("ss://")
        || s.starts_with("socks://")
        || s.starts_with("hysteria2://")
        || s.starts_with("hy2://")
}

/// Разбирает одну строку-ключ. Поддерживает несколько ключей через перенос строки
/// (берётся первый валидный).
pub fn parse_key_input(input: &str) -> AppResult<ProxyProfile> {
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if looks_like_proxy_uri(line) {
            return parse_uri(line);
        }
    }
    Err(AppError::InvalidConfig(
        "не найден поддерживаемый ключ (vless://, vmess://, trojan://, ss://, socks://, hysteria2://)"
            .to_string(),
    ))
}

pub fn parse_uri(uri: &str) -> AppResult<ProxyProfile> {
    let uri = uri.trim();
    if uri.starts_with("vmess://") {
        return parse_vmess(uri);
    }
    if uri.starts_with("ss://") {
        return parse_shadowsocks(uri);
    }

    let url = Url::parse(uri).map_err(|e| AppError::InvalidConfig(format!("некорректный URI: {e}")))?;

    let scheme = url.scheme();
    let protocol = match scheme {
        "vless" => ProxyProtocol::Vless,
        "trojan" => ProxyProtocol::Trojan,
        "socks" => ProxyProtocol::Socks,
        "hysteria2" | "hy2" => ProxyProtocol::Hysteria2,
        other => {
            return Err(AppError::InvalidConfig(format!(
                "неподдерживаемый протокол: {other}"
            )));
        }
    };

    let server = url
        .host_str()
        .ok_or_else(|| AppError::InvalidConfig("в ключе не указан сервер".to_string()))?
        .to_string();

    let default_port = match protocol {
        ProxyProtocol::Trojan | ProxyProtocol::Vless | ProxyProtocol::Hysteria2 => 443,
        ProxyProtocol::Socks => 1080,
        _ => 443,
    };
    let port = url.port().unwrap_or(default_port);

    let name = url
        .fragment()
        .map(decode_component)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("{} {server}:{port}", protocol.label()));

    Ok(ProxyProfile {
        name,
        protocol,
        server,
        port,
        raw_uri: uri.to_string(),
    })
}

fn parse_vmess(uri: &str) -> AppResult<ProxyProfile> {
    let payload = uri
        .trim_start_matches("vmess://")
        .split('#')
        .next()
        .unwrap_or("")
        .trim();

    let decoded = decode_base64(payload)?;
    let json: VmessJson =
        serde_json::from_str(&decoded).map_err(|e| AppError::InvalidConfig(format!("vmess JSON: {e}")))?;

    let server = json
        .add
        .or(json.host)
        .or(json.address)
        .ok_or_else(|| AppError::InvalidConfig("vmess: не указан адрес сервера".to_string()))?;

    let port = json.port.unwrap_or(443);
    let name = json
        .ps
        .or(json.remark)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("VMess {server}:{port}"));

    Ok(ProxyProfile {
        name,
        protocol: ProxyProtocol::Vmess,
        server,
        port,
        raw_uri: uri.to_string(),
    })
}

fn parse_shadowsocks(uri: &str) -> AppResult<ProxyProfile> {
    let rest = uri.trim_start_matches("ss://");
    let (main, fragment) = split_fragment(rest);
    let name = fragment
        .map(decode_component)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Shadowsocks".to_string());

    // ss://BASE64@host:port  или  ss://BASE64(method:pass@host:port)
    if let Some(at_idx) = main.rfind('@') {
        let cred_part = &main[..at_idx];
        let host_part = &main[at_idx + 1..];
        let (server, port) = parse_host_port(host_part, 8388)?;
        let _credentials = decode_base64(cred_part).unwrap_or_else(|_| cred_part.to_string());
        return Ok(ProxyProfile {
            name,
            protocol: ProxyProtocol::Shadowsocks,
            server,
            port,
            raw_uri: uri.to_string(),
        });
    }

    let decoded = decode_base64(main)?;
    // method:password@host:port
    let (credentials, host_part) = decoded
        .rsplit_once('@')
        .ok_or_else(|| AppError::InvalidConfig("ss: некорректный формат".to_string()))?;
    let (server, port) = parse_host_port(host_part, 8388)?;
    let _ = credentials; // используется при генерации sing-box конфига из raw_uri

    Ok(ProxyProfile {
        name,
        protocol: ProxyProtocol::Shadowsocks,
        server,
        port,
        raw_uri: uri.to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct VmessJson {
    add: Option<String>,
    host: Option<String>,
    address: Option<String>,
    port: Option<u16>,
    ps: Option<String>,
    remark: Option<String>,
}

pub fn parse_host_port(host_part: &str, default_port: u16) -> AppResult<(String, u16)> {
    // [IPv6]:port или host:port
    if host_part.starts_with('[') {
        let end = host_part
            .find(']')
            .ok_or_else(|| AppError::InvalidConfig("некорректный IPv6 адрес".to_string()))?;
        let host = host_part[1..end].to_string();
        let port = host_part
            .get(end + 2..)
            .and_then(|p| p.parse().ok())
            .unwrap_or(default_port);
        return Ok((host, port));
    }
    if let Some((host, port)) = host_part.rsplit_once(':') {
        let port: u16 = port
            .parse()
            .map_err(|_| AppError::InvalidConfig(format!("некорректный порт: {port}")))?;
        return Ok((host.to_string(), port));
    }
    Ok((host_part.to_string(), default_port))
}

fn split_fragment(s: &str) -> (&str, Option<&str>) {
    match s.split_once('#') {
        Some((a, b)) => (a, Some(b)),
        None => (s, None),
    }
}

fn decode_component(s: &str) -> String {
    percent_decode_str(s).decode_utf8_lossy().into_owned()
}

pub fn decode_base64(input: &str) -> AppResult<String> {
    let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let padded = match cleaned.len() % 4 {
        2 => format!("{cleaned}=="),
        3 => format!("{cleaned}="),
        _ => cleaned,
    };
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&padded)
        .or_else(|_| {
            base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(padded.trim_end_matches('='))
        })
        .map_err(|e| AppError::InvalidConfig(format!("base64: {e}")))?;
    String::from_utf8(bytes).map_err(|e| AppError::InvalidConfig(format!("utf8: {e}")))
}

/// Парсит query-параметры URI (vless/trojan/hy2).
pub fn parse_query(url: &Url) -> HashMap<String, String> {
    url.query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_vless_with_fragment() {
        let uri = "vless://11111111-1111-1111-1111-111111111111@example.com:443?security=reality&sni=example.com#MyServer";
        let p = parse_uri(uri).unwrap();
        assert_eq!(p.protocol, ProxyProtocol::Vless);
        assert_eq!(p.server, "example.com");
        assert_eq!(p.port, 443);
        assert_eq!(p.name, "MyServer");
    }

    #[test]
    fn detects_proxy_uri() {
        assert!(looks_like_proxy_uri("vless://a@b:443"));
        assert!(!looks_like_proxy_uri("[Interface]"));
    }
}
