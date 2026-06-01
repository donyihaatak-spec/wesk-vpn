//! Журнал изменений сетевой конфигурации (rollback audit trail).

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use chrono::Utc;

pub struct NetworkChangeLog {
    path: PathBuf,
}

impl NetworkChangeLog {
    pub fn new(path: PathBuf) -> Self {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if !path.exists() {
            let header = format!(
                "# VPN Configurator — split tunnel network log\n# Created: {}\n",
                Utc::now().to_rfc3339()
            );
            let _ = std::fs::write(&path, header);
        }
        Self { path }
    }

    pub fn write(&self, action: &str, detail: &str) {
        let line = format!(
            "[{}] {action}: {detail}\n",
            Utc::now().to_rfc3339()
        );
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = file.write_all(line.as_bytes());
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Абсолютный путь с нативными разделителями (для открытия в Проводнике).
    pub fn display_path(&self) -> String {
        self.path.display().to_string()
    }
}
