//! Персистентное хранилище правил split tunneling.

use std::path::{Path, PathBuf};

use chrono::Utc;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::split_tunnel::defaults;
use crate::split_tunnel::model::{SplitMode, SplitRule, VpnInterface};

const STORE_FILE: &str = "split_rules.json";

pub struct SplitTunnelStore {
    path: PathBuf,
    rules: Vec<SplitRule>,
}

impl SplitTunnelStore {
    pub fn new(dir: PathBuf) -> AppResult<Self> {
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(STORE_FILE);
        let mut rules: Vec<SplitRule> = if path.exists() {
            let text = std::fs::read_to_string(&path)?;
            serde_json::from_str(&text).unwrap_or_default()
        } else {
            Vec::new()
        };

        if defaults::should_seed_defaults(&dir, rules.is_empty()) {
            rules = defaults::default_rules();
            let json = serde_json::to_string_pretty(&rules)?;
            std::fs::write(&path, json)?;
            let _ = defaults::mark_defaults_applied(&dir);
        }

        Ok(Self { path, rules })
    }

    pub fn list(&self) -> Vec<SplitRule> {
        self.rules.clone()
    }

    pub fn enabled_rules(&self) -> Vec<SplitRule> {
        self.rules.iter().filter(|r| r.enabled).cloned().collect()
    }

    pub fn add(
        &mut self,
        app_path: Option<String>,
        process_name: Option<String>,
        domain: Option<String>,
        domain_suffix: Option<String>,
        mode: SplitMode,
        interface: VpnInterface,
        priority: i32,
    ) -> AppResult<SplitRule> {
        let now = Utc::now().to_rfc3339();
        let rule = SplitRule {
            id: Uuid::new_v4().to_string(),
            app_path,
            process_name,
            domain,
            domain_suffix,
            mode,
            interface,
            priority,
            enabled: true,
            created_at: now.clone(),
            updated_at: now,
        };
        rule.validate().map_err(AppError::Other)?;
        self.rules.push(rule.clone());
        self.save()?;
        Ok(rule)
    }

    pub fn remove(&mut self, id: &str) -> AppResult<()> {
        let before = self.rules.len();
        self.rules.retain(|r| r.id != id);
        if self.rules.len() == before {
            return Err(AppError::NotFound(id.to_string()));
        }
        self.save()
    }

    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> AppResult<SplitRule> {
        let rule = self
            .rules
            .iter_mut()
            .find(|r| r.id == id)
            .ok_or_else(|| AppError::NotFound(id.to_string()))?;
        rule.enabled = enabled;
        rule.updated_at = Utc::now().to_rfc3339();
        let out = rule.clone();
        self.save()?;
        Ok(out)
    }

    pub fn get(&self, id: &str) -> AppResult<SplitRule> {
        self.rules
            .iter()
            .find(|r| r.id == id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(id.to_string()))
    }

    fn save(&self) -> AppResult<()> {
        let json = serde_json::to_string_pretty(&self.rules)?;
        std::fs::write(&self.path, json)?;
        Ok(())
    }

    pub fn store_path(&self) -> &Path {
        &self.path
    }
}
