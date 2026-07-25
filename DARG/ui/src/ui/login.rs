use crate::app::{AppView, CoreState, UiState};
use crate::ui::themes::Theme;
use coordinator::db::Database;
use eframe::egui;

pub fn draw(
    ctx: &egui::Context,
    ui_state: &mut UiState,
    core_state: &mut CoreState,
    _db: Option<&Database>,
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
                            let btn_login =
                                egui::Button::new(egui::RichText::new("Iniciar Sesión").strong())
                                    .fill(Theme::ACCENT_BLUE)
                                    .rounding(6.0);

                            if ui.add_sized([120.0, 35.0], btn_login).clicked() {
                                let username = ui_state.login_username_input.trim().to_string();
                                let password = ui_state.login_password_input.trim().to_string();

                                if username.is_empty() || password.is_empty() {
                                    ui_state.login_error =
                                        "Usuario y contraseña requeridos".to_string();
                                } else {
                                    let url =
                                        format!("http://{}:8081/login", ui_state.coordinator_ip);
                                    let body = serde_json::json!({
                                        "username": username,
                                        "password": password
                                    });

                                    let client = reqwest::blocking::Client::new();
                                    match client.post(&url).json(&body).send() {
                                        Ok(response) if response.status().is_success() => {
                                            if let Ok(data) = response.json::<serde_json::Value>() {
                                                let user_id =
                                                    data["user_id"].as_i64().unwrap_or(0) as i32;

                                                ui_state.session_user_id = Some(user_id);
                                                ui_state.session_username = Some(username.clone());
                                                ui_state.login_error.clear();

                                                core_state.workflows.clear();
                                                core_state.selected_workflow = None;
                                                core_state.loaded_workflow = None;
                                                core_state.current_tasks.clear();

                                                let wf_url = format!(
                                                    "http://{}:8081/workflows/{}",
                                                    ui_state.coordinator_ip, user_id
                                                );
                                                if let Ok(wf_res) = client.get(&wf_url).send() {
                                                    if let Ok(user_wfs) =
                                                        wf_res.json::<Vec<(String, String)>>()
                                                    {
                                                        core_state.workflows = user_wfs;
                                                    }
                                                }

                                                ui_state.current_view = AppView::Workflows;
                                            }
                                        }
                                        Ok(response) if response.status().as_u16() == 401 => {
                                            ui_state.login_error =
                                                "Credenciales incorrectas".to_string();
                                        }
                                        Ok(_) => {
                                            ui_state.login_error =
                                                "Error del servidor al iniciar sesión".to_string();
                                        }
                                        Err(e) => {
                                            ui_state.login_error = format!(
                                                "No se pudo conectar al Coordinador ({}:8081): {}",
                                                ui_state.coordinator_ip, e
                                            );
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
                                let username = ui_state.login_username_input.trim().to_string();
                                let password = ui_state.login_password_input.trim().to_string();

                                if username.is_empty() || password.is_empty() {
                                    ui_state.login_error =
                                        "Usuario y contraseña requeridos".to_string();
                                } else {
                                    let url =
                                        format!("http://{}:8081/register", ui_state.coordinator_ip);
                                    let body = serde_json::json!({
                                        "username": username,
                                        "password": password
                                    });

                                    let client = reqwest::blocking::Client::new();
                                    match client.post(&url).json(&body).send() {
                                        Ok(response) if response.status().is_success() => {
                                            if let Ok(data) = response.json::<serde_json::Value>() {
                                                let user_id =
                                                    data["user_id"].as_i64().unwrap_or(0) as i32;

                                                ui_state.session_user_id = Some(user_id);
                                                ui_state.session_username = Some(username.clone());
                                                ui_state.login_error.clear();

                                                core_state.workflows.clear();
                                                core_state.selected_workflow = None;
                                                core_state.loaded_workflow = None;
                                                core_state.current_tasks.clear();

                                                let wf_url = format!(
                                                    "http://{}:8081/workflows/{}",
                                                    ui_state.coordinator_ip, user_id
                                                );
                                                if let Ok(wf_res) = client.get(&wf_url).send() {
                                                    if let Ok(user_wfs) =
                                                        wf_res.json::<Vec<(String, String)>>()
                                                    {
                                                        core_state.workflows = user_wfs;
                                                    }
                                                }

                                                ui_state.current_view = AppView::Workflows;
                                            }
                                        }
                                        Ok(_) => {
                                            ui_state.login_error =
                                                "El usuario ya existe o hubo un error".to_string();
                                        }
                                        Err(e) => {
                                            ui_state.login_error = format!(
                                                "No se pudo conectar al Coordinador ({}:8081): {}",
                                                ui_state.coordinator_ip, e
                                            );
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