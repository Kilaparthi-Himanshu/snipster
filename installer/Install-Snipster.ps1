
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
    Write-Host "Find Snipster in the Windows Start menu."
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
