$scriptPath = Join-Path $PSScriptRoot "docserve-live.pwsh"
$scriptBody = Get-Content -LiteralPath $scriptPath -Raw
$scriptBlock = [ScriptBlock]::Create($scriptBody)
& $scriptBlock @args
