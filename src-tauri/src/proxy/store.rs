//! Персистентное хранилище proxy-профилей (ключей Happ).

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::proxy::uri::{parse_key_input, ProxyProfile, ProxyProtocol};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredProfileMeta {
    pub id: String,
    pub name: String,
    pub protocol: ProxyProtocol,
    pub server: String,
    pub port: u16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRecord {
    pub id: String,
    pub name: String,
    pub protocol: ProxyProtocol,
    pub server: String,
    pub port: u16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub raw_uri: String,
}

pub struct ProfileStore {
    dir: PathBuf,
}

impl ProfileStore {
    pub fn new(dir: PathBuf) -> AppResult<Self> {
        std::fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    fn index_path(&self) -> PathBuf {
        self.dir.join("index.json")
    }

    fn uri_path(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.uri"))
    }

    fn read_index(&self) -> AppResult<Vec<StoredProfileMeta>> {
        let path = self.index_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let data = std::fs::read_to_string(path)?;
        if data.trim().is_empty() {
            return Ok(Vec::new());
        }
        Ok(serde_json::from_str(&data)?)
    }

    fn write_index(&self, items: &[StoredProfileMeta]) -> AppResult<()> {
        std::fs::write(self.index_path(), serde_json::to_string_pretty(items)?)?;
        Ok(())
    }

    pub fn import_profile(&self, parsed: ProxyProfile) -> AppResult<ProfileRecord> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let meta = StoredProfileMeta {
            id: id.clone(),
            name: parsed.name.clone(),
            protocol: parsed.protocol,
            server: parsed.server.clone(),
            port: parsed.port,
            created_at: now,
            updated_at: now,
        };
        std::fs::write(self.uri_path(&id), &parsed.raw_uri)?;
        let mut index = self.read_index()?;
        index.push(meta.clone());
        self.write_index(&index)?;
        Ok(to_record(meta, parsed.raw_uri))
    }

    pub fn import_from_text(&self, text: &str) -> AppResult<ProfileRecord> {
        self.import_profile(parse_key_input(text)?)
    }

    pub fn import_many(&self, profiles: Vec<ProxyProfile>) -> AppResult<Vec<ProfileRecord>> {
        profiles.into_iter().map(|p| self.import_profile(p)).collect()
    }

    pub fn list(&self) -> AppResult<Vec<ProfileRecord>> {
        let index = self.read_index()?;
        Ok(index
            .into_iter()
            .map(|m| {
                let raw = std::fs::read_to_string(self.uri_path(&m.id)).unwrap_or_default();
                to_record(m, raw)
            })
            .collect())
    }

    pub fn get(&self, id: &str) -> AppResult<ProfileRecord> {
        let index = self.read_index()?;
        let meta = index
            .into_iter()
            .find(|m| m.id == id)
            .ok_or_else(|| AppError::NotFound(id.to_string()))?;
        let raw = std::fs::read_to_string(self.uri_path(id))?;
        Ok(to_record(meta, raw))
    }

    pub fn rename(&self, id: &str, name: &str) -> AppResult<ProfileRecord> {
        let mut index = self.read_index()?;
        let meta = index
            .iter_mut()
            .find(|m| m.id == id)
            .ok_or_else(|| AppError::NotFound(id.to_string()))?;
        meta.name = name.trim().to_string();
        meta.updated_at = Utc::now();
        let updated = meta.clone();
        self.write_index(&index)?;
        let raw = std::fs::read_to_string(self.uri_path(id))?;
        Ok(to_record(updated, raw))
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        let mut index = self.read_index()?;
        let before = index.len();
        index.retain(|m| m.id != id);
        if index.len() == before {
            return Err(AppError::NotFound(id.to_string()));
        }
        self.write_index(&index)?;
        let p = self.uri_path(id);
        if p.exists() {
            std::fs::remove_file(p)?;
        }
        Ok(())
    }
}

fn to_record(meta: StoredProfileMeta, raw_uri: String) -> ProfileRecord {
    ProfileRecord {
        id: meta.id,
        name: meta.name,
        protocol: meta.protocol,
        server: meta.server,
        port: meta.port,
        created_at: meta.created_at,
        updated_at: meta.updated_at,
        raw_uri,
    }
}
