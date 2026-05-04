@echo off
set TAURI_CDP_PORT=19227
set TAURI_BUILD_NUMBER=15
start /B host-tauri.exe
timeout /T 8 > nul
curl.exe -s http://127.0.0.1:19227/json/version
type C:\Users\wendy\AppData\Local\Temp\ht-cdp-err.txt 2>nul
