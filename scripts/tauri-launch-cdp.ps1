$ErrorActionPreference="Stop"
$env:TAURI_CDP_PORT="19227"
$env:TAURI_BUILD_NUMBER="15"

Get-Process host-tauri -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 2

$exe = "D:\Projects\l3dg3rr\target-windows\debug\host-tauri.exe"
if (-not (Test-Path $exe)) {
  $exe = "D:\Projects\l3dg3rr\target\debug\host-tauri.exe"
}

Write-Host "Launching: $exe"
Start-Process -FilePath $exe -WorkingDirectory "D:\Projects\l3dg3rr\crates\ledgerr-host" -PassThru
Start-Sleep 8

# CDP check
try {
  $v = curl.exe -s http://127.0.0.1:19227/json/version
  if ($v -match "WebView") {
    Write-Host "CDP OK on 19227"
    Write-Host ($v.Substring(0, [math]::Min(300, $v.Length)))
  } else {
    Write-Host "CDP FAILED: $v"
  }
} catch {
  Write-Host "CDP FAILED: $_"
}
