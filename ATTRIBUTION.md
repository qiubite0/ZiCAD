# Asset attribution

Every non-code asset in this repository (images, icons, fonts, example drawings, presets) is listed
here with its author, source and licence. `cargo xtask assets` (part of `cargo xtask ci`) fails if
an asset file is missing from this table.

**Policy (mandatory):** CADCraft contains **no Autodesk, Adobe or Avid iconography, images,
artwork, fonts, hatch patterns, linetypes, templates or presets.** Every asset is original work by
CADCraft contributors or third-party material under an open licence (OSI open source, public domain
/ CC0, or Creative Commons that allows redistribution). Screenshots of Autodesk software are never
committed. Font files are not added here: shared fonts live in
[storytold/craft-fonts](https://github.com/storytold/craft-fonts), an optional build input. The one
exception is the ArtCraft brand in `docs/brand/`: ArtCraft trademarks, not open source, used under
`docs/brand/LICENSE-brand.txt`.

Generated-in-code assets are original and have no file to list:
- the UI icon set (`crates/ui-egui/src/icons.rs`);
- the "CADCraft Stroke" single-stroke drafting font (`crates/fonts/src/stroke.rs`);
- the standard linetype and hatch-pattern libraries (`crates/doc/src/library.rs`; industry-common
  names, our own dash/spacing values);
- the colour index palette, generated from its structure (`crates/color/src/lib.rs`);
- the sample drawings (`crates/engine/src/sample.rs`).

| Asset | Author | Source | Licence | Notes |
|---|---|---|---|---|
| `assets/app-icon/hicolor/128x128/apps/ai.storyteller.cadcraft.png` | CADCraft contributors | generated from `assets/app-icon/cadcraft-1024.png` | MIT OR Apache-2.0 | Linux hicolor icon |
| `assets/app-icon/hicolor/16x16/apps/ai.storyteller.cadcraft.png` | CADCraft contributors | generated from `assets/app-icon/cadcraft-1024.png` | MIT OR Apache-2.0 | Linux hicolor icon |
| `assets/app-icon/hicolor/24x24/apps/ai.storyteller.cadcraft.png` | CADCraft contributors | generated from `assets/app-icon/cadcraft-1024.png` | MIT OR Apache-2.0 | Linux hicolor icon |
| `assets/app-icon/hicolor/256x256/apps/ai.storyteller.cadcraft.png` | CADCraft contributors | generated from `assets/app-icon/cadcraft-1024.png` | MIT OR Apache-2.0 | Linux hicolor icon |
| `assets/app-icon/hicolor/32x32/apps/ai.storyteller.cadcraft.png` | CADCraft contributors | generated from `assets/app-icon/cadcraft-1024.png` | MIT OR Apache-2.0 | Linux hicolor icon |
| `assets/app-icon/hicolor/48x48/apps/ai.storyteller.cadcraft.png` | CADCraft contributors | generated from `assets/app-icon/cadcraft-1024.png` | MIT OR Apache-2.0 | Linux hicolor icon |
| `assets/app-icon/hicolor/512x512/apps/ai.storyteller.cadcraft.png` | CADCraft contributors | generated from `assets/app-icon/cadcraft-1024.png` | MIT OR Apache-2.0 | Linux hicolor icon |
| `assets/app-icon/hicolor/64x64/apps/ai.storyteller.cadcraft.png` | CADCraft contributors | generated from `assets/app-icon/cadcraft-1024.png` | MIT OR Apache-2.0 | Linux hicolor icon |
| `assets/app-icon/cadcraft.icns` | CADCraft contributors | generated (packaging/icons.sh) | MIT OR Apache-2.0 | macOS icon |
| `assets/app-icon/cadcraft.ico` | CADCraft contributors | generated (packaging/icons.sh) | MIT OR Apache-2.0 | Windows icon |
| `assets/app-icon/cadcraft-64.png` | CADCraft contributors | generated | MIT OR Apache-2.0 | |
| `assets/app-icon/cadcraft-256.png` | CADCraft contributors | generated | MIT OR Apache-2.0 | |
| `assets/app-icon/cadcraft-1024.png` | CADCraft contributors | generated | MIT OR Apache-2.0 | |
| `assets/app-icon/cadcraft-macos-512.png` | CADCraft contributors | generated (macOS margin variant) | MIT OR Apache-2.0 | |
| `packaging/macos/dmg/background.svg` | @XusBadia | original work for this repository (2026-10-08), "CADCraft macOS DMG window background"; derived from the CADCraft app icon (`assets/app-icon/`, referenced via `<image>`, not copied); text is outlined Inter / JetBrains Mono glyph paths (OFL fonts the project already uses); no third-party material modified | MIT OR Apache-2.0 (same as the code: `LICENSE-MIT`, `LICENSE-APACHE`) | Finder window background of the macOS DMG; `packaging/macos/dmg/generate.py` renders it to `background.tiff` (see `packaging/macos/dmg/README.md`) |
| `packaging/macos/dmg/background.tiff` | @XusBadia | Rendered from `packaging/macos/dmg/background.svg` by `packaging/macos/dmg/generate.py` | MIT OR Apache-2.0 (same as the code: `LICENSE-MIT`, `LICENSE-APACHE`) | 1x + 2x HiDPI TIFF for the macOS DMG window; includes the app icon (see its row) and LittleCMS's built-in sRGB profile ("No copyright, use freely") |
| `examples/apartment.dxf` | CADCraft contributors | sample drawing made with cadcraft-cli commands | MIT OR Apache-2.0 | Original |
| `docs/images/ui-apartment.png` | CADCraft contributors | screenshot of CADCraft itself (examples/apartment.dxf) | MIT OR Apache-2.0 | Original; no Autodesk UI |
| `docs/images/ui-layout.png` | CADCraft contributors | screenshot of CADCraft itself (a layout of examples/apartment.dxf) | MIT OR Apache-2.0 | Original; no Autodesk UI |
| `docs/images/ui-bracket.png` | CADCraft contributors | screenshot of CADCraft itself (sample drawing generated in code) | MIT OR Apache-2.0 | Original; no Autodesk UI |
| `docs/brand/artcraft-logo-white.png` | ArtCraft Team | storytold/craftrules `assets/brand/` | ArtCraft brand terms (`docs/brand/LICENSE-brand.txt`) | Trademark, not open source |
| `docs/brand/artcraft-logo-white.svg` | ArtCraft Team | storytold/craftrules `assets/brand/` | ArtCraft brand terms (`docs/brand/LICENSE-brand.txt`) | Trademark, not open source |
| `docs/brand/artcraft-logo.png` | ArtCraft Team | storytold/craftrules `assets/brand/` | ArtCraft brand terms (`docs/brand/LICENSE-brand.txt`) | Trademark, not open source |
| `docs/brand/artcraft-logo.svg` | ArtCraft Team | storytold/craftrules `assets/brand/` | ArtCraft brand terms (`docs/brand/LICENSE-brand.txt`) | Trademark, not open source |
| `docs/brand/artcraft-mark-black.png` | ArtCraft Team | storytold/craftrules `assets/brand/` | ArtCraft brand terms (`docs/brand/LICENSE-brand.txt`) | Trademark, not open source |
| `docs/brand/artcraft-mark-black.svg` | ArtCraft Team | storytold/craftrules `assets/brand/` | ArtCraft brand terms (`docs/brand/LICENSE-brand.txt`) | Trademark, not open source |
| `docs/brand/artcraft-mark.png` | ArtCraft Team | storytold/craftrules `assets/brand/` | ArtCraft brand terms (`docs/brand/LICENSE-brand.txt`) | Trademark, not open source |
| `docs/brand/artcraft-mark.svg` | ArtCraft Team | storytold/craftrules `assets/brand/` | ArtCraft brand terms (`docs/brand/LICENSE-brand.txt`) | Trademark, not open source |

## Interface translations

| Asset | Author | Source | Licence | Notes |
|---|---|---|---|---|
| `crates/ui-egui/src/i18n/uk.tsv` | @dmatviichuk | Original Ukrainian translations of CADCraft English strings | MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`) | No proprietary localization resources copied |
| `crates/ui-egui/src/i18n/de.tsv` | the CADCraft contributors | Original German translations of CADCraft English strings | MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`) | No proprietary localization resources copied |
| `crates/ui-egui/src/i18n/es.tsv` | @PedroGuille | Original Spanish translations of CADCraft English strings | MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`) | No proprietary localization resources copied |
| `crates/ui-egui/src/i18n/pt_br.tsv` | @WladmirJunior | Original Brazilian Portuguese translations of CADCraft English strings | MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`) | No proprietary localization resources copied |
| `crates/ui-egui/src/i18n/cs.tsv` | @Losminek (AI-assisted) | Original Czech translations of CADCraft English strings | MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`) | No proprietary localization resources copied |
| `crates/ui-egui/src/i18n/ru.tsv` | @pechnikov2026-star | Original Russian translations of CADCraft English strings | MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`) | No proprietary localization resources copied |
| `crates/ui-egui/src/i18n/fr.tsv` | @pechnikov2026-star | Original French translations of CADCraft English strings | MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`) | No proprietary localization resources copied |
| `crates/ui-egui/src/i18n/zh.tsv` | @mayloglog | Original Simplified Chinese translations of CADCraft English strings | MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`) | No proprietary localization resources copied |
| `crates/ui-egui/src/i18n/sv.tsv` | Erik Holmbom (@holmbom) with Claude Code (AI-assisted) | Original Swedish translations of CADCraft English strings | MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`) | No proprietary localization resources copied; terminology and the running app reviewed by a native speaker |
| `crates/ui-egui/src/i18n/it.tsv` | Emanuele Bottelli (AI-assisted) | Original Italian translations of CADCraft English strings | MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`) | No proprietary localization resources copied |
| `crates/ui-egui/src/i18n/ko.tsv` | @huk082 (AI-assisted) | Original Korean translations of CADCraft English strings | MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`) | No proprietary localization resources copied; no font assets added |
| `crates/ui-egui/src/i18n/ar.tsv` | @hasan-sdaif | Original Arabic translations of CADCraft English strings | MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`) | No proprietary localization resources copied |

## ZiCAD 二次开发新增资产

| Asset | Author | Source | Licence | Notes |
|---|---|---|---|---|
| `crates/ui-egui/assets/fonts/NotoSansSC-UI.otf` | Noto CJK 字体项目；字体 name 表内嵌版权 Adobe 2014–2021 | 本地已有中文 UI 字体子集，内部 family 为 Noto Sans CJK SC；正式公开前复核原始下载与子集生成记录 | 随附 SIL OFL 1.1 文本，见同目录 LICENSE-OFL.txt | 未声称本字体是 ZiCAD 原创；见 FONT-NOTICE.md |
| `apps/cadcraft-web/dev-host/apartment.dxf` | CADCraft contributors | 复制自 `examples/apartment.dxf` | MIT OR Apache-2.0 | 宿主桥接示例 |
| `apps/cadcraft-web/dev-host/mep-coordination.dxf` | ZiCAD 二次开发贡献者 | `tools/gen_mep_dxf.py` 合成演示图 | MIT OR Apache-2.0 | 非真实客户图纸 |
| `apps/cadcraft-web/dev-host/mep-coordination-en.dxf` | ZiCAD 二次开发贡献者 | `tools/gen_mep_dxf.py en` 合成演示图 | MIT OR Apache-2.0 | 英文演示 |
| `docs/images/zicad-help-menu.png` | ZiCAD 二次开发贡献者 | 本地 ZiCAD 页面与合成演示图截图 | MIT OR Apache-2.0 | 不含个人 Key；AI/导出截图在菜单品牌统一前录制 |
| `docs/images/zicad-ai-chat.png` | ZiCAD 二次开发贡献者 | 本地 ZiCAD 页面与合成演示图截图 | MIT OR Apache-2.0 | 不含个人 Key；AI/导出截图在菜单品牌统一前录制 |
| `docs/images/zicad-dwg-export.png` | ZiCAD 二次开发贡献者 | 本地 ZiCAD 页面与合成演示图截图 | MIT OR Apache-2.0 | 不含个人 Key；AI/导出截图在菜单品牌统一前录制 |
| `docs/images/zicad-batch-print.png` | ZiCAD 二次开发贡献者 | 本地批量打印面板与合成演示图截图 | MIT OR Apache-2.0 | 已验证单图框 PDF 下载；不含私人图纸或凭据 |

| `docs/images/zicad-wordmark.png` | ZiCAD 二次开发贡献者（AI 辅助设计） | 用户选定的第 9 款字标 | MIT OR Apache-2.0 | ZiCAD 品牌字标 |
| `docs/images/zicad-cleanroom-live.png` | ZiCAD 二次开发贡献者 | 本地网页实际截图 | MIT OR Apache-2.0 | 使用合成洁净室演示图 |
| `examples/cleanroom-demo.dxf` | ZiCAD 二次开发贡献者 | `tools/generate_cleanroom_demo.py` | MIT OR Apache-2.0 | 原创示意图，英文标注 |
| `examples/commercial-complex-demo.dxf`、`apps/cadcraft-web/dev-host/commercial-complex-demo.dxf` | ZiCAD 二次开发贡献者 | `tools/generate_commercial_demo.py` 原创生成 | MIT OR Apache-2.0 | 商业综合体合成底图 |
| `docs/images/zicad-commercial-live.png` | ZiCAD 二次开发贡献者 | 本地网页实际截图 | MIT OR Apache-2.0 | 商业综合体演示 |
| `examples/cleanroom-pro-demo.dxf`、`apps/cadcraft-web/dev-host/cleanroom-pro-demo.dxf` | ZiCAD 二次开发贡献者 | `tools/generate_cleanroom_pro_demo.py` 原创生成 | MIT OR Apache-2.0 | 洁净室工艺与机电协调示意 |
| `docs/images/zicad-cleanroom-pro-live.png` | ZiCAD 二次开发贡献者 | 本地 ZiCAD 网页实际截图 | MIT OR Apache-2.0 | 使用原创合成演示图 |
| `docs/images/zicad-cleanroom-ai-live.png`、`zicad-cleanroom-pdf-live.png`、`zicad-cleanroom-dwg-live.png`、`zicad-cleanroom-export-result.png` | ZiCAD 二次开发贡献者 | 本地网页实际 AI 与出图操作截图 | MIT OR Apache-2.0 | 使用原创合成图纸 |
| `docs/demo/cleanroom-pro-demo.dwg` | ZiCAD 二次开发贡献者 | 本地转换服务由洁净室 DXF 导出 | MIT OR Apache-2.0 | 2018 版，600 实体读回验证 |
| `docs/images/zicad-model-settings-live.png` | ZiCAD 二次开发贡献者 | 本地接口连接测试实际截图 | MIT OR Apache-2.0 | 密钥已遮蔽 |
| `docs/images/zicad-text-detail-live.png` | ZiCAD 二次开发贡献者 | 本地洁净室图纸文字细节实际截图 | MIT OR Apache-2.0 | 原创合成图纸 |
| `docs/images/zicad-overseas-presets-live.png` | ZiCAD 二次开发贡献者 | 本地海外模型预设菜单实际截图 | MIT OR Apache-2.0 | 不含 API Key |
| `docs/images/zicad-hero.png` | ZiCAD 二次开发贡献者（AI 辅助设计） | 基于用户选定字标与封面风格生成 | MIT OR Apache-2.0 | 工业洁净室概念场景封面 |
