# Intent Kernel AI OS - Windows Simple Installer
# Run: powershell -ExecutionPolicy Bypass -File install.ps1
# Local repo:  powershell -ExecutionPolicy Bypass -File install.ps1 -Local
# ISO media:   powershell -ExecutionPolicy Bypass -File install.ps1 -Iso
# Build repo:  C:\Users\Dizzle\Documents\GitHub\cautious-octo-dollop
# ISO root:    C:\Users\Dizzle\IntentKernelISO

param(
    [switch]$Local,
    [switch]$Iso,
    [string]$InstallDir = "D:\intentkernel",
    [string]$RepoRoot = "C:\Users\Dizzle\Documents\GitHub\cautious-octo-dollop",
    [string]$IsoRoot = "C:\Users\Dizzle\IntentKernelISO",
    [string]$ZipPath = "",
    [string]$SourceDir = ""
)

$ErrorActionPreference = "Stop"

function Resolve-ExistingPath {
    param([string[]]$Candidates)
    foreach ($candidate in $Candidates) {
        if ($candidate -and (Test-Path $candidate)) {
            return (Resolve-Path $candidate).Path
        }
    }
    return $null
}

function Find-PackageZip {
    param(
        [string]$PackageName,
        [string[]]$SearchRoots
    )
    foreach ($root in $SearchRoots) {
        if (-not $root) { continue }
        foreach ($relative in @(
            $PackageName,
            "dist\$PackageName",
            "IntentKernel\$PackageName"
        )) {
            $candidate = Join-Path $root $relative
            if (Test-Path $candidate) {
                return (Resolve-Path $candidate).Path
            }
        }
    }
    return $null
}

function Find-StagedTree {
    param([string[]]$SearchRoots)
    foreach ($root in $SearchRoots) {
        if (-not $root) { continue }
        foreach ($relative in @("", "IntentKernel", "dist\IntentKernel")) {
            $candidate = if ($relative) { Join-Path $root $relative } else { $root }
            if (Test-Path (Join-Path $candidate "bin\ai-runtime.exe")) {
                return (Resolve-Path $candidate).Path
            }
        }
    }
    return $null
}

function Copy-StagedTree {
    param(
        [string]$From,
        [string]$To
    )
    New-Item -ItemType Directory -Force -Path $To | Out-Null
    robocopy $From $To /E /NFL /NDL /NJH /NJS /nc /ns /np | Out-Null
    if ($LASTEXITCODE -ge 8) {
        throw "Failed to copy staged install tree from $From to $To (robocopy exit $LASTEXITCODE)"
    }
}

# Configuration
$Version = "1.0.0"
$InstallDir = $InstallDir.TrimEnd('\')
$IsoRoot = $IsoRoot.TrimEnd('\')
$DownloadUrl = "https://releases.intentkernel.ai"
$ScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

$GitHubRepo = Join-Path $env:USERPROFILE "Documents\GitHub\cautious-octo-dollop"
$ClionRepo = Join-Path $env:USERPROFILE "CLionProjects\cautious-octo-dollop"
$ProfileRepo = Join-Path $env:USERPROFILE "cautious-octo-dollop"
$Root = Resolve-ExistingPath @(
    $RepoRoot,
    $GitHubRepo,
    $ClionRepo,
    $ProfileRepo,
    $ScriptRoot,
    $IsoRoot,
    "C:\Users\Dizzle\CLionProjects\cautious-octo-dollop",
    "C:\Users\Dizzle\cautious-octo-dollop",
    "D:\cautious-octo-dollop"
)
if (-not $Root) {
    $Root = $RepoRoot
}

if ($Iso) {
    $Local = $true
}

# Banner
Write-Host @"

╔═══════════════════════════════════════════════════════════════╗
║              INTENT KERNEL AI OS - WINDOWS INSTALLER         ║
║                        Version $Version                         ║
╚═══════════════════════════════════════════════════════════════╝

"@ -ForegroundColor Cyan

# Check admin
if (-NOT ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host "❌ This script requires Administrator privileges" -ForegroundColor Red
    Write-Host "Right-click and select 'Run as Administrator'" -ForegroundColor Yellow
    exit 1
}

Write-Host "✓ Running with Administrator privileges" -ForegroundColor Green
Write-Host "✓ Install directory: $InstallDir" -ForegroundColor Green
if ($Iso -or (Test-Path $IsoRoot)) {
    Write-Host "✓ ISO media root: $IsoRoot" -ForegroundColor Green
}

$Package = "intentkernel-$Version-windows-x86_64.zip"
$SearchRoots = @($IsoRoot, $Root, $RepoRoot, $ScriptRoot) | Select-Object -Unique
$StagedTree = $null
$DownloadPath = $ZipPath
$UsedStagedCopy = $false

if ($Local) {
    Write-Host "`nUsing local package..." -ForegroundColor Yellow

    if (-not $DownloadPath) {
        $DownloadPath = Find-PackageZip -PackageName $Package -SearchRoots $SearchRoots
    }

    if (-not $DownloadPath) {
        $StagedTree = Find-StagedTree -SearchRoots $SearchRoots
        if (-not $StagedTree -and -not $SourceDir) {
            $SourceDir = Resolve-ExistingPath @(
                (Join-Path $Root "dist\IntentKernel"),
                (Join-Path $IsoRoot "IntentKernel"),
                $IsoRoot
            )
        }
        if (-not $StagedTree -and $SourceDir -and (Test-Path (Join-Path $SourceDir "bin\ai-runtime.exe"))) {
            $StagedTree = (Resolve-Path $SourceDir).Path
        }
        if ($StagedTree) {
            Write-Host "✓ Found staged install tree: $StagedTree" -ForegroundColor Green
        } elseif ($SourceDir -and (Test-Path $SourceDir)) {
            $DownloadPath = "$env:TEMP\$Package"
            Write-Host "Packaging from $SourceDir" -ForegroundColor Yellow
            if (-not (Test-Path (Split-Path $DownloadPath -Parent))) {
                New-Item -ItemType Directory -Force -Path (Split-Path $DownloadPath -Parent) | Out-Null
            }
            Compress-Archive -Path (Join-Path $SourceDir "*") -DestinationPath $DownloadPath -Force
        } else {
            Write-Host "❌ Local package not found." -ForegroundColor Red
            Write-Host "  ISO media:  $IsoRoot" -ForegroundColor Yellow
            Write-Host "  Build repo: $RepoRoot" -ForegroundColor Yellow
            Write-Host "  Build with: powershell -ExecutionPolicy Bypass -File .\scripts\package-windows.ps1" -ForegroundColor Yellow
            exit 1
        }
    } else {
        Write-Host "✓ Local package ready: $DownloadPath" -ForegroundColor Green
    }
} else {
    Write-Host "`nResolving install package..." -ForegroundColor Yellow
    if (-not $DownloadPath) {
        $DownloadPath = Find-PackageZip -PackageName $Package -SearchRoots $SearchRoots
    }
    if ($DownloadPath) {
        Write-Host "✓ Using offline package: $DownloadPath" -ForegroundColor Green
    } else {
        Write-Host "Downloading Intent Kernel AI OS..." -ForegroundColor Yellow
        $DownloadPath = "$env:TEMP\$Package"
        try {
            Invoke-WebRequest -Uri "$DownloadUrl/$Package" -OutFile $DownloadPath -UseBasicParsing
        } catch {
            Write-Host "❌ Download failed: $_" -ForegroundColor Red
            Write-Host "Place media under $IsoRoot or run with -Local / -Iso" -ForegroundColor Yellow
            exit 1
        }
        Write-Host "✓ Downloaded successfully" -ForegroundColor Green
    }
}

# Extract or copy
Write-Host "`nInstalling files..." -ForegroundColor Yellow

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
if ($StagedTree) {
    Copy-StagedTree -From $StagedTree -To $InstallDir
    $UsedStagedCopy = $true
    Write-Host "✓ Copied staged install tree" -ForegroundColor Green
} else {
    Expand-Archive -Path $DownloadPath -DestinationPath $InstallDir -Force
    Write-Host "✓ Extracted successfully" -ForegroundColor Green
}

# Install services
Write-Host "`nInstalling services..." -ForegroundColor Yellow

$env:INTENTKERNEL_ROOT = $InstallDir
[Environment]::SetEnvironmentVariable("INTENTKERNEL_ROOT", $InstallDir, "Machine")
[Environment]::SetEnvironmentVariable("INTENTKERNEL_ISO_ROOT", $IsoRoot, "Machine")
& "$InstallDir\bin\ai-runtime.exe" install
& "$InstallDir\bin\intent-verifier.exe" install

Start-Service IntentKernelRuntime -ErrorAction SilentlyContinue
Start-Service IntentKernelVerifier -ErrorAction SilentlyContinue

Write-Host "✓ Services installed and started" -ForegroundColor Green

# Configure firewall
Write-Host "`nConfiguring firewall..." -ForegroundColor Yellow

$existing = Get-NetFirewallRule -DisplayName "Intent Kernel gRPC" -ErrorAction SilentlyContinue
if (-not $existing) {
    New-NetFirewallRule -DisplayName "Intent Kernel gRPC" -Direction Inbound -Protocol TCP -LocalPort 50051 -Action Allow | Out-Null
}

Write-Host "✓ Firewall configured" -ForegroundColor Green

# Add to PATH
Write-Host "`nAdding to PATH..." -ForegroundColor Yellow

$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($CurrentPath -notlike "*$InstallDir\bin*") {
    [Environment]::SetEnvironmentVariable("Path", "$CurrentPath;$InstallDir\bin", "Machine")
}

Write-Host "✓ Added to PATH" -ForegroundColor Green

# Create shortcuts
Write-Host "`nCreating shortcuts..." -ForegroundColor Yellow

$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut("$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Intent Kernel Dashboard.lnk")
$Shortcut.TargetPath = "$InstallDir\bin\ai-runtime.exe"
$Shortcut.Arguments = "--dashboard"
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.Save()

Write-Host "✓ Shortcuts created" -ForegroundColor Green

# Setup auto-update
Write-Host "`nSetting up auto-update..." -ForegroundColor Yellow

$existingTask = Get-ScheduledTask -TaskName "IntentKernel Auto Update" -ErrorAction SilentlyContinue
if ($existingTask) {
    Unregister-ScheduledTask -TaskName "IntentKernel Auto Update" -Confirm:$false
}

$Action = New-ScheduledTaskAction -Execute "$InstallDir\bin\intentkernel-update.exe"
$Trigger = New-ScheduledTaskTrigger -Daily -At 2am
Register-ScheduledTask -Action $Action -Trigger $Trigger -TaskName "IntentKernel Auto Update" -Description "Automatic updates for Intent Kernel AI OS" | Out-Null

Write-Host "✓ Auto-update configured" -ForegroundColor Green

# Success
Write-Host @"

╔═══════════════════════════════════════════════════════════════╗
║                  ✓ INSTALLATION COMPLETE!                    ║
╚═══════════════════════════════════════════════════════════════╝

Intent Kernel AI OS has been successfully installed!

Next steps:
  1. Open Start Menu → Intent Kernel Dashboard
  2. Configure your first intent: intentkernel configure
  3. View docs: https://docs.intentkernel.ai

Thank you for installing Intent Kernel AI OS!

"@ -ForegroundColor Green

# Cleanup
if (-not $UsedStagedCopy -and -not $Local -and -not $ZipPath -and $DownloadPath -like "$env:TEMP\*") {
    Remove-Item $DownloadPath -Force -ErrorAction SilentlyContinue
}