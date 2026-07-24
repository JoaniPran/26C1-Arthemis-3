use crate::app::{AppView, CoreState, UiState};
use crate::ui::themes::Theme;
use coordinator::db::Database;
use eframe::egui;

pub fn draw(
    ctx: &egui::Context,
    ui_state: &mut UiState,
    core_state: &mut CoreState,
    db: Option<&Database>,
) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.centered_and_justified(|ui| {
            egui::Frame::none()
                .fill(Theme::BG_CENTRAL_PANEL)
                .stroke(egui::Stroke::new(1.0, Theme::BORDER_LIGHT))
                .rounding(10.0)
                .inner_margin(40.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(
                            egui::RichText::new("Arthemis Orchestrator")
                                .size(28.0)
                                .strong()
                                .color(Theme::TEXT_WHITE),
                        );
                        ui.add_space(30.0);

                        ui.label(
                            egui::RichText::new("Usuario")
                                .size(14.0)
                                .color(Theme::TEXT_LIGHT_GRAY),
                        );
                        ui.add_space(5.0);
                        ui.add_sized(
                            [250.0, 30.0],
                            egui::TextEdit::singleline(&mut ui_state.login_username_input),
                        );
                        
                        ui.add_space(15.0);

                        ui.label(
                            egui::RichText::new("Contraseña")
                                .size(14.0)
                                .color(Theme::TEXT_LIGHT_GRAY),
                        );
                        ui.add_space(5.0);
                        ui.add_sized(
                            [250.0, 30.0],
                            egui::TextEdit::singleline(&mut ui_state.login_password_input)
                                .password(true),
                        );

                        ui.add_space(20.0);

                        if !ui_state.login_error.is_empty() {
                            ui.label(
                                egui::RichText::new(&ui_state.login_error)
                                    .color(Theme::STATUS_FAILED)
                                    .size(13.0),
                            );
                            ui.add_space(10.0);
                        }

                        ui.horizontal_centered(|ui| {
                            let btn_login = egui::Button::new(
                                egui::RichText::new("Iniciar Sesión").strong(),
                            )
                            .fill(Theme::ACCENT_BLUE)
                            .rounding(6.0);

                            if ui.add_sized([120.0, 35.0], btn_login).clicked() {
                                if ui_state.login_username_input.trim().is_empty() || ui_state.login_password_input.trim().is_empty() {
                                    ui_state.login_error = "Usuario y contraseña requeridos".to_string();
                                } else if let Some(database) = db {
                                    match database.authenticate_user(
                                        &ui_state.login_username_input,
                                        &ui_state.login_password_input,
                                    ) {
                                        Ok(user_id) => {
                                            ui_state.session_user_id = Some(user_id);
                                            ui_state.session_username = Some(ui_state.login_username_input.clone());
                                            ui_state.login_error.clear();
                                            
                                            if let Ok(wfs) = database.get_workflows_for_user(user_id) {
                                                core_state.workflows = wfs;
                                            }
                                            
                                            ui_state.current_view = AppView::Workflows;
                                        }
                                        Err(_) => {
                                            ui_state.login_error = "Credenciales incorrectas".to_string();
                                        }
                                    }
                                }
                            }

                            let btn_register = egui::Button::new(
                                egui::RichText::new("Registrarse").color(Theme::TEXT_LIGHT_GRAY),
                            )
                            .fill(Theme::HOVER_ROW)
                            .rounding(6.0);

                            if ui.add_sized([120.0, 35.0], btn_register).clicked() {
                                if ui_state.login_username_input.trim().is_empty() || ui_state.login_password_input.trim().is_empty() {
                                    ui_state.login_error = "Usuario y contraseña requeridos".to_string();
                                } else if let Some(database) = db {
                                    match database.register_user(
                                        &ui_state.login_username_input,
                                        &ui_state.login_password_input,
                                    ) {
                                        Ok(user_id) => {
                                            ui_state.session_user_id = Some(user_id);
                                            ui_state.session_username = Some(ui_state.login_username_input.clone());
                                            ui_state.login_error.clear();
                                            core_state.workflows.clear();
                                            let workspace_path = std::path::Path::new("workflows").join(&ui_state.login_username_input);
                                            let _ = std::fs::create_dir_all(workspace_path);

                                            ui_state.current_view = AppView::Workflows;
                                        }
                                        Err(_) => {
                                            ui_state.login_error =
                                                "El usuario ya existe o hubo un error".to_string();
                                        }
                                    }
                                }
                            }
                        });
                    });
                });
        });
    });
}