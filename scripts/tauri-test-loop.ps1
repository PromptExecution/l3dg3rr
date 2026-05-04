param([int]$CountdownMs=8000,[int]$WaitSeconds=15,[int]$LoopCount=1)

$ErrorActionPreference="Stop"
$buildFile="D:\Projects\l3dg3rr\.build_counter"
$harness="D:\Projects\l3dg3rr\scripts\tauri-test-harness.ps1"

for ($i=0; $i -lt $LoopCount; $i++) {
  $buildNum = if (Test-Path $buildFile) { [int](Get-Content $buildFile -Raw) } else { 0 }
  $buildNum++
  $buildNum | Out-File -Encoding ascii $buildFile

  Write-Host "[$($i+1)/$LoopCount] Build #$buildNum"

  $env:PATH="C:\Users\wendy\.cargo\bin;C:\msys64\mingw64\bin;"+$env:PATH
  $env:TAURI_BUILD_NUMBER="$buildNum"
  Set-Location "D:\Projects\l3dg3rr"
  cmd.exe /c "C:\Users\wendy\.cargo\bin\cargo.exe build -p ledgerr-host --bin host-tauri 2>nul"
  $exitCode = $LASTEXITCODE
  if ($exitCode -ne 0 -and $exitCode -ne $null) { Write-Host "  BUILD FAILED (exit $exitCode)" } else { Write-Host "  build OK" }

  & $harness -CountdownMs $CountdownMs -WaitSeconds $WaitSeconds 2>&1 | Out-Null

  $domFile = "$env:TEMP\host-tauri-dom-dump.txt"
  if (Test-Path $domFile) {
    $dom = Get-Content $domFile -Raw
    if ($dom -match "b$buildNum") {
      Write-Host "  DOM build #$buildNum OK"
    } else {
      Write-Host "  WARNING: build #$buildNum not in DOM"
    }
    Copy-Item $domFile "D:\Projects\l3dg3rr\.b00t\scratch\dom-dump-b$buildNum.txt" -Force
  } else {
    Write-Host "  WARNING: no DOM dump"
  }

  $slo = Get-ChildItem "$env:TEMP\host-tauri-slo-*.json" | Sort-Object LastWriteTime -Descending | Select-Object -First 1
  if ($slo) {
    $data = Get-Content $slo.FullName -Raw | ConvertFrom-Json
    Write-Host "  SLO: signal=$($data.signal_path_ok) watchdog=$($data.watchdog_ok) screenshot=$($data.screenshot_ok)"
  }
  Write-Host ""
}
Write-Host "=== $LoopCount iterations done ==="
