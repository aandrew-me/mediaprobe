$ErrorActionPreference = "Stop"

$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\MediaProbe"
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$SourceExe = Join-Path $RepoRoot "target\release\mediaprobe.exe"

if (-not (Test-Path $SourceExe)) {
    Write-Host "Release binary not found at $SourceExe. Building..." -ForegroundColor Yellow
    Push-Location $RepoRoot
    cargo build --release
    Pop-Location
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$TargetExe = Join-Path $InstallDir "mediaprobe.exe"
Copy-Item -Force $SourceExe $TargetExe

# Create Start Menu shortcut
$StartMenu = [Environment]::GetFolderPath("StartMenu")
$ShortcutPath = Join-Path $StartMenu "Programs\MediaProbe.lnk"
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($ShortcutPath)
$Shortcut.TargetPath = $TargetExe
$Shortcut.IconLocation = "$TargetExe,0"
$Shortcut.Description = "Inspect media file metadata"
$Shortcut.Save()

# Register Windows Explorer context menu ("Inspect with MediaProbe") for current user (no admin needed)
$RegPath = "HKCU:\Software\Classes\*\shell\MediaProbe"
$CmdPath = Join-Path $RegPath "command"
New-Item -Path $RegPath -Force | Out-Null
Set-ItemProperty -Path $RegPath -Name "(Default)" -Value "Inspect with MediaProbe"
Set-ItemProperty -Path $RegPath -Name "Icon" -Value "$TargetExe,0"

New-Item -Path $CmdPath -Force | Out-Null
Set-ItemProperty -Path $CmdPath -Name "(Default)" -Value "`"$TargetExe`" `"%1`""

Write-Host "Installed MediaProbe to $TargetExe" -ForegroundColor Green
Write-Host "Added Start Menu shortcut and Explorer context menu entry 'Inspect with MediaProbe'." -ForegroundColor Green
