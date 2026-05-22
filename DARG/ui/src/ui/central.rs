use crate::app::{AppView, CoreState, UiState, TaskStatus};
use eframe::egui;

pub fn draw(ctx: &egui::Context, ui_state: &UiState, core_state: &CoreState) {
    egui::CentralPanel::default().show(ctx, |ui| match ui_state.current_view {
        AppView::Workflows => {
            if let Some(wf) = &core_state.selected_workflow {
                egui::Frame::none()
                    .inner_margin(egui::Margin::symmetric(20.0, 15.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("Pipeline: {}", wf))
                                    .size(16.0)
                                    .strong()
                                    .color(egui::Color32::WHITE)
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let icon_stop = "\u{200B}\u{f04d}\u{200B}";
                                let icon_play = "\u{200B}\u{f04b}\u{200B}";
                                
                                let btn_stop = egui::Button::new(egui::RichText::new(icon_stop).size(16.0))
                                    .frame(false)
                                    .rounding(egui::Rounding::same(6.0));

                                if ui.add_sized([28.0, 28.0], btn_stop).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                    println!("Deteniendo...");
                                }
                                
                                ui.add_space(5.0);
                                
                                let btn_play = egui::Button::new(egui::RichText::new(icon_play).size(16.0))
                                    .frame(false)
                                    .rounding(egui::Rounding::same(6.0));

                                if ui.add_sized([28.0, 28.0], btn_play).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                    println!("Ejecutando...");
                                }
                            });
                        });
                    });

                let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 0.0, egui::Color32::from_gray(40));

                egui::Frame::none().inner_margin(20.0).show(ui, |ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(23, 23, 23))
                        .stroke(egui::Stroke::new(0.5, egui::Color32::from_rgb(45, 50, 55))) 
                        .rounding(6.0)
                        .inner_margin(egui::Margin { left: 8.0, right: 8.0, top: 10.0, bottom: 10.0 })
                        .show(ui, |ui| {
                            ui.set_min_height(ui.available_height());
                            ui.set_min_width(ui.available_width());

                            egui::ScrollArea::vertical()
                                .auto_shrink([false; 2])
                                .show(ui, |ui| {
                                    for task in &core_state.current_tasks {
                                        let id = ui.make_persistent_id(task.id);
                                        let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(ctx, id, task.status == TaskStatus::Running);
                                        let is_open = state.is_open();

                                        let row_height = 36.0;
                                        let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), row_height), egui::Sense::click());

                                        if response.clicked() {
                                            state.toggle(ui);
                                        }

                                        let bg_color = if is_open {
                                            egui::Color32::from_rgb(45, 50, 55)
                                        } else if response.hovered() {
                                            egui::Color32::from_rgb(28, 32, 38) 
                                        } else {
                                            egui::Color32::TRANSPARENT
                                        };

                                        if bg_color != egui::Color32::TRANSPARENT {
                                            ui.painter().rect_filled(rect, 4.0, bg_color);
                                        }
                                        
                                        response.on_hover_cursor(egui::CursorIcon::PointingHand);

                                        ui.allocate_ui_at_rect(rect, |ui| {
                                            ui.horizontal_centered(|ui| {
                                                ui.add_space(15.0);
                                            
                                                let icon_arrow = if is_open { "\u{f078}" } else { "\u{f054}" };
                                                ui.add_sized(
                                                    [12.0, ui.available_height()],
                                                    egui::Label::new(egui::RichText::new(icon_arrow).size(11.0).color(egui::Color32::from_gray(140))).selectable(false)
                                                );

                                                ui.add_space(15.0); 

                                                let (icon, color) = match task.status {
                                                    TaskStatus::Pending => ("\u{200B}\u{f017}\u{200B}", egui::Color32::GRAY),       
                                                    TaskStatus::Running => ("\u{200B}\u{f111}\u{200B}", egui::Color32::YELLOW),      
                                                    TaskStatus::Success => ("\u{200B}\u{f058}\u{200B}", egui::Color32::LIGHT_GREEN), 
                                                    // TaskStatus::Failed  => ("\u{200B}\u{f057}\u{200B}", egui::Color32::LIGHT_RED),
                                                };

                                                ui.add_sized(
                                                    [16.0, ui.available_height()],
                                                    egui::Label::new(egui::RichText::new(icon).size(14.0).color(color)).selectable(false)
                                                );

                                                ui.add_space(5.0);
                                                ui.add(egui::Label::new(egui::RichText::new(&task.name).size(14.0).color(egui::Color32::from_gray(230))).selectable(false));
                                            });
                                        });

                                        state.show_body_unindented(ui, |ui| {
                                            egui::Frame::none()
                                                .inner_margin(egui::Margin { left: 55.0, right: 10.0, top: 5.0, bottom: 15.0 })
                                                .show(ui, |ui| {
                                                    if task.logs.is_empty() {
                                                        ui.label(egui::RichText::new("No hay logs disponibles.").color(egui::Color32::DARK_GRAY).monospace().size(12.0));
                                                    } else {
                                                        for log in &task.logs {
                                                            ui.label(egui::RichText::new(log).color(egui::Color32::LIGHT_GRAY).monospace().size(12.0));
                                                        }
                                                    }
                                                });
                                        });
                                    }
                                });
                        });
                });

            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new("Selecciona un workflow del explorador para ver sus detalles.").color(egui::Color32::GRAY).size(14.0));
                });
            }
        }

        AppView::Projects => {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Panel Principal de Proyectos");
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("Aquí visualizaremos métricas a nivel entorno.")
                            .color(egui::Color32::GRAY),
                    );
                });
            });
        }

        AppView::Settings => {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Configuración del Clúster");
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("Aquí irán las opciones de los Workers y red.")
                            .color(egui::Color32::GRAY),
                    );
                });
            });
        }
    });
}
