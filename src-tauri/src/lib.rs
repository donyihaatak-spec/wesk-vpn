//! Точка сборки Tauri-приложения: инициализация состояния, плагинов и
//! регистрация команд. Бинарник (`main.rs`) лишь вызывает `run()`.

mod commands;
mod error;
mod proxy;
mod split_tunnel;
mod vpn;

use std::sync::Mutex;

use tauri::Manager;

use crate::error::AppError;
use crate::proxy::manager::ProxyManager;
use crate::proxy::singbox_paths::default_search_paths;
use crate::proxy::store::ProfileStore;
use crate::split_tunnel::SplitTunnelEngine;
use crate::vpn::manager::VpnManager;
use crate::vpn::store::ConfigStore;

/// Глобальное состояние приложения, доступное всем командам через `State`.
pub struct AppState {
    pub store: Mutex<ConfigStore>,
    pub vpn: VpnManager,
    pub profiles: Mutex<ProfileStore>,
    pub proxy: ProxyManager,
    pub split_tunnel: Mutex<SplitTunnelEngine>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let base = app
                .path()
                .app_data_dir()
                .map_err(|_| AppError::NoDataDir)?;
            let store = ConfigStore::new(base.join("configs"))?;
            let vpn = VpnManager::new(base.join("tunnels"))?;
            let profiles = ProfileStore::new(base.join("profiles"))?;
            let singbox_paths = default_search_paths(app, &base);
            let proxy = ProxyManager::new(base.join("proxy-runtime"), singbox_paths)?;
            let split_tunnel = SplitTunnelEngine::new(base.join("split_tunnel"))?;
            app.manage(AppState {
                store: Mutex::new(store),
                vpn,
                profiles: Mutex::new(profiles),
                proxy,
                split_tunnel: Mutex::new(split_tunnel),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_configs,
            commands::get_config,
            commands::import_config_from_path,
            commands::import_config_from_text,
            commands::rename_config,
            commands::delete_config,
            commands::connect_vpn,
            commands::disconnect_vpn,
            commands::get_vpn_status,
            commands::list_profiles,
            commands::import_profile_from_text,
            commands::import_subscription,
            commands::rename_profile,
            commands::delete_profile,
            commands::connect_profile,
            commands::disconnect_profile,
            commands::get_proxy_status,
            commands::check_singbox,
            commands::reset_tun_adapter,
            commands::add_split_rule,
            commands::remove_split_rule,
            commands::set_split_rule_enabled,
            commands::list_split_rules,
            commands::apply_split_rules,
            commands::detect_processes,
            commands::get_split_tunnel_status,
            commands::get_app_diagnostics,
        ])
        .run(tauri::generate_context!())
        .expect("ошибка при запуске Tauri-приложения");
}
