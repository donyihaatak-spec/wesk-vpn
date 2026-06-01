//! Модель и парсер конфигурации WireGuard (`.conf`).
//!
//! Формат WireGuard — это INI-подобный файл с секциями `[Interface]`
//! (ровно одна) и `[Peer]` (одна или несколько). Парсер ниже толерантен
//! к комментариям (`#`), пустым строкам и неизвестным ключам, но проверяет
//! обязательные поля, чтобы заведомо битый файл не попал в хранилище.

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WireguardInterface {
    pub private_key: String,
    #[serde(default)]
    pub addresses: Vec<String>,
    #[serde(default)]
    pub dns: Vec<String>,
    #[serde(default)]
    pub listen_port: Option<u16>,
    #[serde(default)]
    pub mtu: Option<u32>,
    #[serde(default)]
    pub table: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WireguardPeer {
    pub public_key: String,
    #[serde(default)]
    pub preshared_key: Option<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub allowed_ips: Vec<String>,
    #[serde(default)]
    pub persistent_keepalive: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WireguardConfig {
    pub interface: WireguardInterface,
    pub peers: Vec<WireguardPeer>,
}

impl WireguardConfig {
    /// Разбирает текст `.conf`-файла. Возвращает ошибку `InvalidConfig`,
    /// если структура нарушена или отсутствуют обязательные поля.
    pub fn parse(input: &str) -> AppResult<Self> {
        #[derive(PartialEq)]
        enum Section {
            None,
            Interface,
            Peer,
        }

        let mut section = Section::None;
        let mut interface = WireguardInterface::default();
        let mut has_interface = false;

        let mut peers: Vec<WireguardPeer> = Vec::new();
        let mut current_peer = WireguardPeer::default();
        let mut in_peer = false;

        for raw_line in input.lines() {
            let line = strip_comment(raw_line);
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some(header) = parse_section_header(line) {
                match header.to_ascii_lowercase().as_str() {
                    "interface" => {
                        if in_peer {
                            peers.push(std::mem::take(&mut current_peer));
                            in_peer = false;
                        }
                        section = Section::Interface;
                        has_interface = true;
                    }
                    "peer" => {
                        if in_peer {
                            peers.push(std::mem::take(&mut current_peer));
                        }
                        current_peer = WireguardPeer::default();
                        section = Section::Peer;
                        in_peer = true;
                    }
                    other => {
                        return Err(AppError::InvalidConfig(format!(
                            "неизвестная секция [{other}]"
                        )));
                    }
                }
                continue;
            }

            let (key, value) = split_key_value(line).ok_or_else(|| {
                AppError::InvalidConfig(format!("строка не является парой ключ=значение: '{line}'"))
            })?;

            match section {
                Section::Interface => apply_interface_field(&mut interface, &key, &value)?,
                Section::Peer => apply_peer_field(&mut current_peer, &key, &value)?,
                Section::None => {
                    return Err(AppError::InvalidConfig(format!(
                        "параметр '{key}' расположен вне секции"
                    )));
                }
            }
        }

        if in_peer {
            peers.push(current_peer);
        }

        if !has_interface {
            return Err(AppError::InvalidConfig(
                "отсутствует секция [Interface]".to_string(),
            ));
        }
        if interface.private_key.trim().is_empty() {
            return Err(AppError::InvalidConfig(
                "в секции [Interface] не задан PrivateKey".to_string(),
            ));
        }
        if peers.is_empty() {
            return Err(AppError::InvalidConfig(
                "не найдено ни одной секции [Peer]".to_string(),
            ));
        }
        for (idx, peer) in peers.iter().enumerate() {
            if peer.public_key.trim().is_empty() {
                return Err(AppError::InvalidConfig(format!(
                    "в секции [Peer] #{} не задан PublicKey",
                    idx + 1
                )));
            }
        }

        Ok(Self { interface, peers })
    }

    /// Эндпоинт первого пира — используется для краткой подписи в списке.
    pub fn primary_endpoint(&self) -> Option<String> {
        self.peers.first().and_then(|p| p.endpoint.clone())
    }

    /// Объединённый список AllowedIPs всех пиров.
    pub fn all_allowed_ips(&self) -> Vec<String> {
        self.peers
            .iter()
            .flat_map(|p| p.allowed_ips.iter().cloned())
            .collect()
    }
}

fn strip_comment(line: &str) -> &str {
    match line.find('#') {
        Some(idx) => &line[..idx],
        None => line,
    }
}

fn parse_section_header(line: &str) -> Option<&str> {
    let line = line.trim();
    if line.starts_with('[') && line.ends_with(']') && line.len() >= 2 {
        Some(line[1..line.len() - 1].trim())
    } else {
        None
    }
}

fn split_key_value(line: &str) -> Option<(String, String)> {
    let idx = line.find('=')?;
    let key = line[..idx].trim().to_string();
    let value = line[idx + 1..].trim().to_string();
    if key.is_empty() {
        return None;
    }
    Some((key, value))
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn apply_interface_field(
    interface: &mut WireguardInterface,
    key: &str,
    value: &str,
) -> AppResult<()> {
    match key.to_ascii_lowercase().as_str() {
        "privatekey" => interface.private_key = value.to_string(),
        "address" => interface.addresses = split_csv(value),
        "dns" => interface.dns = split_csv(value),
        "listenport" => {
            interface.listen_port = Some(value.parse().map_err(|_| {
                AppError::InvalidConfig(format!("некорректный ListenPort: '{value}'"))
            })?);
        }
        "mtu" => {
            interface.mtu = Some(
                value
                    .parse()
                    .map_err(|_| AppError::InvalidConfig(format!("некорректный MTU: '{value}'")))?,
            );
        }
        "table" => interface.table = Some(value.to_string()),
        // Неизвестные ключи (PreUp/PostUp и пр.) игнорируем: исходный текст
        // конфигурации сохраняется отдельно и используется при подключении.
        _ => {}
    }
    Ok(())
}

fn apply_peer_field(peer: &mut WireguardPeer, key: &str, value: &str) -> AppResult<()> {
    match key.to_ascii_lowercase().as_str() {
        "publickey" => peer.public_key = value.to_string(),
        "presharedkey" => peer.preshared_key = Some(value.to_string()),
        "endpoint" => peer.endpoint = Some(value.to_string()),
        "allowedips" => peer.allowed_ips = split_csv(value),
        "persistentkeepalive" => {
            peer.persistent_keepalive = Some(value.parse().map_err(|_| {
                AppError::InvalidConfig(format!("некорректный PersistentKeepalive: '{value}'"))
            })?);
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
        [Interface]
        # основной интерфейс
        PrivateKey = aFakePrivateKeyBase64StringForTesting==
        Address = 10.0.0.2/32, fd00::2/128
        DNS = 1.1.1.1, 8.8.8.8
        ListenPort = 51820
        MTU = 1420

        [Peer]
        PublicKey = aFakePublicKeyBase64StringForTesting==
        PresharedKey = aFakePresharedKeyBase64String==
        Endpoint = vpn.example.com:51820
        AllowedIPs = 0.0.0.0/0, ::/0
        PersistentKeepalive = 25
    "#;

    #[test]
    fn parses_valid_config() {
        let cfg = WireguardConfig::parse(SAMPLE).expect("должен распарситься");
        assert_eq!(cfg.interface.addresses.len(), 2);
        assert_eq!(cfg.interface.dns, vec!["1.1.1.1", "8.8.8.8"]);
        assert_eq!(cfg.interface.listen_port, Some(51820));
        assert_eq!(cfg.interface.mtu, Some(1420));
        assert_eq!(cfg.peers.len(), 1);
        assert_eq!(
            cfg.primary_endpoint(),
            Some("vpn.example.com:51820".to_string())
        );
        assert_eq!(cfg.all_allowed_ips(), vec!["0.0.0.0/0", "::/0"]);
    }

    #[test]
    fn parses_multiple_peers() {
        let input = r#"
            [Interface]
            PrivateKey = key==
            Address = 10.0.0.2/32

            [Peer]
            PublicKey = peer1==
            AllowedIPs = 10.0.0.0/24

            [Peer]
            PublicKey = peer2==
            AllowedIPs = 192.168.0.0/24
        "#;
        let cfg = WireguardConfig::parse(input).unwrap();
        assert_eq!(cfg.peers.len(), 2);
        assert_eq!(cfg.peers[0].public_key, "peer1==");
        assert_eq!(cfg.peers[1].public_key, "peer2==");
    }

    #[test]
    fn rejects_missing_interface() {
        let input = "[Peer]\nPublicKey = x==\n";
        assert!(WireguardConfig::parse(input).is_err());
    }

    #[test]
    fn rejects_missing_private_key() {
        let input = "[Interface]\nAddress = 10.0.0.2/32\n[Peer]\nPublicKey = x==\n";
        assert!(WireguardConfig::parse(input).is_err());
    }

    #[test]
    fn rejects_missing_peer() {
        let input = "[Interface]\nPrivateKey = key==\n";
        assert!(WireguardConfig::parse(input).is_err());
    }
}
