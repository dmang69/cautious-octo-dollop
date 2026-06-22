# Stage Intent Kernel Windows install media for offline ISO/USB use
# Usage: powershell -ExecutionPolicy Bypass -File scripts\stage-iso.ps1

param(
    [string]$RepoRoot = "C:\Users\Dizzle\Documents\GitHub\cautious-octo-dollop",
    [string]$IsoRoot = "C:\Users\Dizzle\IntentKernelISO"
)

$ErrorActionPreference = "Stop"

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

$PackageScript = Join-Path $Root "scripts\package-windows.ps1"
if (-not (Test-Path $PackageScript)) {
    throw "Missing package script: $PackageScript"
}

& $PackageScript -RepoRoot $Root

$Version = "1.0.0"
$Package = "intentkernel-$Version-windows-x86_64.zip"
$ZipPath = Join-Path $Root "dist\$Package"
$Stage = Join-Path $Root "dist\IntentKernel"

if (-not (Test-Path $ZipPath)) {
    throw "Package zip not found: $ZipPath"
}

New-Item -ItemType Directory -Force -Path $IsoRoot | Out-Null

Copy-Item $ZipPath (Join-Path $IsoRoot $Package) -Force
Copy-Item (Join-Path $Root "install.ps1") (Join-Path $IsoRoot "install.ps1") -Force
robocopy $Stage (Join-Path $IsoRoot "IntentKernel") /E /NFL /NDL /NJH /NJS /nc /ns /np | Out-Null
if ($LASTEXITCODE -ge 8) {
    throw "Failed to stage IntentKernel tree to $IsoRoot"
}

Write-Host "✓ ISO media staged to $IsoRoot" -ForegroundColor Green
Write-Host "Install with:" -ForegroundColor Green
Write-Host "  powershell -ExecutionPolicy Bypass -File `"$IsoRoot\install.ps1`" -Iso"