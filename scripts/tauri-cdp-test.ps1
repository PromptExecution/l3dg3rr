$ErrorActionPreference="Stop"
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=19222"
$env:TAURI_BUILD_NUMBER = (Get-Content "D:\Projects\l3dg3rr\.build_counter" -Raw).Trim()

Get-Process host-tauri -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 1

$proc = Start-Process -FilePath "D:\Projects\l3dg3rr\target\debug\host-tauri.exe" -WorkingDirectory "D:\Projects\l3dg3rr\crates\ledgerr-host" -PassThru
Write-Host "Launched PID: $($proc.Id)"

Start-Sleep 6

# Try CDP connection
try {
  $resp = curl.exe -s http://127.0.0.1:19222/json/version
  if ($resp -match "WebView") {
    Write-Host "CDP CONNECTED on port 19222"
    Write-Host ($resp.Substring(0, [math]::Min(300, $resp.Length)))
    # List available pages
    $pages = curl.exe -s http://127.0.0.1:19222/json
    Write-Host "Pages: $pages"
  } else {
    Write-Host "CDP response but no WebView match: $resp"
  }
} catch {
  Write-Host "CDP FAILED: $_"
  # Try default port
  try {
    $resp2 = curl.exe -s http://127.0.0.1:9222/json/version
    Write-Host "Port 9222 responded: $($resp2.Substring(0, [math]::Min(200, $resp2.Length)))"
  } catch {
    Write-Host "Port 9222 also failed"
  }
}

$proc.Kill()
