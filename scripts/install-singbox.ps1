# Install sing-box for VPN Configurator (Windows)
# Uses project-local temp dir to avoid TEMP short-path issues (C:\Users\71C9~1) with Cyrillic usernames.
$ErrorActionPreference = "Stop"

$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$destDir = Join-Path $projectRoot "src-tauri\binaries"
$destFile = Join-Path $destDir "sing-box-x86_64-pc-windows-msvc.exe"
$workDir = Join-Path $projectRoot ".singbox-install"
$zipPath = Join-Path $workDir "sing-box.zip"
$extractDir = Join-Path $workDir "extract"

Write-Host "Target directory: $destDir"
New-Item -ItemType Directory -Force -Path $destDir | Out-Null

if (Test-Path $workDir) {
    Remove-Item -Recurse -Force $workDir
}
New-Item -ItemType Directory -Force -Path $extractDir | Out-Null

Write-Host "Fetching latest sing-box release from GitHub..."
$release = Invoke-RestMethod -Uri "https://api.github.com/repos/SagerNet/sing-box/releases/latest"
$asset = $release.assets | Where-Object { $_.name -match "windows-amd64\.zip" } | Select-Object -First 1

if (-not $asset) {
    Write-Error "windows-amd64.zip not found in latest release"
}

Write-Host "Downloading $($asset.name)..."
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zipPath

Expand-Archive -Path $zipPath -DestinationPath $extractDir -Force

$exe = Get-ChildItem -Path $extractDir -Recurse -Filter "sing-box.exe" | Select-Object -First 1
if (-not $exe) {
    Write-Error "sing-box.exe not found inside archive"
}

Copy-Item $exe.FullName -Destination $destFile -Force

Remove-Item -Recurse -Force $workDir -ErrorAction SilentlyContinue

Write-Host "OK: $destFile"
Write-Host ""
Write-Host "Next steps:"
Write-Host "  1. Restart the app: npm run tauri:dev"
Write-Host "  2. Run terminal as Administrator (required for TUN/VPN on Windows)"
Write-Host "  3. Click Connect on your profile"
