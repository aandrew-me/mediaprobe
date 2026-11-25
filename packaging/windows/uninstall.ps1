$ErrorActionPreference = "Stop"

$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\MediaProbe"
$StartMenu = [Environment]::GetFolderPath("StartMenu")
$ShortcutPath = Join-Path $StartMenu "Programs\MediaProbe.lnk"
$RegPath = "HKCU:\Software\Classes\*\shell\MediaProbe"

if (Test-Path $RegPath) {
    Remove-Item -Path $RegPath -Recurse -Force
    Write-Host "Removed Explorer context menu entry." -ForegroundColor Yellow
}

if (Test-Path $ShortcutPath) {
    Remove-Item -Path $ShortcutPath -Force
    Write-Host "Removed Start Menu shortcut." -ForegroundColor Yellow
}

if (Test-Path $InstallDir) {
    Remove-Item -Path $InstallDir -Recurse -Force
    Write-Host "Removed $InstallDir." -ForegroundColor Yellow
}

Write-Host "MediaProbe uninstalled." -ForegroundColor Green
