# Build Windows release zip for install.ps1 -Local
# Usage: powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
# Repo:  C:\Users\Dizzle\Documents\GitHub\cautious-octo-dollop

param(
    [string]$RepoRoot = "C:\Users\Dizzle\Documents\GitHub\cautious-octo-dollop",
    [string]$IsoRoot = "C:\Users\Dizzle\IntentKernelISO",
    [switch]$StageIso
)

$ErrorActionPreference = "Stop"

$Version = "1.0.0"
$ScriptRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
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

Write-Host "==> Packaging Intent Kernel AI OS $Version for Windows" -ForegroundColor Cyan

if (Test-Path ".\share\brand\intent-kernel-logo.png") {
    Write-Host "==> Generating brand assets..." -ForegroundColor Yellow
    Push-Location scripts
    npm install --silent 2>$null
    node generate-brand-assets.mjs
    Pop-Location
}

Write-Host "==> Building Rust binaries..." -ForegroundColor Yellow
cargo build --release -p ai-runtime -p intent-verifier -p intentkernel-update -p intentkernel-cli -p ikd-verify
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$Stage = Join-Path $Root "dist\IntentKernel"
$Bin = Join-Path $Stage "bin"
$Share = Join-Path $Stage "share"

if (Test-Path $Stage) {
    Remove-Item $Stage -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $Bin | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $Share "wasm") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $Share "proto") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $Share "dashboard") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $Share "brand") | Out-Null

$exes = @("ai-runtime", "intent-verifier", "intentkernel-update", "intentkernel", "ikd-verify")
foreach ($name in $exes) {
    Copy-Item (Join-Path $Root "target\release\$name.exe") $Bin
}

if (Test-Path ".\kernel\build_parser_wasm.sh") {
    if (Get-Command "bash" -ErrorAction SilentlyContinue) {
        bash ./kernel/build_parser_wasm.sh
    }
}

if (Test-Path ".\build\wasm\intent_parser.wasm") {
    Copy-Item ".\build\wasm\intent_parser.wasm" (Join-Path $Share "wasm\intent_parser.wasm")
}

Copy-Item ".\core\ai-runtime\proto\intentkernel.proto" (Join-Path $Share "proto\intentkernel.proto")
Copy-Item ".\share\dashboard\index.html" (Join-Path $Share "dashboard\index.html")
if (Test-Path ".\share\brand\intent-kernel-logo.png") {
    Copy-Item ".\share\brand\intent-kernel-logo.png" (Join-Path $Share "brand\intent-kernel-logo.png")
}
if (Test-Path ".\share\brand\intent-kernel-logo-dark.png") {
    Copy-Item ".\share\brand\intent-kernel-logo-dark.png" (Join-Path $Share "brand\intent-kernel-logo-dark.png")
}

$TauriExe = Join-Path $Root "shell\tauri-app\src-tauri\target\release\intentkernel-shell.exe"
if (-not (Test-Path $TauriExe)) {
    $TauriExe = Join-Path $Root "target\release\intentkernel-shell.exe"
}
if (Test-Path $TauriExe) {
    Copy-Item $TauriExe (Join-Path $Bin "intentkernel-dashboard.exe")
    Write-Host "✓ Bundled intentkernel-dashboard.exe" -ForegroundColor Green
} else {
    Write-Host "    NOTE: build Tauri shell for native dashboard (npm run tauri build)" -ForegroundColor Yellow
}
Set-Content -Path (Join-Path $Stage "VERSION") -Value $Version -NoNewline

$ZipName = "intentkernel-$Version-windows-x86_64.zip"
$DistDir = Join-Path $Root "dist"
$ZipPath = Join-Path $DistDir $ZipName

if (-not (Test-Path $DistDir)) {
    New-Item -ItemType Directory -Force -Path $DistDir | Out-Null
}
if (Test-Path $ZipPath) {
    Remove-Item $ZipPath -Force
}

Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $ZipPath -Force

Write-Host "✓ Package created: $ZipPath" -ForegroundColor Green
Write-Host "Install locally with:" -ForegroundColor Green
Write-Host "  powershell -ExecutionPolicy Bypass -File install.ps1 -Local"

if ($StageIso) {
    $StageScript = Join-Path $Root "scripts\stage-iso.ps1"
    & $StageScript -RepoRoot $Root -IsoRoot $IsoRoot
}