use crate::app::{AppView, CoreState, TaskStatus, UiState, WorkflowExecutionState};
use crate::ui::components::task_row;
use crate::ui::themes::Theme;
use eframe::egui;

pub fn draw(
    ctx: &egui::Context,
    ui_state: &UiState,
    core_state: &mut CoreState,
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

                                ui.add_space(12.0);
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Workers online: {}",
                                        ui_state.connected_workers
                                    ))
                                    .size(13.0)
                                    .color(Theme::TEXT_MUTED),
                                );

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.add_space(5.0);
                                        let has_started = !core_state.current_tasks.is_empty()
                                            && core_state.current_tasks.iter().any(|t| t.status != TaskStatus::Sleeping);

                                        let has_finished = !core_state.current_tasks.is_empty()
                                            && core_state.current_tasks.iter().all(|t| {
                                                t.status == TaskStatus::Success || t.status == TaskStatus::Failed
                                            });

                                        let can_click = !has_started || has_finished;

                                        let main_icon = if has_started {
                                            "\u{200B}\u{f0e2}\u{200B}"
                                        } else {
                                            "\u{200B}\u{f04b}\u{200B}"
                                        };

                                        let mut btn_action = egui::Button::new(
                                            egui::RichText::new(main_icon).size(16.0),
                                        )
                                        .frame(false)
                                        .rounding(egui::Rounding::same(6.0));

                                        if !can_click {
                                            btn_action = btn_action.sense(egui::Sense::hover());
                                        }

                                        let action_response = ui.add_sized([28.0, 28.0], btn_action);

                                        if can_click {
                                            let hover_text = if has_finished {
                                                "Reiniciar ejecución (Limpia logs y vuelve a comenzar de inmediato)"
                                            } else {
                                                "Comenzar ejecución del Pipeline"
                                            };

                                            let action_response = action_response
                                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                                .on_hover_text(hover_text);

                                            if action_response.clicked() {
                                                core_state.workflow_execution_state = WorkflowExecutionState::Running;
                                                core_state.workflow_running = true;

                                                // -------------------------------------------------------------
                                                // PETICIONES HTTP AL COORDINADOR EN LUGAR DE BDD LOCAL
                                                // -------------------------------------------------------------
                                                if let Some(user_id) = ui_state.session_user_id {
                                                    let client = reqwest::blocking::Client::new();

                                                    if has_finished {
                                                        // RESET: Si ya había terminado, reiniciamos el workflow vía HTTP
                                                        let url = format!(
                                                            "http://{}:8081/reset_workflow",
                                                            ui_state.coordinator_ip
                                                        );
                                                        let body = serde_json::json!({
                                                            "user_id": user_id,
                                                            "workflow_name": wf
                                                        });
                                                        let _ = client.post(&url).json(&body).send();
                                                    } else {
                                                        // START: Pasamos las tareas a PENDING vía HTTP
                                                        for task in &core_state.current_tasks {
                                                            let url = format!(
                                                                "http://{}:8081/start_task",
                                                                ui_state.coordinator_ip
                                                            );
                                                            let body = serde_json::json!({
                                                                "task_id": task.id
                                                            });
                                                            let _ = client.post(&url).json(&body).send();
                                                        }
                                                    }
                                                }

                                                // Actualización visual local en la UI
                                                for task in &mut core_state.current_tasks {
                                                    task.status = TaskStatus::Pending;
                                                    if has_finished {
                                                        task.logs.clear();
                                                    }
                                                }
                                            }
                                        } else {
                                            let _ = action_response.on_hover_text("Pipeline en ejecución. Espere a que termine o falle.");
                                            if core_state.workflow_execution_state == WorkflowExecutionState::Running {
                                                ctx.request_repaint();
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
                        egui::RichText::new(format!(
                            "Workers online: {}",
                            ui_state.connected_workers
                        ))
                        .size(15.0)
                        .strong()
                        .color(Theme::TEXT_WHITE),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("Aquí irán las opciones de los Workers y red.")
                            .color(Theme::TEXT_MUTED),
                    );
                });
            });
        }

        AppView::Login => {}
    });
}
