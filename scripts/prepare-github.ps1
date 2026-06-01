# Собирает чистую папку wesk-vpn-github/ только с VPN-клиентом Wesk.
# Запуск: npm run prepare:github
# Или:   powershell -ExecutionPolicy Bypass -File scripts/prepare-github.ps1

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$out = Join-Path $root "wesk-vpn-github"

Write-Host "Wesk - preparing GitHub folder..." -ForegroundColor Cyan
Write-Host "  Source: $root"
Write-Host "  Output: $out"

if (Test-Path $out) {
    Remove-Item $out -Recurse -Force
}
New-Item -ItemType Directory -Path $out | Out-Null

$rootFiles = @(
    "package.json",
    "package-lock.json",
    "vite.config.ts",
    "tsconfig.json",
    "tsconfig.node.json",
    "index.html",
    "postcss.config.mjs",
    "tailwind.config.ts",
    ".gitignore",
    "README.md",
    "GITHUB.md",
    "LICENSE"
)

foreach ($file in $rootFiles) {
    $src = Join-Path $root $file
    if (Test-Path $src) {
        Copy-Item $src $out
        Write-Host "  + $file"
    }
}

function Copy-Tree {
    param([string]$RelativePath)
    $src = Join-Path $root $RelativePath
    $dst = Join-Path $out $RelativePath
    if (-not (Test-Path $src)) { return }
    Copy-Item $src $dst -Recurse -Force
    Write-Host "  + $RelativePath/"
}

Copy-Tree "scripts"
Copy-Tree "src-tauri/capabilities"
Copy-Tree "src-tauri/icons"
Copy-Tree "src/styles"
Copy-Tree "src/hooks"

New-Item -ItemType Directory -Path (Join-Path $out "src/lib") -Force | Out-Null
$vpnLib = @("tauri.ts", "brand.ts", "smoothTransition.ts")
foreach ($f in $vpnLib) {
    Copy-Item (Join-Path $root "src/lib/$f") (Join-Path $out "src/lib/$f")
}
Write-Host "  + src/lib/ ($($vpnLib.Count) files)"

$srcTauriOut = Join-Path $out "src-tauri"
New-Item -ItemType Directory -Path (Join-Path $srcTauriOut "binaries") -Force | Out-Null
Copy-Item (Join-Path $root "src-tauri/binaries/README.md") (Join-Path $srcTauriOut "binaries\") -ErrorAction SilentlyContinue
Copy-Item (Join-Path $root "src-tauri/binaries/.gitkeep") (Join-Path $srcTauriOut "binaries\") -ErrorAction SilentlyContinue

foreach ($file in @("Cargo.toml", "build.rs", "tauri.conf.json")) {
    Copy-Item (Join-Path $root "src-tauri/$file") $srcTauriOut
    Write-Host "  + src-tauri/$file"
}

Copy-Tree "src-tauri/src"

New-Item -ItemType Directory -Path (Join-Path $out "src/components") -Force | Out-Null
$vpnComponents = @(
    "AppBackdrop.tsx", "AppFooterShowcase.tsx", "BottomBar.tsx", "ConnectAmbient.tsx", "ConnectProgress.tsx",
    "ConnectionHero.tsx", "ConfigCard.tsx", "ConfigList.tsx",
    "DesktopSidebar.tsx", "EmptyState.tsx", "Header.tsx", "ImportConfigModal.tsx", "ImportKeyModal.tsx",
    "Logo.tsx", "ProfileCard.tsx", "ProfileList.tsx", "SettingsModal.tsx",
    "Spinner.tsx", "StatusBadge.tsx", "Toast.tsx", "Toggle.tsx"
)
foreach ($c in $vpnComponents) {
    Copy-Item (Join-Path $root "src/components/$c") (Join-Path $out "src/components/$c")
}
Write-Host "  + src/components/ ($($vpnComponents.Count) files)"

foreach ($f in @("App.tsx", "main.tsx", "vite-env.d.ts")) {
    Copy-Item (Join-Path $root "src/$f") (Join-Path $out "src/$f")
    Write-Host "  + src/$f"
}

$size = (Get-ChildItem $out -Recurse -File | Measure-Object -Property Length -Sum).Sum / 1MB
Write-Host ""
Write-Host "Done! Folder: $out" -ForegroundColor Green
Write-Host ("Size: {0:N1} MB" -f $size)
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Open github.com -> New repository"
Write-Host "  2. Drag contents of wesk-vpn-github into the repo"
Write-Host "  3. Or install Git and follow GITHUB.md"
