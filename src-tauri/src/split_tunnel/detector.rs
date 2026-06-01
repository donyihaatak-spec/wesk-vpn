//! Детектор запущенных процессов для сопоставления с правилами.

use sysinfo::{ProcessRefreshKind, RefreshKind, System};

use crate::split_tunnel::model::{ProcessInfo, SplitRule};

/// Список всех процессов с именем и путём к exe.
pub fn detect_processes() -> Vec<ProcessInfo> {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    sys.processes()
        .iter()
        .map(|(pid, proc_)| ProcessInfo {
            pid: pid.as_u32(),
            name: proc_.name().to_string_lossy().into_owned(),
            exe_path: proc_.exe().map(|p| p.to_string_lossy().into_owned()),
        })
        .collect()
}

/// Процессы, соответствующие хотя бы одному правилу.
pub fn match_rules(rules: &[SplitRule]) -> Vec<ProcessInfo> {
    let enabled: Vec<_> = rules.iter().filter(|r| r.enabled).collect();
    if enabled.is_empty() {
        return Vec::new();
    }

    detect_processes()
        .into_iter()
        .filter(|proc_| rule_matches_process(&enabled, proc_))
        .collect()
}

fn rule_matches_process(rules: &[&SplitRule], proc_: &ProcessInfo) -> bool {
    let proc_name = proc_.name.to_lowercase();
    let proc_path = proc_
        .exe_path
        .as_ref()
        .map(|p| p.to_lowercase());

    rules.iter().any(|rule| {
        if let Some(ref name) = rule.process_name {
            if proc_name == name.to_lowercase() {
                return true;
            }
        }
        if let Some(ref path) = rule.app_path {
            if proc_path.as_ref().is_some_and(|p| p == &path.to_lowercase()) {
                return true;
            }
        }
        false
    })
}
