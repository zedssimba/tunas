# S.A.L.T CI downloads: start or refresh the web server, then check it answers.
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
docker compose up -d
Start-Sleep -Seconds 3
try {
  $r = Invoke-WebRequest -UseBasicParsing http://127.0.0.1:2065/tunas/latest.json
  Write-Host "salt-get is up. TUNAS latest.json:" -ForegroundColor Green
  Write-Host $r.Content
} catch {
  Write-Host "salt-get did not answer on port 2065: $_" -ForegroundColor Red
}
