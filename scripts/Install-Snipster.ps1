
$ErrorActionPreference = "Stop"

$folder = $PSScriptRoot
$certificate = Join-Path $folder "Snipster.cer"
$package = Join-Path $folder "Snipster.msix"

$expectedThumbprint = "CC502421E7ACBFD825B5220947181D21602DDFB3"

try {
    Write-Host ""
    Write-Host "Installing Snipster..." -ForegroundColor Cyan

    if (-not (Test-Path $certificate)) {
        throw "Snipster.cer not found."
    }

    if (-not (Test-Path $package)) {
        throw "Snipster.msix not found."
    }

    $cert = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new(
        $certificate
    )

    if ($cert.Thumbprint -ne $expectedThumbprint) {
        throw "Certificate fingerprint mismatch. Installation aborted."
    }

    Write-Host "Certificate fingerprint verified." -ForegroundColor Green

    $existing = Get-ChildItem Cert:\LocalMachine\TrustedPeople |
        Where-Object Thumbprint -eq $expectedThumbprint

    if (-not $existing) {
        Import-Certificate `
            -FilePath $certificate `
            -CertStoreLocation "Cert:\LocalMachine\TrustedPeople" |
            Out-Null
    }

    Write-Host "Installing application package..."

    Add-AppxPackage -Path $package

    Write-Host ""
    Write-Host "Snipster installed successfully!" -ForegroundColor Green

    # Find the newly installed application.
    $app = Get-AppxPackage -Name "Snipster" |
        Sort-Object Version -Descending |
        Select-Object -First 1

    if (-not $app) {
        throw "Installation succeeded, but Snipster could not be located."
    }

    # Construct the application's Windows activation ID.
    $appId = "$($app.PackageFamilyName)!Snipster"

    Write-Host "Launching Snipster..." -ForegroundColor Cyan

    # Launch through Windows Explorer so the app starts
    # through normal Windows application activation.
    Start-Process -FilePath "$env:WINDIR\explorer.exe" `
        -ArgumentList "shell:AppsFolder\$appId"

    Write-Host ""
    Write-Host "Snipster is now running in your system tray!" -ForegroundColor Green
    Write-Host "Press Win + Ctrl + Shift + S to capture a screenshot."
}
catch {
    Write-Host ""
    Write-Host "Installation failed:" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
}
finally {
    Write-Host ""
    Read-Host "Press Enter to close"
}
