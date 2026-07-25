use crate::app::{CoreState, UiState};
use crate::ui::themes::Theme;
use crate::utils::upload_workflow_file;
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

pub fn draw_import_modal(ctx: &egui::Context, ui_state: &mut UiState) {
    if !ui_state.show_import_modal {
        return;
    }

    if let Some(rx) = &ui_state.upload_rx {
        if let Ok(result) = rx.try_recv() {
            match result {
                Ok(file_name) => {
                    ui_state.expected_file = file_name.clone();
                    ui_state.import_message = "Validando YAML...".to_string();
                    ui_state.import_is_error = false;
                }
                Err(error_msg) => {
                    ui_state.is_importing = false;
                    ui_state.import_message = error_msg;
                    ui_state.import_is_error = true;
                }
            }
            ui_state.upload_rx = None;
        }
    }

    egui::Area::new(egui::Id::new("modal_overlay"))
        .order(egui::Order::PanelResizeLine)
        .fixed_pos(egui::pos2(0.0, 0.0))
        .show(ctx, |ui| {
            let screen_rect = ui.ctx().screen_rect();
            ui.painter()
                .rect_filled(screen_rect, 0.0, Theme::OVERLAY_BG);
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
                .inner_margin(egui::Margin {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0,
                })
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        let icon_close = "\u{200B}\u{f00d}\u{200B}";
                        let icon_text = egui::RichText::new(icon_close)
                            .size(18.0)
                            .color(Theme::TEXT_MUTED);
                        let close_btn = egui::Button::new(icon_text)
                            .frame(false)
                            .rounding(egui::Rounding::same(6.0));

                        if ui
                            .add_sized([32.0, 32.0], close_btn)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            ui_state.show_import_modal = false;
                        }
                    });
                });

            let is_hovering_file = ctx.input(|i| !i.raw.hovered_files.is_empty());

            let (stroke_color, stroke_width, bg_color) = if is_hovering_file {
                (Theme::DROPZONE_HOVER_STROKE, 3.0, Theme::DROPZONE_HOVER_BG)
            } else {
                (Theme::DROPZONE_IDLE_STROKE, 2.0, Theme::DROPZONE_IDLE_BG)
            };

            egui::Frame::none()
                .fill(bg_color)
                .stroke(egui::Stroke::new(stroke_width, stroke_color))
                .inner_margin(40.0)
                .outer_margin(egui::Margin {
                    left: 35.0,
                    right: 35.0,
                    top: 0.0,
                    bottom: 35.0,
                })
                .rounding(12.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        let icon_upload = "\u{200B}\u{f0ee}\u{200B}";
                        ui.label(
                            egui::RichText::new(icon_upload)
                                .size(48.0)
                                .color(stroke_color),
                        );

                        ui.add_space(15.0);

                        ui.label(
                            egui::RichText::new("Arrastra tu archivo YAML aquí")
                                .size(16.0)
                                .strong()
                                .color(Theme::TEXT_WHITE),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("Solo se admiten extensiones .yaml o .yml")
                                .size(14.0)
                                .color(Theme::TEXT_MUTED),
                        );

                        ui.add_space(25.0);
                        ui.label(
                            egui::RichText::new("o")
                                .size(14.0)
                                .color(Theme::TEXT_DARK_GRAY),
                        );
                        ui.add_space(15.0);

                        let btn_text = egui::RichText::new("Explorar archivos del sistema")
                            .size(16.0)
                            .strong()
                            .color(Theme::TEXT_WHITE);

                        let explorer_btn = egui::Button::new(btn_text)
                            .rounding(egui::Rounding::same(6.0))
                            .fill(Theme::ACCENT_BLUE);

                        let btn_response = ui
                            .add_enabled_ui(!ui_state.is_importing, |ui| {
                                ui.add_sized([300.0, 40.0], explorer_btn)
                            })
                            .inner;

                        if btn_response
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                            && let Some(path) = rfd::FileDialog::new()
                                .add_filter("YAML", &["yaml", "yml"])
                                .pick_file()
                        {
                            pending_path = Some(path);
                        }

                        if ui_state.is_importing {
                            ui.add_space(20.0);
                            ui.horizontal_centered(|ui| {
                                let text_w = 90.0;
                                let spinner_w = 20.0;
                                ui.add_space(
                                    (ui.available_width() / 2.0) - (text_w / 2.0) - spinner_w,
                                );
                                ui.add(egui::Spinner::new().size(14.0).color(Theme::TEXT_MUTED));
                                ui.add_space(1.0);
                                ui.label(
                                    egui::RichText::new(&ui_state.import_message)
                                        .color(Theme::TEXT_LIGHT_GRAY)
                                        .size(14.0),
                                );
                            });
                            ctx.request_repaint();
                        } else if !ui_state.import_message.is_empty() {
                            ui.add_space(20.0);
                            let (icon_msg, color_msg) = if ui_state.import_is_error {
                                ("\u{f057}", Theme::STATUS_FAILED)
                            } else {
                                ("\u{f058}", Theme::TEXT_LIGHT_GRAY)
                            };
                            ui.label(
                                egui::RichText::new(format!(
                                    "{}  {}",
                                    icon_msg, ui_state.import_message
                                ))
                                .color(color_msg)
                                .size(14.0),
                            );
                        }
                    });
                });

            ctx.input(|i| {
                if !i.raw.dropped_files.is_empty()
                    && let Some(path) = &i.raw.dropped_files[0].path
                {
                    pending_path = Some(path.clone());
                }
            });

            if let Some(path) = pending_path {
                if let Some(username) = ui_state.session_username.clone() {
                    let coordinator_ip = ui_state.coordinator_ip.clone();

                    ui_state.is_importing = true;
                    ui_state.import_message = "Guardando en el servidor...".to_string();
                    ui_state.import_is_error = false;

                    let (tx, rx) = mpsc::channel();
                    ui_state.upload_rx = Some(rx);

                    thread::spawn(move || {
                        let result = upload_workflow_file(&path, &username, &coordinator_ip);
                        let _ = tx.send(result);
                    });
                } else {
                    ui_state.import_message = "Error: Sesión no encontrada.".to_string();
                    ui_state.import_is_error = true;
                }
            }
        });
}
