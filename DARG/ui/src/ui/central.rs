use crate::app::{AppView, CoreState, UiState, TaskStatus, WorkflowExecutionState};
use crate::ui::components::task_row;
use crate::ui::themes::Theme;
use coordinator::db::Database;
use eframe::egui;

pub fn draw(
    ctx: &egui::Context,
    ui_state: &UiState,
    core_state: &mut CoreState,
    db: Option<&Database>,
) {
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
                                        let icon_pause = "\u{200B}\u{f04c}\u{200B}";

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

                                        let is_running = core_state.workflow_execution_state
                                            == WorkflowExecutionState::Running;

                                        let play_icon =
                                            if is_running { icon_pause } else { icon_play };

                                        let mut btn_play = egui::Button::new(
                                            egui::RichText::new(play_icon).size(16.0),
                                        )
                                        .frame(false)
                                        .rounding(egui::Rounding::same(6.0));

                                        if is_running {
                                            btn_play = btn_play.sense(egui::Sense::hover());
                                            ctx.request_repaint();
                                        }

                                        let play_response = ui.add_sized([28.0, 28.0], btn_play);

                                        if core_state.workflow_execution_state
                                            == WorkflowExecutionState::Idle
                                        {
                                            let play_response = play_response
                                                .on_hover_cursor(egui::CursorIcon::PointingHand);

                                            if play_response.clicked() {
                                                core_state.workflow_execution_state =
                                                    WorkflowExecutionState::Running;

                                                if let Some(database) = db {
                                                    for task in &core_state.current_tasks {
                                                        let _ = database.set_task_pending(task.id);
                                                    }
                                                }

                                                for task in &mut core_state.current_tasks {
                                                    task.status = TaskStatus::Pending;
                                                }

                                            }
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
                                    for task in &mut core_state.current_tasks {
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
