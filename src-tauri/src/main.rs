// На Windows в release-сборке прячем лишнее консольное окно.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    vpn_configurator_lib::run();
}
