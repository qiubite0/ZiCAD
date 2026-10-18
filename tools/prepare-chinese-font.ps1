param([string]$OutputDir = (Join-Path $env:LOCALAPPDATA 'ZiCAD/font-input'))
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$Font = Join-Path $OutputDir 'LXGWWenKai-Regular.ttf'
$Expected = '2D73EEA94AD808999D90C0000C239A84A33B09D4BC6AB6C5B8BBF1EDA8A7FC8B'
if (-not (Test-Path -LiteralPath $Font) -or (Get-FileHash -LiteralPath $Font).Hash -ne $Expected) {
    Invoke-WebRequest 'https://github.com/lxgw/LxgwWenKai/releases/download/v1.530/LXGWWenKai-Regular.ttf' -OutFile $Font
}
if ((Get-FileHash -LiteralPath $Font).Hash -ne $Expected) { throw 'Font checksum mismatch' }
Invoke-WebRequest 'https://raw.githubusercontent.com/lxgw/LxgwWenKai/v1.530/OFL.txt' -OutFile (Join-Path $OutputDir 'OFL.txt')
$env:ZICAD_CJK_FONT = (Resolve-Path -LiteralPath $Font).Path
Write-Output "ZICAD_CJK_FONT=$env:ZICAD_CJK_FONT"
