//! Подготовка и очистка TUN-интерфейса перед запуском sing-box.

pub const TUN_INTERFACE_NAME: &str = "wesk-tun";
pub const TUN_IPV4: &str = "172.19.0.1";

/// Убивает orphan sing-box и снимает stale IP с Wintun (Windows).
pub fn cleanup_stale_tun() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        return windows::run();
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use std::process::Command;
    use std::thread;
    use std::time::Duration;

    use super::{TUN_INTERFACE_NAME, TUN_IPV4};

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    pub fn run() -> Result<(), String> {
        kill_orphan_singbox();
        thread::sleep(Duration::from_millis(350));
        remove_stale_ipv4()?;
        thread::sleep(Duration::from_millis(250));
        Ok(())
    }

    fn kill_orphan_singbox() {
        use std::os::windows::process::CommandExt;
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "sing-box.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }

    fn remove_stale_ipv4() -> Result<(), String> {
        let ps = format!(
            r#"$ErrorActionPreference = 'SilentlyContinue'
$addr = '{ip}'
Get-NetIPAddress -IPAddress $addr | Remove-NetIPAddress -Confirm:$false
Get-NetAdapter | Where-Object {{
  $_.Name -eq '{iface}' -or $_.InterfaceDescription -like '*Wintun*' -or $_.InterfaceDescription -like '*sing-box*'
}} | ForEach-Object {{
  Disable-NetAdapter -Name $_.Name -Confirm:$false
  Start-Sleep -Milliseconds 200
  Enable-NetAdapter -Name $_.Name -Confirm:$false
}}
"#,
            ip = TUN_IPV4,
            iface = TUN_INTERFACE_NAME,
        );

        use std::os::windows::process::CommandExt;
        let out = Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &ps,
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("PowerShell недоступен: {e}"))?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !stderr.trim().is_empty() {
                return Err(format!("очистка TUN: {stderr}"));
            }
        }
        Ok(())
    }
}
