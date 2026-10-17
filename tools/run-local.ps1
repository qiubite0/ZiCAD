$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $projectRoot
if (-not (Test-Path -LiteralPath 'target/debug/cadcraft-cli.exe')) {
    cargo build -p cadcraft-cli -j 2
    if ($LASTEXITCODE -ne 0) { throw 'Converter build failed' }
}
if (-not (Test-Path -LiteralPath 'dist/web/index.html')) {
    throw 'Build the web app first: cd apps/cadcraft-web; trunk build'
}
python tools/dev-proxy.py
