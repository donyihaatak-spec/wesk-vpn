//! Поиск исполняемого файла sing-box на диске.

use std::path::{Path, PathBuf};
use std::process::Command;

use tauri::Manager;

#[cfg(target_os = "windows")]
const EXE_NAMES: &[&str] = &[
    "sing-box-x86_64-pc-windows-msvc.exe",
    "sing-box.exe",
];

#[cfg(not(target_os = "windows"))]
const EXE_NAMES: &[&str] = &["sing-box"];

/// Собирает список каталогов для поиска sing-box при старте приложения.
pub fn default_search_paths(app: &tauri::App, app_data: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Надёжно в dev-сборке: src-tauri/binaries (compile-time путь).
    paths.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries"));

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.to_path_buf());
            paths.push(dir.join("binaries"));
        }
    }

    if let Ok(res) = app.path().resource_dir() {
        paths.push(res.clone());
        paths.push(res.join("binaries"));
    }

    // Пользователь может положить бинарник в каталог данных приложения.
    paths.push(app_data.join("binaries"));

    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("src-tauri").join("binaries"));
        paths.push(cwd.join("binaries"));
    }

    #[cfg(target_os = "windows")]
    paths.push(PathBuf::from(r"C:\Program Files\sing-box"));

    dedupe_paths(paths)
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for p in paths {
        if !out.iter().any(|x| x == &p) {
            out.push(p);
        }
    }
    out
}

pub fn find_singbox_exe(search_paths: &[PathBuf]) -> Result<PathBuf, String> {
    for dir in search_paths {
        for name in EXE_NAMES {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    if let Ok(path) = which_in_path() {
        return Ok(path);
    }

    let checked = search_paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join("\n  · ");

    Err(format!(
        "не найден sing-box. Проверенные каталоги:\n  · {checked}\n\n\
         Скачайте: https://github.com/SagerNet/sing-box/releases\n\
         Распакуйте sing-box.exe в:\n  \
         {manifest}/binaries/sing-box-x86_64-pc-windows-msvc.exe\n\n\
         Или запустите: .\\scripts\\install-singbox.ps1",
        manifest = env!("CARGO_MANIFEST_DIR")
    ))
}

fn which_in_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    let cmd = "where";
    #[cfg(not(target_os = "windows"))]
    let cmd = "which";

    let output = Command::new(cmd)
        .arg("sing-box")
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("not in PATH".to_string());
    }
    let line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if line.is_empty() {
        Err("empty".to_string())
    } else {
        Ok(PathBuf::from(line))
    }
}
