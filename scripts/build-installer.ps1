
$ErrorActionPreference = "Stop"

# --------------------------------------------------
# 1. Resolve project directories
# --------------------------------------------------

$root = Split-Path $PSScriptRoot -Parent
$package = Join-Path $root "package"
$installer = Join-Path $root "installer"

New-Item -ItemType Directory -Path $installer -Force |
    Out-Null

# --------------------------------------------------
# 2. Build Rust executable
# --------------------------------------------------

Write-Host "`n[1/7] Building Snipster..." -ForegroundColor Cyan

Push-Location $root

try {
    cargo build --release

    if ($LASTEXITCODE -ne 0) {
        throw "Rust build failed."
    }
}
finally {
    Pop-Location
}

# --------------------------------------------------
# 3. Update MSIX payload
# --------------------------------------------------

Write-Host "`n[2/7] Updating MSIX payload..." -ForegroundColor Cyan

Copy-Item `
    (Join-Path $root "target\release\snipster.exe") `
    (Join-Path $package "payload\snipster.exe") `
    -Force

Copy-Item `
    (Join-Path $package "Package.appxmanifest") `
    (Join-Path $package "payload\AppxManifest.xml") `
    -Force

# --------------------------------------------------
# 4. Locate Windows SDK tools
# --------------------------------------------------

Write-Host "`n[3/7] Checking Windows SDK..." -ForegroundColor Cyan

$bin = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64"

if (-not (Test-Path "$bin\makeappx.exe")) {
    throw "MakeAppx not found. Install the Windows SDK."
}

if (-not (Test-Path "$bin\signtool.exe")) {
    throw "SignTool not found. Install the Windows SDK."
}

# --------------------------------------------------
# 5. Generate MSIX package
# --------------------------------------------------

Write-Host "`n[4/7] Packaging MSIX..." -ForegroundColor Cyan

& "$bin\makeappx.exe" pack `
    /d (Join-Path $package "payload") `
    /p (Join-Path $package "Snipster.msix") `
    /o

if ($LASTEXITCODE -ne 0) {
    throw "MSIX packaging failed."
}

# --------------------------------------------------
# 6. Sign and verify package
# --------------------------------------------------

Write-Host "`n[5/7] Signing MSIX..." -ForegroundColor Cyan

$thumbprint = "CC502421E7ACBFD825B5220947181D21602DDFB3"

& "$bin\signtool.exe" sign `
    /fd SHA256 `
    /sha1 $thumbprint `
    /s My `
    (Join-Path $package "Snipster.msix")

if ($LASTEXITCODE -ne 0) {
    throw "MSIX signing failed."
}

Write-Host "Verifying package signature..." -ForegroundColor Cyan

& "$bin\signtool.exe" verify /pa /v `
    (Join-Path $package "Snipster.msix")

if ($LASTEXITCODE -ne 0) {
    throw "MSIX signature verification failed."
}

# --------------------------------------------------
# 7. Prepare installer directory
# --------------------------------------------------

Write-Host "`n[6/7] Preparing installer files..." -ForegroundColor Cyan

# Copy signed package.
Copy-Item `
    (Join-Path $package "Snipster.msix") `
    (Join-Path $installer "Snipster.msix") `
    -Force

# Copy public signing certificate.
Copy-Item `
    (Join-Path $package "Snipster.cer") `
    (Join-Path $installer "Snipster.cer") `
    -Force

# Copy installation and uninstallation scripts.
$installerScripts = @(
    "Install Snipster.bat",
    "Install-Snipster.ps1",
    "Uninstall Snipster.bat",
    "Uninstall-Snipster.ps1"
)

foreach ($file in $installerScripts) {
    $source = Join-Path $PSScriptRoot $file
    $destination = Join-Path $installer $file

    if (-not (Test-Path $source)) {
        throw "Installer script not found: $source"
    }

    Copy-Item $source $destination -Force
}

# --------------------------------------------------
# Finished
# --------------------------------------------------

Write-Host "`n[7/7] Build completed!" -ForegroundColor Green

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "       SNIPSTER BUILD SUCCESSFUL" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green

Write-Host "`nInstaller directory:"
Write-Host $installer -ForegroundColor Yellow

Write-Host "`nFiles generated:"
Get-ChildItem $installer | Select-Object Name, Length

Write-Host "`nReady for testing and distribution!" -ForegroundColor Green

# Run from repository root:
# .\scripts\build-installer.ps1
