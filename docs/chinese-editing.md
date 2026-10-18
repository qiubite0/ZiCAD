# 中文图纸与属性编辑

构建前运行 `./tools/prepare-chinese-font.ps1`，在当前 PowerShell 会话设置 ZICAD_CJK_FONT 后再 cargo build。脚本从作者仓库下载固定版本 v1.530，并验证 SHA256。字体保存在仓库外，不提交字体二进制。

完整 LXGW WenKai Regular 作为 TEXT、MTEXT、标注等共用字体引擎的缺字回退，覆盖简繁常用工程文字；不是所有 Unicode 字符的覆盖承诺。原文字样式不变，可用字体优先，缺字才回退。PDF、PNG 与桌面及 WASM 绘图复用该字体引擎。分发嵌入字体的程序时必须附带原 OFL.txt。

- CHPROP：选择对象后，输入 Color / Layer / Linetype / Lineweight / Ltscale / Transparency，再输入值；可重复修改，回车结束。JSON 参数与 properties.set 相同。
- SETBYLAYER：选中对象，将颜色、线型、线宽、透明度恢复为 ByLayer。保留所在图层和线型比例；不深入修改块定义，可撤销。

这两个命令位于修改菜单，CLI、AI 和 MCP 可调用相同命令核心。
