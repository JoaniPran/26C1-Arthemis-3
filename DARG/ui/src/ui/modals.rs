use eframe::egui;
use std::fs;
use std::path::PathBuf;
use crate::app::UiState;

pub fn draw_import_modal(ctx: &egui::Context, ui_state: &mut UiState) {
    if !ui_state.show_import_modal {
        return;
    }

    egui::Area::new(egui::Id::new("modal_overlay"))
        .order(egui::Order::PanelResizeLine)
        .fixed_pos(egui::pos2(0.0, 0.0))
        .show(ctx, |ui| {
            let screen_rect = ui.ctx().screen_rect();
            ui.painter().rect_filled(
                screen_rect,
                0.0,
                egui::Color32::from_black_alpha(170), 
            );
            ui.allocate_response(screen_rect.size(), egui::Sense::click());
        });

    let mut pending_path: Option<PathBuf> = None;

    egui::Window::new("importar_modal_window")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .min_width(600.0) 
        .show(ctx, |ui| {
            egui::Frame::none()
                .inner_margin(egui::Margin { left: 0.0, right: 0.0, top: 0.0, bottom: 0.0 })
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        let icon_close = "\u{200B}\u{f00d}\u{200B}"; 
                        let icon_text = egui::RichText::new(icon_close).size(18.0).color(egui::Color32::GRAY);
                        let close_btn = egui::Button::new(icon_text)
                            .frame(false)
                            .rounding(egui::Rounding::same(6.0));
                        
                        if ui.add_sized([32.0, 32.0], close_btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            ui_state.show_import_modal = false;
                        }
                    });
                });

            let is_hovering_file = ctx.input(|i| !i.raw.hovered_files.is_empty());
            
            let (stroke_color, stroke_width, bg_color) = if is_hovering_file {
                (egui::Color32::from_rgb(80, 140, 255), 3.0, egui::Color32::from_rgb(20, 30, 45))
            } else {
                (egui::Color32::from_gray(60), 2.0, egui::Color32::from_rgb(24, 24, 24))
            };
            
            egui::Frame::none()
                .fill(bg_color)
                .stroke(egui::Stroke::new(stroke_width, stroke_color))
                .inner_margin(40.0)
                .outer_margin(egui::Margin { left: 35.0, right: 35.0, top: 0.0, bottom: 35.0 })
                .rounding(12.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        let icon_upload = "\u{200B}\u{f0ee}\u{200B}";
                        ui.label(egui::RichText::new(icon_upload).size(48.0).color(stroke_color));
                        
                        ui.add_space(15.0);
                        
                        ui.label(egui::RichText::new("Arrastra tu archivo YAML aquí").size(20.0).strong().color(egui::Color32::WHITE));
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Solo se admiten extensiones .yaml o .yml").size(14.0).color(egui::Color32::GRAY));
                        
                        ui.add_space(25.0);
                        ui.label(egui::RichText::new("o").size(14.0).color(egui::Color32::DARK_GRAY));
                        ui.add_space(15.0);
                        
                        let btn_text = egui::RichText::new("Explorar archivos del sistema").size(14.0).strong().color(egui::Color32::WHITE);
                        let explore_btn = egui::Button::new(btn_text)
                            .rounding(egui::Rounding::same(6.0))
                            .fill(egui::Color32::from_rgb(45, 90, 180)); 
                        
                        if ui.add_sized([280.0, 38.0], explore_btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("YAML", &["yaml", "yml"])
                                .pick_file() 
                            {
                                pending_path = Some(path);
                            }
                        }

                        if !ui_state.import_message.is_empty() {
                            ui.add_space(20.0);
                            
                            let (icon_msg, color_msg) = if ui_state.import_is_error {
                                ("\u{f057}", egui::Color32::LIGHT_RED) 
                            } else {
                                ("\u{f058}", egui::Color32::from_gray(220))
                            };
                            
                            ui.label(
                                egui::RichText::new(format!("{}  {}", icon_msg, ui_state.import_message))
                                    .color(color_msg)
                                    .size(14.0)
                            );
                        }
                    });
                });

            ctx.input(|i| {
                if !i.raw.dropped_files.is_empty() {
                    if let Some(path) = &i.raw.dropped_files[0].path {
                        pending_path = Some(path.clone());
                    }
                }
            });

            
            if let Some(path) = pending_path {
                let ext = path.extension().unwrap_or_default().to_str().unwrap_or_default().to_lowercase();
                
                if ext == "yaml" || ext == "yml" {
                    let dest_dir = std::path::Path::new("workflows");
                    let _ = fs::create_dir_all(dest_dir);
                    
                    if let Some(file_name) = path.file_name() {
                        let destination = dest_dir.join(file_name);
                        match fs::copy(&path, destination) {
                            Ok(_) => {
                                ui_state.import_message = format!("Archivo '{}' importado exitosamente.", file_name.to_string_lossy());
                                ui_state.import_is_error = false;
                            }
                            Err(e) => {
                                ui_state.import_message = format!("Error interno al guardar: {}", e);
                                ui_state.import_is_error = true;
                            }
                        }
                    }
                } else {
                    ui_state.import_message = "Error: Formato no soportado. Solo se permiten archivos .yaml".to_string();
                    ui_state.import_is_error = true;
                }
            }
        });
}