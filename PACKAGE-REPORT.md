# ZiCAD 源码上传候选包核对报告

- 源码及资料文件：720 个；未压缩大小：12,657,545 字节。
- 每个文件的 SHA-256 见 UPLOAD-MANIFEST.csv。
- 不含 .git 历史、target、dist、浏览器配置、个人测试凭据脚本、日志和缓存。
- 已排除上游专有商标图片和未经调整的自动工作流；原版权/许可证文本保留。
- 强凭据模式扫描：本次未检出匹配；此结论不是无敏感信息保证，上传前仍需人工审查。
- 素材表检查：所选资产均有 ATTRIBUTION.md 路径记录。
- 本包包含当前工作区的未提交新增文件，不只是 git diff；没有提交、推送或上传到外部平台。

## 验证结果

- Rust AI/打印/DWG 专项测试 13 项通过。
- 整理进仓库的 Python HTTP 与真实 DWG 转换测试 3 项通过。
- WASM 网页构建通过。
- 单图框真实网页 PDF 下载通过；多图框/复杂图签与独立 CAD 兼容性仍按发布检查复核。

## 尚需发布者确认

1. 中文 UI 字体的原始来源和子集生成记录需要进一步复核，详见 docs/发布检查.md 和字体 FONT-NOTICE.md。本包是上传候选资料，不是完成全部素材审查的正式发布包。
2. 桌面图标和 DMG 等原有素材需在分发桌面产品前做 ZiCAD 品牌审核。
3. 公网登录、权限、限流、文件隔离和密钥管理尚未建设；开发服务不应原样开放为匿名生产 API。
4. 未运行全仓库完整 CI；只报告上列专项检查。

## 如何上传

先解压源码 ZIP，把 ZiCAD 目录内的文件作为仓库根目录上传；不要仅把 ZIP 文件上传为源码仓库内容。ZIP 可以另作为 Release 附件。首次上传请按 README 构建，按 docs/源码与文件上传清单.md 复核。

## 显式排除的已跟踪/新增文件

- `.github/workflows/freebsd.yml`：继承的自动工作流，需自有仓库审核后启用
- `.github/workflows/nix.yml`：继承的自动工作流，需自有仓库审核后启用
- `.github/workflows/packaging-lint.yml`：继承的自动工作流，需自有仓库审核后启用
- `.github/workflows/release.yml`：继承的自动工作流，需自有仓库审核后启用
- `.github/workflows/windows-arm64.yml`：继承的自动工作流，需自有仓库审核后启用
- `AGENTS.md`：开发助手说明，不属于运行源码
- `CLAUDE.md`：开发助手说明，不属于运行源码
- `docs/brand/artcraft-logo-white.png`：上游专有商标图案
- `docs/brand/artcraft-logo-white.svg`：上游专有商标图案
- `docs/brand/artcraft-logo.png`：上游专有商标图案
- `docs/brand/artcraft-logo.svg`：上游专有商标图案
- `docs/brand/artcraft-mark-black.png`：上游专有商标图案
- `docs/brand/artcraft-mark-black.svg`：上游专有商标图案
- `docs/brand/artcraft-mark.png`：上游专有商标图案
- `docs/brand/artcraft-mark.svg`：上游专有商标图案
