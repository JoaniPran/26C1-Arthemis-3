use crate::app::{AppView, CoreState, UiState};
use crate::ui::themes::Theme;
use coordinator::db::Database;
use eframe::egui;
use std::fs;

pub fn draw(
    ctx: &egui::Context,
    ui_state: &mut UiState,
    core_state: &mut CoreState,
    db: Option<&Database>,
) {
    if ui_state.current_view == AppView::Workflows {
        let frame = egui::Frame::none()
            .fill(Theme::BG_SIDEBAR)
            .inner_margin(egui::Margin::same(0.0));

        egui::SidePanel::left("content_panel")
            .resizable(true)
            .default_width(280.0)
            .width_range(220.0..=300.0)
            .frame(frame)
            .show(ctx, |ui| {
                egui::Frame::none().inner_margin(15.0).show(ui, |ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), 28.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            let title_text = egui::RichText::new("WORKFLOWS")
                                .size(12.0)
                                .strong()
                                .color(Theme::TEXT_DARK_GRAY);

                            ui.label(title_text);

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let icon_plus = "\u{200B}\u{f067}\u{200B}";
                                    let btn_plus = egui::Button::new(
                                        egui::RichText::new(icon_plus).size(16.0),
                                    )
                                    .frame(false)
                                    .rounding(egui::Rounding::same(6.0));

                                    if ui
                                        .add_sized([28.0, 28.0], btn_plus)
                                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                                        .clicked()
                                    {
                                        ui_state.show_import_modal = true;
                                        ui_state.import_message.clear();
                                        ui_state.import_is_error = false;
                                    }
                                },
                            );
                        },
                    );

                    ui.add_space(5.0);
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 1.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().rect_filled(rect, 0.0, Theme::BORDER_DARK);
                    ui.add_space(10.0);

                    ui.spacing_mut().item_spacing.y = 2.0;

                    let mut delete_workflow: Option<String> = None;
                    for (file_name, _) in &core_state.workflows {
                        let is_selected = core_state.selected_workflow.as_ref() == Some(file_name);

                        let height = 28.0;
                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), height),
                            egui::Sense::click(),
                        );

                        let bg_color = if is_selected {
                            Theme::SELECTION_BG
                        } else if response.hovered() {
                            Theme::HOVER_FILE
                        } else {
                            egui::Color32::TRANSPARENT
                        };

                        let icon_color = if is_selected {
                            egui::Color32::LIGHT_BLUE
                        } else {
                            Theme::TEXT_DARK_GRAY
                        };
                        let text_color = if is_selected {
                            Theme::TEXT_WHITE
                        } else {
                            Theme::TEXT_LIGHT_GRAY
                        };

                        if bg_color != egui::Color32::TRANSPARENT {
                            ui.painter().rect_filled(rect, 6.0, bg_color);
                        }

                        let mut job = egui::text::LayoutJob::default();
                        job.append(
                            "\u{200B}\u{f15b}\u{200B}",
                            0.0,
                            egui::text::TextFormat {
                                font_id: egui::FontId::proportional(14.0),
                                color: icon_color,
                                ..Default::default()
                            },
                        );
                        job.append(
                            &format!("  {}", file_name),
                            0.0,
                            egui::text::TextFormat {
                                font_id: egui::FontId::proportional(14.0),
                                color: text_color,
                                ..Default::default()
                            },
                        );

                        let galley = ctx.fonts(|f| f.layout_job(job));
                        let text_pos = rect.left_center()
                            - egui::vec2(0.0, galley.rect.height() / 2.0)
                            + egui::vec2(10.0, 0.0);

                        ui.painter().galley(text_pos, galley, Theme::TEXT_WHITE);

                        if response.clicked() {
                            core_state.selected_workflow = Some(file_name.clone());
                        }

                        response.context_menu(|ui| {
                            if ui.button("Eliminar").clicked() {
                                delete_workflow = Some(file_name.clone());
                                ui.close_menu();
                            }
                        });

                        response.on_hover_cursor(egui::CursorIcon::PointingHand);
                    }

                    if let Some(file_name) = delete_workflow {
                        let workflow_path = std::path::Path::new("workflows").join(&file_name);
                        let _ = fs::remove_file(&workflow_path);

                        if let Some(db) = db
                            && let Ok(workflow_id) = db.get_workflow_id(&file_name)
                        {
                            let _ = db.delete_workflows(workflow_id);
                        }

                        core_state.workflows.retain(|(f, _)| f != &file_name);
                        if core_state.selected_workflow.as_ref() == Some(&file_name) {
                            core_state.selected_workflow = None;
                            core_state.loaded_workflow = None;
                            core_state.current_tasks.clear();
                        }
                    }
                });

                let rect = ui.max_rect();
                ui.painter().vline(
                    rect.right(),
                    rect.y_range(),
                    egui::Stroke::new(1.0_f32, Theme::BORDER_DARK),
                );
            });
    }
}
