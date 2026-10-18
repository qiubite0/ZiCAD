//! ZiCAD creator identity and project information.
use crate::CadApp;
use egui::{Rect, RichText, Sense, pos2, vec2};
pub const PROJECT_URL: &str = "https://github.com/qiubite0/ZiCAD";
pub const EMAIL: &str = "zhichengbian7@gmail.com";

pub fn avatar(ui: &mut egui::Ui, size: f32) {
    let id = egui::Id::new("zicad_creator_avatar");
    let texture = ui.ctx().data_mut(|d| d.get_temp::<egui::TextureHandle>(id));
    let texture = texture.or_else(|| {
        let image = image::load_from_memory(include_bytes!("../assets/brand/creator-panda.png")).ok()?.to_rgba8();
        let color = egui::ColorImage::from_rgba_unmultiplied([image.width() as usize, image.height() as usize], image.as_raw());
        let handle = ui.ctx().load_texture("zicad-creator", color, egui::TextureOptions::LINEAR);
        ui.ctx().data_mut(|d| d.insert_temp(id, handle.clone()));
        Some(handle)
    });
    if let Some(texture) = texture {
        ui.image((texture.id(), vec2(size, size)));
    }
}

pub fn logo(ui: &mut egui::Ui, size: f32) {
    let id = egui::Id::new("zicad_about_logo");
    let texture = ui.ctx().data_mut(|d| d.get_temp::<egui::TextureHandle>(id)).or_else(|| {
        let image = image::load_from_memory(include_bytes!("../../../assets/app-icon/cadcraft-256.png")).ok()?.to_rgba8();
        let pixels = egui::ColorImage::from_rgba_unmultiplied([image.width() as usize, image.height() as usize], image.as_raw());
        let texture = ui.ctx().load_texture("zicad-about-logo", pixels, egui::TextureOptions::LINEAR);
        ui.ctx().data_mut(|d| d.insert_temp(id, texture.clone()));
        Some(texture)
    });
    if let Some(texture) = texture {
        ui.image((texture.id(), vec2(size, size)));
    }
}

pub fn titlebar(app: &mut CadApp, ui: &mut egui::Ui, r: Rect, height: f32) {
    let mac = ui.ctx().os().is_mac();
    let controls = if mac { 0.0 } else { 132.0 };
    let drag_rect = Rect::from_min_size(r.min, vec2((r.width() - controls - 180.0).max(80.0), height));
    let drag = ui.interact(drag_rect, ui.id().with("zicad_window_drag"), Sense::CLICK | Sense::DRAG);
    if drag.drag_started() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    if drag.double_clicked() {
        let maximized = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
    }
    let left = r.left() + if mac { 84.0 } else { 4.0 };
    let icon_id = egui::Id::new("zicad_title_icon");
    let icon = ui.ctx().data_mut(|d| d.get_temp::<egui::TextureHandle>(icon_id)).or_else(|| {
        let image = image::load_from_memory(include_bytes!("../../../assets/app-icon/cadcraft-256.png")).ok()?.to_rgba8();
        let pixels = egui::ColorImage::from_rgba_unmultiplied([image.width() as usize, image.height() as usize], image.as_raw());
        let texture = ui.ctx().load_texture("zicad-title-icon", pixels, egui::TextureOptions::LINEAR);
        ui.ctx().data_mut(|d| d.insert_temp(icon_id, texture.clone()));
        Some(texture)
    });
    if let Some(icon) = icon {
        let icon_size = if mac { 24.0 } else { r.height().min(64.0) };
        let icon_top = if mac { r.top() + (height - icon_size) / 2.0 } else { r.top() + (r.height() - icon_size) / 2.0 };
        let rect = Rect::from_min_size(pos2(left, icon_top), vec2(icon_size, icon_size));
        ui.painter().image(icon.id(), rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), egui::Color32::WHITE);
    }
    ui.painter().text(
        pos2(r.center().x, r.top() + height / 2.0),
        egui::Align2::CENTER_CENTER,
        "ZiCAD",
        egui::FontId::proportional(14.0),
        crate::theme::Tokens::get().text,
    );
    let identity_width = 180.0;
    let identity = Rect::from_min_size(pos2((r.right() - controls - identity_width).max(left + 125.0), r.top()), vec2(identity_width, height));
    ui.scope_builder(egui::UiBuilder::new().max_rect(identity), |ui| {
        ui.horizontal_centered(|ui| {
            avatar(ui, 22.0);
            if ui.link("科技熊 · 创作者").on_hover_text("关于作者与 ZiCAD").clicked() {
                app.start("ui.dialog.about");
            }
        });
    });
    if !mac {
        for (index, text) in ["−", "□", "×"].iter().enumerate() {
            let rect = Rect::from_min_size(pos2(r.right() - controls + index as f32 * 44.0, r.top()), vec2(44.0, height));
            ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                if ui.add_sized(rect.size(), egui::Button::new(*text).frame(false)).clicked() {
                    match index {
                        0 => ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true)),
                        1 => {
                            let max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
                        }
                        _ => app.start("ui.quit"),
                    }
                }
            });
        }
    }
}

pub fn about(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        logo(ui, 76.0);
        ui.vertical(|ui| {
            ui.heading("ZiCAD");
            ui.label("面向 Agent 工作流的开源二维 CAD");
            ui.label(RichText::new("二次开发作者：AI 科技熊 · zhichengbian7").strong());
            ui.label(format!("版本 {} · Windows / Web", env!("CARGO_PKG_VERSION")));
        });
    });
    ui.add_space(10.0);
    ui.label("ZiCAD 围绕建筑、机电与日常工程改图，将二维绘图、AI 辅助、MCP 自动化和图纸交付组织为统一工作流。支持本地桌面运行，也提供浏览器入口。");
    ui.add_space(8.0);
    ui.label(RichText::new("主要功能").strong());
    for line in [
        "AI 辅助：配置大模型接口，通过对话与图片附件辅助绘图，执行结构化命令并反馈结果。",
        "MCP 与自动化：通过外部 Agent、CLI 和控制接口查询图纸、执行命令与导出。",
        "绘图与改图：图层、图块、标注和属性编辑，支持 CHPROP、SETBYLAYER 与撤销。",
        "工程交付：图框识别、批量 PDF、图纸目录；桌面版支持 DWG 打开、编辑与保存。",
        "中文体验：中文界面与工程文字缺字回退，内置建筑、住宅及工业案例。",
        "性能与文件优化：持续优化 CAD 图纸处理速度与保存文件大小，关注大图响应、重复对象清理和冗余数据整理。",
    ] {
        ui.label(format!("• {line}"));
    }
    ui.add_space(8.0);
    ui.label("公众号：BIM小站　｜　视频号：AI科技熊");
    ui.hyperlink_to("开源项目：github.com/qiubite0/ZiCAD", PROJECT_URL);
    ui.hyperlink_to(format!("Email:{EMAIL}"), format!("mailto:{EMAIL}"));
    ui.label("欢迎提交问题、使用反馈与改进建议，共同完善工程绘图工作流。");
    ui.separator();
    ui.collapsing("开源来源与使用说明", |ui| {
        ui.hyperlink_to("基于 CADCraft 二次开发", "https://github.com/storytold/cadcraft");
        ui.label("Rust / egui / WebAssembly · MIT OR Apache-2.0。原项目及依赖版权见随附 LICENSE、NOTICE 与 ATTRIBUTION。");
        ui.label("复杂 DWG、行业代理对象及特殊字体仍需结合实际图纸验证；修改重要图纸时建议另存副本。AI 接口由用户配置。");
    });
}

pub fn creator_footer(ui: &mut egui::Ui) {
    ui.add_space(8.0);
    ui.vertical_centered(|ui| {
        avatar(ui, 48.0);
        ui.label(RichText::new("AI 科技熊").small());
    });
}
