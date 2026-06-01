//! Загрузка и разбор subscription-URL (как в Happ).
//!
//! Поддерживается стандартная подписка: HTTP GET → тело с URI построчно
//! (часто целиком в base64). Зашифрованные `happ://crypto...` — в roadmap.

use crate::error::{AppError, AppResult};
use crate::proxy::uri::{decode_base64, looks_like_proxy_uri, parse_uri, ProxyProfile};

/// Скачивает подписку и возвращает список распарсенных профилей.
pub async fn fetch_subscription(url: &str) -> AppResult<Vec<ProxyProfile>> {
    let url = url.trim();
    if url.starts_with("happ://") {
        return Err(AppError::Other(
            "зашифрованные подписки happ://crypto пока не поддерживаются".to_string(),
        ));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Other(e.to_string()))?;

    let body = client
        .get(url)
        .header("User-Agent", "VPN-Configurator/0.1")
        .send()
        .await
        .map_err(|e| AppError::Other(format!("не удалось загрузить подписку: {e}")))?
        .text()
        .await
        .map_err(|e| AppError::Other(format!("ошибка чтения подписки: {e}")))?;

    parse_subscription_body(&body)
}

/// Разбирает тело подписки (текст или base64).
pub fn parse_subscription_body(body: &str) -> AppResult<Vec<ProxyProfile>> {
    let trimmed = body.trim();
    let text = try_decode_base64_body(trimmed).unwrap_or_else(|| trimmed.to_string());

    let mut profiles = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || !looks_like_proxy_uri(line) {
            continue;
        }
        match parse_uri(line) {
            Ok(p) => profiles.push(p),
            Err(e) => eprintln!("пропуск строки подписки: {e}"),
        }
    }

    if profiles.is_empty() {
        return Err(AppError::InvalidConfig(
            "в подписке не найдено поддерживаемых ключей".to_string(),
        ));
    }

    Ok(profiles)
}

fn try_decode_base64_body(input: &str) -> Option<String> {
    if looks_like_proxy_uri(input) {
        return None;
    }
    decode_base64(input).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiline_subscription() {
        let body = "vless://11111111-1111-1111-1111-111111111111@a.com:443#S1\nvless://22222222-2222-2222-2222-222222222222@b.com:443#S2\n";
        let profiles = parse_subscription_body(body).unwrap();
        assert_eq!(profiles.len(), 2);
    }
}
