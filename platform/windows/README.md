# Windows Platform — AI OS Context Manager

Installs the AI Context Manager as a Windows Service.

## Prerequisites

- Visual Studio 2022 or Build Tools
- Rust toolchain (`rustup`)
- WiX Toolset v3 (for building the MSI installer)
- Administrator rights

## Build the MSI

```powershell
# Build the Rust binary
cd ..\..\core\ai-runtime
cargo build --release

# Build the Windows service wrapper
cd ..\..\platform\windows\service
dotnet build -c Release

# Build the MSI
cd ..
candle.exe installer.wix
light.exe installer.wixobj -o AIContextManager.msi
```

## Installation

```powershell
msiexec /i AIContextManager.msi /qn
```

## Service Management

```powershell
Get-Service AIContextManager
Start-Service AIContextManager
Stop-Service AIContextManager
```
