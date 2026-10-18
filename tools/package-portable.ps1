<# Build a Windows x64 portable ZIP without copying user configuration. #>
param(
    [string]$BuildRoot = (Split-Path -Parent $PSScriptRoot),
    [string]$OutputRoot = (Join-Path (Split-Path -Parent $PSScriptRoot) 'dist/portable'),
    [string]$RuntimeDir,
    [switch]$SkipBuild
)
$ErrorActionPreference = 'Stop'
$SourceRoot = Split-Path -Parent $PSScriptRoot
if (-not $SkipBuild) {
    Push-Location $BuildRoot
    try {
        & cargo build --release -p cadcraft -p cadcraft-cli -j 2
        if ($LASTEXITCODE -ne 0) { throw 'Release build failed' }
    } finally { Pop-Location }
}
$Bin = Join-Path $BuildRoot 'target/release'
foreach ($Exe in 'cadcraft.exe','cadcraft-cli.exe') {
    if (-not (Test-Path -LiteralPath (Join-Path $Bin $Exe))) { throw "Missing $Exe" }
}
$Name = 'ZiCAD-0.7.0-preview-windows-x64-portable'
$Stage = Join-Path $OutputRoot $Name
if (Test-Path -LiteralPath $Stage) { throw "Output already exists: $Stage" }
New-Item -ItemType Directory -Path $Stage -Force | Out-Null
Copy-Item -LiteralPath (Join-Path $Bin 'cadcraft.exe') -Destination (Join-Path $Stage 'ZiCAD.exe')
Copy-Item -LiteralPath (Join-Path $Bin 'cadcraft-cli.exe') -Destination (Join-Path $Stage 'ZiCAD-cli.exe')
foreach ($NameOfFile in 'LICENSE-MIT','LICENSE-APACHE','NOTICE','ATTRIBUTION.md') {
    Copy-Item -LiteralPath (Join-Path $SourceRoot $NameOfFile) -Destination $Stage
}
New-Item -ItemType Directory -Path (Join-Path $Stage 'examples'),(Join-Path $Stage 'licenses') | Out-Null
foreach ($Drawing in 'apartment-120-furnished.dxf','cleanroom-pro-demo.dxf','commercial-complex-demo.dxf') {
    Copy-Item -LiteralPath (Join-Path $SourceRoot "examples/$Drawing") -Destination (Join-Path $Stage 'examples')
}
Copy-Item -LiteralPath (Join-Path $SourceRoot 'crates/ui-egui/assets/fonts/LICENSE-OFL.txt') -Destination (Join-Path $Stage 'licenses/NotoSansSC-OFL.txt')
Copy-Item -LiteralPath (Join-Path $SourceRoot 'crates/ui-egui/assets/fonts/FONT-NOTICE.md') -Destination (Join-Path $Stage 'licenses')
if ($RuntimeDir) {
    Get-ChildItem -LiteralPath $RuntimeDir -Filter '*.dll' -File | Copy-Item -Destination $Stage
    @'
Microsoft Visual C++ Runtime (x64)
Copyright Microsoft Corporation. All rights reserved.
These app-local runtime files are copied from the installed Visual Studio VC Redist directory.
They are not covered by ZiCAD's MIT/Apache-2.0 licenses.
https://learn.microsoft.com/cpp/windows/redistributing-visual-cpp-files
'@ | Set-Content -LiteralPath (Join-Path $Stage 'licenses/Microsoft-VC-Runtime.txt') -Encoding utf8
}
@'
ZiCAD 0.7.0 Preview - Windows x64 免安装版

1. 将 ZIP 完整解压到可写目录，双击 ZiCAD.exe。
2. 文件 -> 打开，选择 examples 中的 DXF；也可将 DXF/DWG 拖到程序。
3. AI 面板 -> Settings，填写自己的接口端点、模型和 Key。
   DeepSeek / OpenAI 可直连；Claude / Gemini / Grok 预设使用 OpenRouter Key。
4. 批量 PDF：AI 面板 -> 批量打印，识别并复核图框后导出。
5. DWG：文件 -> 导出 DWG；桌面版直接保存，无需网页服务或 Python。
6. ZiCAD-cli.exe 为命令行与 MCP 程序，可使用 --help 查看参数。

配置及日志默认位于 %APPDATA%\ZiCAD；免安装不表示配置随程序目录移动。
程序内未附带任何私人 Key、聊天记录或浏览器配置。
图纸中文显示及复杂行业对象仍需兼容性验证；示例属于方案演示。
这是未签名预览包。Windows 10/11 x64，需要支持程序图形后端的显卡驱动。

项目：https://github.com/qiubite0/ZiCAD
联系：zhichengbian7@gmail.com
源码与开源许可：见项目主页、LICENSE-MIT、LICENSE-APACHE、NOTICE。
'@ | Set-Content -LiteralPath (Join-Path $Stage '使用说明.txt') -Encoding utf8
$Manifest = Get-ChildItem -LiteralPath $Stage -Recurse -File | ForEach-Object {
    [pscustomobject]@{
        Path = [IO.Path]::GetRelativePath($Stage,$_.FullName)
        Bytes = $_.Length
        SHA256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash
    }
}
$Manifest | Export-Csv -LiteralPath (Join-Path $Stage 'MANIFEST.csv') -NoTypeInformation -Encoding utf8
$Zip = Join-Path $OutputRoot "$Name.zip"
Compress-Archive -LiteralPath $Stage -DestinationPath $Zip -CompressionLevel Optimal
Get-FileHash -LiteralPath $Zip -Algorithm SHA256 | Format-List
Get-Item -LiteralPath $Zip | Select-Object FullName,Length
