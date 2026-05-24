use crate::app::{AppView, CoreState, UiState};
use crate::ui::components::task_row;
use crate::ui::themes::Theme;
use eframe::egui;

pub fn draw(ctx: &egui::Context, ui_state: &UiState, core_state: &CoreState) {
    egui::CentralPanel::default().show(ctx, |ui| match ui_state.current_view {
        AppView::Workflows => {
            if let Some(wf) = &core_state.selected_workflow {
                let display_name = core_state
                    .workflows
                    .iter()
                    .find(|(file_name, _)| file_name == wf)
                    .map(|(_, name)| name.as_str())
                    .unwrap_or(wf.as_str());

                egui::Frame::none()
                    .inner_margin(egui::Margin::symmetric(20.0, 15.0))
                    .show(ui, |ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(ui.available_width(), 28.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.label(
                                    egui::RichText::new(format!("Pipeline: {}", display_name))
                                        .size(16.0)
                                        .strong()
                                        .color(Theme::TEXT_WHITE),
                                );

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let icon_stop = "\u{200B}\u{f04d}\u{200B}";
                                        let icon_play = "\u{200B}\u{f04b}\u{200B}";

                                        let btn_stop = egui::Button::new(
                                            egui::RichText::new(icon_stop).size(16.0),
                                        )
                                        .frame(false)
                                        .rounding(egui::Rounding::same(6.0));

                                        if ui
                                            .add_sized([28.0, 28.0], btn_stop)
                                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                                            .clicked()
                                        {
                                            println!("Deteniendo...");
                                        }

                                        ui.add_space(5.0);

                                        let btn_play = egui::Button::new(
                                            egui::RichText::new(icon_play).size(16.0),
                                        )
                                        .frame(false)
                                        .rounding(egui::Rounding::same(6.0));

                                        if ui
                                            .add_sized([28.0, 28.0], btn_play)
                                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                                            .clicked()
                                        {
                                            println!("Ejecutando...");
                                        }
                                    },
                                );
                            },
                        );
                    });

                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 1.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 0.0, Theme::BORDER_DARK);

                egui::Frame::none().inner_margin(20.0).show(ui, |ui| {
                    egui::Frame::none()
                        .fill(Theme::BG_CENTRAL_PANEL)
                        .stroke(egui::Stroke::new(0.5, Theme::BORDER_LIGHT))
                        .rounding(6.0)
                        .inner_margin(egui::Margin {
                            left: 8.0,
                            right: 8.0,
                            top: 10.0,
                            bottom: 10.0,
                        })
                        .show(ui, |ui| {
                            ui.set_min_height(ui.available_height());
                            ui.set_min_width(ui.available_width());

                            egui::ScrollArea::vertical()
                                .auto_shrink([false; 2])
                                .show(ui, |ui| {
                                    for task in &core_state.current_tasks {
                                        task_row::draw(ctx, ui, task);
                                    }
                                });
                        });
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(
                            "Selecciona un workflow del explorador para ver sus detalles.",
                        )
                        .color(Theme::TEXT_MUTED)
                        .size(14.0),
                    );
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
                            .color(Theme::TEXT_MUTED),
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
                            .color(Theme::TEXT_MUTED),
                    );
                });
            });
        }
    });
}
