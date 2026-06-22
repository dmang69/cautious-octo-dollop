# IntentKernel — Windows 11 dev/runtime setup
# Usage: powershell -ExecutionPolicy Bypass -File setup-windows.ps1
# Repo:  C:\Users\Dizzle\Documents\GitHub\cautious-octo-dollop

param(
    [string]$RepoRoot = "C:\Users\Dizzle\Documents\GitHub\cautious-octo-dollop"
)

$ErrorActionPreference = "Stop"

$ScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$GitHubRepo = Join-Path $env:USERPROFILE "Documents\GitHub\cautious-octo-dollop"
$ClionRepo = Join-Path $env:USERPROFILE "CLionProjects\cautious-octo-dollop"
$ProfileRepo = Join-Path $env:USERPROFILE "cautious-octo-dollop"
$Root = $null
foreach ($candidate in @($RepoRoot, $GitHubRepo, $ClionRepo, $ProfileRepo, $ScriptRoot, "C:\Users\Dizzle\CLionProjects\cautious-octo-dollop", "C:\Users\Dizzle\cautious-octo-dollop", "D:\cautious-octo-dollop")) {
    if ($candidate -and (Test-Path $candidate)) {
        $Root = (Resolve-Path $candidate).Path
        break
    }
}
if (-not $Root) {
    $Root = $RepoRoot
}
Set-Location $Root

[Environment]::SetEnvironmentVariable("INTENTKERNEL_DEV_ROOT", $Root, "User")
$env:INTENTKERNEL_DEV_ROOT = $Root

Write-Host "==> IntentKernel Windows setup" -ForegroundColor Cyan
Write-Host "    Repo: $Root"

function Ensure-Command($name, $installHint) {
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
        Write-Warning "Missing: $name — $installHint"
        return $false
    }
    return $true
}

# ── Prerequisites ───────────────────────────────────────────────────────────
$ok = $true
$ok = (Ensure-Command "rustc" "Install Rust: https://rustup.rs") -and $ok
$ok = (Ensure-Command "cargo" "Install Rust toolchain") -and $ok
$ok = (Ensure-Command "node" "winget install OpenJS.NodeJS.LTS") -and $ok
$ok = (Ensure-Command "npm" "comes with Node.js") -and $ok

if (-not $ok) {
    Write-Host "Install missing tools above, then re-run this script." -ForegroundColor Yellow
    exit 1
}

# WebView2 (Tauri)
if (-not (Get-AppxPackage -Name "Microsoft.WebView2*")) {
    Write-Host "==> Installing WebView2 runtime..." -ForegroundColor Yellow
    winget install --id Microsoft.EdgeWebView2Runtime -e --accept-package-agreements --accept-source-agreements
}

# ── Build core workspace ────────────────────────────────────────────────────
Write-Host "==> Building Rust workspace (ikd-verify, ai-runtime, intent-verifier)..." -ForegroundColor Cyan
if (Test-Path ".\share\brand\intent-kernel-logo.png") {
    Push-Location scripts
    npm install --silent 2>$null
    node generate-brand-assets.mjs
    Pop-Location
}

cargo build --release -p ikd-verify -p ai-runtime -p intent-verifier -p intentkernel-update -p intentkernel-cli
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# WASM parser
if (Test-Path ".\kernel\build_parser_wasm.sh") {
    Write-Host "==> Building WASM parser (via WSL bash if available)..." -ForegroundColor Cyan
    if (Get-Command "bash" -ErrorAction SilentlyContinue) {
        bash ./kernel/build_parser_wasm.sh
    }
}

# Tauri frontend deps
if (Test-Path ".\shell\tauri-app\package.json") {
    Write-Host "==> npm install (Tauri shell)..." -ForegroundColor Cyan
    Push-Location shell/tauri-app
    npm install
    Pop-Location
}

# ── PATH shim ───────────────────────────────────────────────────────────────
$BinDir = Join-Path $Root "target\release"
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$BinDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$BinDir", "User")
    $env:Path = "$env:Path;$BinDir"
    Write-Host "==> Added $BinDir to user PATH (new shells)" -ForegroundColor Green
}

Write-Host ""
Write-Host "Setup complete. Verify with:" -ForegroundColor Green
Write-Host "  ikd-verify --kernel-check --os win11"
Write-Host "  intentkernel status"
Write-Host "  cargo run --release -p ai-runtime"
Write-Host "  cd shell\tauri-app; npm run tauri dev"