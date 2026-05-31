Param(
    [Parameter(Mandatory = $false)]
    [ValidateSet('install', 'migrate', 'usb', 'scan')]
    [string]$Mode = 'install'
)

Write-Host "CRASS LAUNCH — Windows installer" -ForegroundColor Cyan
Write-Host "Mode: $Mode"

function Run-Install {
    Write-Host "[CRASS LAUNCH] Preparing Windows installation..."
    Write-Host "Checking EFI/Legacy boot and available volumes..."
    Write-Host "This is a scaffold. Replace this with real installation logic."
}

function Run-Migrate {
    Write-Host "[CRASS LAUNCH] Starting CRASS MIGRATE..."
    python3 "$PSScriptRoot\..\migration\crass_migrate.py"
}

function Generate-Usb {
    Write-Host "[CRASS LAUNCH] Creating bootable CRASS USB media..."
    Write-Host "This operation requires administrative privileges."
}

switch ($Mode) {
    'install' { Run-Install }
    'migrate' { Run-Migrate }
    'usb' { Generate-Usb }
    'scan' { Write-Host "Scanning system for disks and bootloaders..." }
    default { Write-Host "Unsupported mode: $Mode" }
}
