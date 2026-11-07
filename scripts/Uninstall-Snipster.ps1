
$ErrorActionPreference = "Stop"

try {
    Write-Host ""
    Write-Host "Uninstalling Snipster..." -ForegroundColor Cyan

    $package = Get-AppxPackage -Name Snipster

    if (-not $package) {
        Write-Host "Snipster is not installed."
    }
    else {
        Get-Process snipster -ErrorAction SilentlyContinue |
            Stop-Process -Force

        Remove-AppxPackage -Package $package.PackageFullName

        Write-Host "Snipster successfully uninstalled!" -ForegroundColor Green
    }

    Write-Host ""
    Write-Host "The signing certificate has been retained."
    Write-Host "It can be removed separately if no longer needed."
}
catch {
    Write-Host ""
    Write-Host "Uninstallation failed:" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
}
finally {
    Write-Host ""
    Read-Host "Press Enter to close"
}
