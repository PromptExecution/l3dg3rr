$ErrorActionPreference="Stop"
Get-Process host-tauri -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 2

$env:PATH = "C:\Users\wendy\.cargo\bin;C:\msys64\mingw64\bin;" + $env:PATH
Set-Location "D:\Projects\l3dg3rr"

Write-Host "[build] release (tauri bundle)..."
Set-Location "D:\Projects\l3dg3rr\crates\ledgerr-host"
cmd.exe /c "C:\Users\wendy\.cargo\bin\cargo.exe tauri build --bundles msi"
if ($LASTEXITCODE -ne 0) { throw "tauri build failed" }
Set-Location "D:\Projects\l3dg3rr"

$msi = "D:\Projects\l3dg3rr\target\release\bundle\msi\ledgrrr_1.8.0_x64_en-US.msi"
if (-not (Test-Path $msi)) { throw "MSI not found at $msi" }

Write-Host "[install] MSI..."
$old = Get-WmiObject Win32_Product | Where-Object { $_.Name -match "ledgrrr" }
if ($old) { $old.Uninstall() | Out-Null; Start-Sleep 2 }
Start-Process msiexec -Verb RunAs -ArgumentList "/i `"$msi`" /quiet /norestart" -Wait
Write-Host "Installed: C:\Program Files\ledgrrr\ledgrrr.exe"

# Verify
Get-Item "C:\Program Files\ledgrrr\ledgrrr.exe" -ErrorAction SilentlyContinue | ForEach-Object {
  Write-Host ("  " + [math]::Round($_.Length/1KB,1) + " KB")
}
