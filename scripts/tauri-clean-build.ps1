$ErrorActionPreference="Stop"
$env:PATH = "C:\Users\wendy\.cargo\bin;C:\msys64\mingw64\bin;" + $env:PATH

Get-Process host-tauri -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 1

Remove-Item -Recurse -Force "D:\Projects\l3dg3rr\target-windows\debug\host-tauri.exe" -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force "D:\Projects\l3dg3rr\target-windows\debug\ledgerr-host.exe" -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force "D:\Projects\l3dg3rr\target-windows\debug\.fingerprint\ledgerr-host-*" -ErrorAction SilentlyContinue

Set-Location "D:\Projects\l3dg3rr"
Write-Host "[build] compiling..."
$buildOut = cmd.exe /c "C:\Users\wendy\.cargo\bin\cargo.exe build -p ledgerr-host --bin host-tauri --target-dir D:\Projects\l3dg3rr\target-windows 2>&1"
if ($LASTEXITCODE -eq 0) { Write-Host "  build OK" } else { Write-Host "  build FAILED (exit $LASTEXITCODE)" }

Write-Host "[launch] starting with CDP..."
$buildNum = (Get-Content "D:\Projects\l3dg3rr\.build_counter" -Raw).Trim()
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = "D:\Projects\l3dg3rr\target-windows\debug\host-tauri.exe"
$psi.WorkingDirectory = "D:\Projects\l3dg3rr\crates\ledgerr-host"
$psi.UseShellExecute = $false
$psi.EnvironmentVariables["WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS"] = "--remote-debugging-port=19233"
$psi.EnvironmentVariables["TAURI_BUILD_NUMBER"] = $buildNum
$proc = [System.Diagnostics.Process]::Start($psi)
Write-Host "  PID: $($proc.Id)"
Start-Sleep 8

Write-Host "[cdp] checking..."
try {
  $v = curl.exe -s http://127.0.0.1:19232/json/version
  if ($v -match "WebView") {
    Write-Host "CDP OK"
    Write-Host ($v.Substring(0, [math]::Min(300, $v.Length)))
  } else {
    Write-Host "CDP FAILED: $v"
  }
} catch {
  Write-Host "CDP FAILED: $_"
}
