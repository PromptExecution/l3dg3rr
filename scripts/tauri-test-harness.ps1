param(
  [string]$BinaryPath="D:\Projects\l3dg3rr\target\debug\host-tauri.exe",
  [string]$WorkDir="D:\Projects\l3dg3rr",
  [int]$WaitSeconds=12,
  [int]$KillDelayMs=500,
  [int]$CountdownMs=5000,
  [string]$ScreenshotDir=$null
)

if (-not $ScreenshotDir) { $ScreenshotDir = $env:TEMP }

$ErrorActionPreference="Stop"
$runId=[guid]::NewGuid().ToString().Substring(0,8)
$testUuid=[guid]::NewGuid().ToString()
$screenshotPath=Join-Path $ScreenshotDir "host-tauri-screenshot-$runId.png"
$screenshotBase64Path=Join-Path $ScreenshotDir "host-tauri-screenshot-$runId.b64.txt"
$start=[datetime]::UtcNow
$telemetryFile=Join-Path $env:TEMP "host-tauri-telemetry-$runId.txt"
$sloPath=Join-Path $env:TEMP "host-tauri-slo-$runId.json"
$stderrFile=Join-Path $env:TEMP "host-tauri-stderr-$runId.txt"
$stdoutFile=Join-Path $env:TEMP "host-tauri-stdout-$runId.txt"

Write-Host "=== Tauri Test Harness ==="
Write-Host "Run ID:     $runId"
Write-Host "UUID:       $testUuid"
Write-Host "Countdown:  ${CountdownMs}ms"
Write-Host "KillDelay:  ${KillDelayMs}ms"
Write-Host "Screenshot: $screenshotPath"
Write-Host "Binary:     $BinaryPath"
Write-Host ""

# ── Launch ────────────────────────────────────────────────────────────────────
$env:PATH="C:\Users\wendy\.cargo\bin;C:\msys64\mingw64\bin;"+$env:PATH
$env:RUST_BACKTRACE="full"
$env:TAURI_TEST_UUID=$testUuid
$env:TAURI_TEST_KILL_DELAY="$CountdownMs"
$env:TAURI_TEST_SCREENSHOT_PATH=$screenshotPath

$psi=New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName=$BinaryPath
$psi.WorkingDirectory=$WorkDir
$psi.UseShellExecute=$false
$psi.RedirectStandardError=$true
$psi.RedirectStandardOutput=$true
$psi.EnvironmentVariables["PATH"]=$env:PATH
$psi.EnvironmentVariables["RUST_BACKTRACE"]="full"
$psi.EnvironmentVariables["TAURI_TEST_UUID"]=$testUuid
$psi.EnvironmentVariables["TAURI_TEST_KILL_DELAY"]="$CountdownMs"
$psi.EnvironmentVariables["TAURI_TEST_SCREENSHOT_PATH"]=$screenshotPath

$proc=[System.Diagnostics.Process]::Start($psi)
Write-Host "Launched PID: $($proc.Id)"

# ── Wait for the countdown to finish (window should close itself) ─────────────
$waitForExit = $CountdownMs + 3000  # countdown + margin
$elapsed=0
while ((-not $proc.HasExited)-and($elapsed-lt$waitForExit)){Start-Sleep -Milliseconds 200;$elapsed+=200}

# ── Screenshot (after window has had time to render) ──────────────────────────
Write-Host "Capturing screenshot..."
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms
try {
  Start-Sleep 1
  $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  $bmp = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
  $gfx = [System.Drawing.Graphics]::FromImage($bmp)
  $gfx.CopyFromScreen($bounds.X, $bounds.Y, 0, 0, $bounds.Size)
  $bmp.Save($screenshotPath, [System.Drawing.Imaging.ImageFormat]::Png)
  $gfx.Dispose()
  $bmp.Dispose()
  Write-Host "  saved: $screenshotPath"

  # Base64 encode
  $bytes = [System.IO.File]::ReadAllBytes($screenshotPath)
  $b64 = [System.Convert]::ToBase64String($bytes)
  $b64 | Out-File -Encoding ascii $screenshotBase64Path
  Write-Host "  base64: $screenshotBase64Path ($($b64.Length) chars)"
} catch {
  Write-Host "  screenshot failed: $_"
  $screenshotPath = ""
  $screenshotBase64Path = ""
}

# ── 3x Redundant kill ─────────────────────────────────────────────────────────
for ($i=1;$i-le3;$i++){if($proc.HasExited){break};Write-Host "Kill attempt $i...";$proc.Kill();Start-Sleep -Milliseconds $KillDelayMs;if($proc.HasExited){Write-Host "  killed on attempt $i";break}}
if(-not $proc.HasExited){Write-Host "WMI force kill...";Get-WmiObject Win32_Process|Where-Object{$_.ProcessId-eq$proc.Id}|ForEach-Object{$_.Terminate()};Start-Sleep 1}

# ── Collect output ────────────────────────────────────────────────────────────
$stdout="";$stderr=""
try{if(-not $proc.StandardOutput.EndOfStream){$stdout=$proc.StandardOutput.ReadToEnd()};if(-not $proc.StandardError.EndOfStream){$stderr=$proc.StandardError.ReadToEnd()}}catch{Write-Host "Warning: output read error: $_"}
$stdout|Out-File -Encoding utf8 $stdoutFile
$stderr|Out-File -Encoding utf8 $stderrFile

# ── Write telemetry ───────────────────────────────────────────────────────────
$lines=@()
$lines+="=== TELEMETRY ==="
$lines+="run_id:   $runId"
$lines+="uuid:     $testUuid"
$lines+="pid:      $($proc.Id)"
$lines+="exit_code: $($proc.ExitCode)"
$lines+="duration_ms: $([math]::Round(((Get-Date)-$start).TotalMilliseconds,0))"
$lines+="has_exited: $($proc.HasExited)"
$lines+="countdown_ms: $CountdownMs"
$lines+="screenshot: $screenshotPath"
$lines+="screenshot_b64: $screenshotBase64Path"
$lines+="stdout_len: $($stdout.Length)"
$lines+="stderr_len: $($stderr.Length)"
$lines+=""
$lines+="=== STDOUT ==="
$lines+=$stdout
$lines+=""
$lines+="=== STDERR ==="
$lines+=$stderr
$lines-join "`n"|Out-File -Encoding utf8 $telemetryFile

$uuidInStdout=$stdout.Contains($testUuid)
$uuidInStderr=$stderr.Contains($testUuid)
$uuidMatch=$uuidInStdout-or$uuidInStderr
$screenshotExists = ($screenshotPath -ne "") -and (Test-Path $screenshotPath)
$end=[datetime]::UtcNow

# ── Write SLO ─────────────────────────────────────────────────────────────────
$slo=@{
  run_id=$runId
  uuid=$testUuid
  uuid_in_stdout=$uuidInStdout
  uuid_in_stderr=$uuidInStderr
  uuid_matched=$uuidMatch
  exit_code=$proc.ExitCode
  has_exited=$proc.HasExited
  countdown_ms=$CountdownMs
  screenshot_path=$screenshotPath
  screenshot_exists=$screenshotExists
  duration_ms=[math]::Round(($end-$start).TotalMilliseconds,0)
  ts_iso=$end.ToString("o")
  signal_path_ok=$uuidMatch
  watchdog_ok=$proc.HasExited
  screenshot_ok=$screenshotExists
}
$slo|ConvertTo-Json|Out-File -Encoding utf8 $sloPath

# ── Results ───────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "=== RESULTS ==="
Write-Host "Exit code:   $($proc.ExitCode)"
Write-Host "UUID match:  $uuidMatch (stdout=$uuidInStdout stderr=$uuidInStderr)"
Write-Host "Screenshot:  $(if($screenshotExists){'captured ('+[math]::Round((Get-Item $screenshotPath).Length/1KB,1)+' KB)'}else{'failed'})"
Write-Host "Exited:      $($proc.HasExited)"
Write-Host ""
if($uuidMatch){Write-Host "SIGNAL PATH PROVEN - UUID found in telemetry"}else{Write-Host "SIGNAL PATH BROKEN - UUID not in telemetry"}
if($proc.HasExited){Write-Host "WATCHDOG OK - process exited"}else{Write-Host "WATCHDOG: process still running after 3 kills"}
if($screenshotExists){Write-Host "SCREENSHOT OK - saved to $screenshotPath"}else{Write-Host "SCREENSHOT MISSING"}
Write-Host ""
Write-Host "Telemetry: $telemetryFile"
Write-Host "SLO:       $sloPath"
Write-Host ""

[pscustomobject]@{
  run_id=$runId
  uuid=$testUuid
  uuid_matched=$uuidMatch
  exit_code=$proc.ExitCode
  has_exited=$proc.HasExited
  screenshot=$screenshotPath
  screenshot_ok=$screenshotExists
  telemetry_file=$telemetryFile
  slo_path=$sloPath
}
