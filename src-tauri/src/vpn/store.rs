//! Локальное хранилище WireGuard-конфигураций.
//!
//! Структура на диске:
//! ```text
//! <app_data_dir>/configs/
//!   ├── index.json      # список метаданных (id, имя, даты)
//!   ├── <id>.conf       # исходный текст конфигурации
//!   └── ...
//! ```
//!
//! Исходный `.conf` хранится «как есть», чтобы при подключении передать его
//! WireGuard без потери неподдерживаемых парсером директив (PostUp и т.п.).
//! `index.json` хранит только метаданные для быстрого построения списка.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::vpn::config::WireguardConfig;

/// Метаданные конфигурации, хранящиеся в `index.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredConfigMeta {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Полная запись конфигурации, отдаваемая во фронтенд.
/// Содержит метаданные, краткую сводку из распарсенного `.conf`
/// и исходный текст (`raw`) для повторного экспорта/подключения.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VpnConfigRecord {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub addresses: Vec<String>,
    pub dns: Vec<String>,
    pub endpoint: Option<String>,
    pub allowed_ips: Vec<String>,
    pub peer_count: usize,
    /// Конфигурация распарсилась без ошибок (false — файл повреждён).
    pub valid: bool,
    pub raw: String,
}

pub struct ConfigStore {
    dir: PathBuf,
}

impl ConfigStore {
    /// Создаёт хранилище в указанном каталоге, создавая его при необходимости.
    /// Путь к каталогу данных приложения резолвится вызывающей стороной
    /// (в `lib.rs`), поэтому модуль остаётся независимым от Tauri.
    pub fn new(dir: PathBuf) -> AppResult<Self> {
        std::fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    fn index_path(&self) -> PathBuf {
        self.dir.join("index.json")
    }

    fn conf_path(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.conf"))
    }

    fn read_index(&self) -> AppResult<Vec<StoredConfigMeta>> {
        let path = self.index_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let data = std::fs::read_to_string(&path)?;
        if data.trim().is_empty() {
            return Ok(Vec::new());
        }
        Ok(serde_json::from_str(&data)?)
    }

    fn write_index(&self, items: &[StoredConfigMeta]) -> AppResult<()> {
        let data = serde_json::to_string_pretty(items)?;
        std::fs::write(self.index_path(), data)?;
        Ok(())
    }

    /// Импорт конфигурации из текста. Перед сохранением выполняется
    /// валидация через парсер, чтобы битые файлы не попадали в хранилище.
    pub fn import_from_text(&self, name: &str, raw: &str) -> AppResult<VpnConfigRecord> {
        let parsed = WireguardConfig::parse(raw)?;

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let meta = StoredConfigMeta {
            id: id.clone(),
            name: sanitize_name(name),
            created_at: now,
            updated_at: now,
        };

        std::fs::write(self.conf_path(&id), raw)?;

        let mut index = self.read_index()?;
        index.push(meta.clone());
        self.write_index(&index)?;

        Ok(build_record(meta, raw.to_string(), Some(parsed)))
    }

    /// Импорт конфигурации из файла на диске. Имя берётся из имени файла.
    pub fn import_from_path(&self, path: &str) -> AppResult<VpnConfigRecord> {
        let raw = std::fs::read_to_string(path)?;
        let name = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "WireGuard".to_string());
        self.import_from_text(&name, &raw)
    }

    /// Полный список сохранённых конфигураций.
    pub fn list(&self) -> AppResult<Vec<VpnConfigRecord>> {
        let index = self.read_index()?;
        let mut records = Vec::with_capacity(index.len());
        for meta in index {
            let raw = std::fs::read_to_string(self.conf_path(&meta.id)).unwrap_or_default();
            let parsed = WireguardConfig::parse(&raw).ok();
            records.push(build_record(meta, raw, parsed));
        }
        Ok(records)
    }

    /// Получение одной конфигурации по идентификатору.
    pub fn get(&self, id: &str) -> AppResult<VpnConfigRecord> {
        let index = self.read_index()?;
        let meta = index
            .into_iter()
            .find(|m| m.id == id)
            .ok_or_else(|| AppError::NotFound(id.to_string()))?;
        let raw = std::fs::read_to_string(self.conf_path(id)).unwrap_or_default();
        let parsed = WireguardConfig::parse(&raw).ok();
        Ok(build_record(meta, raw, parsed))
    }

    /// Переименование конфигурации.
    pub fn rename(&self, id: &str, new_name: &str) -> AppResult<VpnConfigRecord> {
        let mut index = self.read_index()?;
        let meta = index
            .iter_mut()
            .find(|m| m.id == id)
            .ok_or_else(|| AppError::NotFound(id.to_string()))?;
        meta.name = sanitize_name(new_name);
        meta.updated_at = Utc::now();
        let updated = meta.clone();
        self.write_index(&index)?;

        let raw = std::fs::read_to_string(self.conf_path(id)).unwrap_or_default();
        let parsed = WireguardConfig::parse(&raw).ok();
        Ok(build_record(updated, raw, parsed))
    }

    /// Удаление конфигурации: убирает запись из индекса и удаляет `.conf`.
    pub fn delete(&self, id: &str) -> AppResult<()> {
        let mut index = self.read_index()?;
        let before = index.len();
        index.retain(|m| m.id != id);
        if index.len() == before {
            return Err(AppError::NotFound(id.to_string()));
        }
        self.write_index(&index)?;

        let conf = self.conf_path(id);
        if conf.exists() {
            std::fs::remove_file(conf)?;
        }
        Ok(())
    }
}

fn sanitize_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        "WireGuard".to_string()
    } else {
        trimmed.chars().take(120).collect()
    }
}

fn build_record(
    meta: StoredConfigMeta,
    raw: String,
    parsed: Option<WireguardConfig>,
) -> VpnConfigRecord {
    let valid = parsed.is_some();
    let (addresses, dns, endpoint, allowed_ips, peer_count) = match &parsed {
        Some(cfg) => (
            cfg.interface.addresses.clone(),
            cfg.interface.dns.clone(),
            cfg.primary_endpoint(),
            cfg.all_allowed_ips(),
            cfg.peers.len(),
        ),
        None => (Vec::new(), Vec::new(), None, Vec::new(), 0),
    };

    VpnConfigRecord {
        id: meta.id,
        name: meta.name,
        created_at: meta.created_at,
        updated_at: meta.updated_at,
        addresses,
        dns,
        endpoint,
        allowed_ips,
        peer_count,
        valid,
        raw,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "[Interface]\nPrivateKey = key==\nAddress = 10.0.0.2/32\nDNS = 1.1.1.1\n[Peer]\nPublicKey = peer==\nEndpoint = host:51820\nAllowedIPs = 0.0.0.0/0\n";

    fn temp_store() -> ConfigStore {
        let dir = std::env::temp_dir().join(format!("vpncfg-test-{}", Uuid::new_v4()));
        ConfigStore::new(dir).unwrap()
    }

    #[test]
    fn import_list_get_delete_roundtrip() {
        let store = temp_store();

        assert_eq!(store.list().unwrap().len(), 0);

        let rec = store.import_from_text("Тест", SAMPLE).unwrap();
        assert_eq!(rec.name, "Тест");
        assert!(rec.valid);
        assert_eq!(rec.endpoint, Some("host:51820".to_string()));
        assert_eq!(rec.peer_count, 1);

        let list = store.list().unwrap();
        assert_eq!(list.len(), 1);

        let fetched = store.get(&rec.id).unwrap();
        assert_eq!(fetched.id, rec.id);
        assert_eq!(fetched.raw, SAMPLE);

        store.delete(&rec.id).unwrap();
        assert_eq!(store.list().unwrap().len(), 0);
        assert!(store.get(&rec.id).is_err());
    }

    #[test]
    fn rename_updates_name() {
        let store = temp_store();
        let rec = store.import_from_text("old", SAMPLE).unwrap();
        let renamed = store.rename(&rec.id, "new").unwrap();
        assert_eq!(renamed.name, "new");
    }

    #[test]
    fn rejects_invalid_on_import() {
        let store = temp_store();
        assert!(store.import_from_text("bad", "not a config").is_err());
        assert_eq!(store.list().unwrap().len(), 0);
    }

    #[test]
    fn delete_missing_errors() {
        let store = temp_store();
        assert!(store.delete("nonexistent").is_err());
    }
}
