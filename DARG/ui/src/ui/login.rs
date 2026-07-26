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
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(Theme::BG_CENTRAL_PANEL))
        .show(ctx, |ui| {
            ui.add_space(15.0);

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(20.0); // Margen derecho desde el borde de la ventana

                    let btn_register = egui::Button::new(
                        egui::RichText::new("Registrarse").color(Theme::TEXT_LIGHT_GRAY),
                    )
                    .fill(Theme::HOVER_ROW)
                    .rounding(6.0);

                    if ui.add_sized([110.0, 32.0], btn_register).clicked() {
                        let username = ui_state.login_username_input.trim().to_string();
                        let password = ui_state.login_password_input.trim().to_string();

                        if username.is_empty() || password.is_empty() {
                            ui_state.login_error = "Usuario y contraseña requeridos".to_string();
                        } else {
                            let url = format!("https://{}:8081/register", ui_state.coordinator_ip);
                            let body = serde_json::json!({
                                "username": username,
                                "password": password
                            });

                            let client = crate::utils::insecure_client();
                            match client.post(&url).json(&body).send() {
                                Ok(response) if response.status().is_success() => {
                                    if let Ok(data) = response.json::<serde_json::Value>() {
                                        let user_id = data["user_id"].as_i64().unwrap_or(0) as i32;

                                        ui_state.session_user_id = Some(user_id);
                                        ui_state.session_username = Some(username.clone());
                                        ui_state.login_error.clear();

                                        core_state.workflows.clear();
                                        core_state.selected_workflow = None;
                                        core_state.loaded_workflow = None;
                                        core_state.current_tasks.clear();

                                        let wf_url = format!(
                                            "https://{}:8081/workflows/{}",
                                            ui_state.coordinator_ip, user_id
                                        );
                                        if let Ok(wf_res) = client.get(&wf_url).send()
                                            && let Ok(user_wfs) =
                                                wf_res.json::<Vec<(String, String)>>()
                                        {
                                            core_state.workflows = user_wfs;
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

                    ui.add_space(8.0);

                    let btn_login =
                        egui::Button::new(egui::RichText::new("Iniciar Sesión").strong())
                            .fill(Theme::ACCENT_BLUE)
                            .rounding(6.0);

                    if ui.add_sized([120.0, 32.0], btn_login).clicked() {
                        let username = ui_state.login_username_input.trim().to_string();
                        let password = ui_state.login_password_input.trim().to_string();

                        if username.is_empty() || password.is_empty() {
                            ui_state.login_error = "Usuario y contraseña requeridos".to_string();
                        } else {
                            let url = format!("https://{}:8081/login", ui_state.coordinator_ip);
                            let body = serde_json::json!({
                                "username": username,
                                "password": password
                            });

                            let client = crate::utils::insecure_client();
                            match client.post(&url).json(&body).send() {
                                Ok(response) if response.status().is_success() => {
                                    if let Ok(data) = response.json::<serde_json::Value>() {
                                        let user_id = data["user_id"].as_i64().unwrap_or(0) as i32;

                                        ui_state.session_user_id = Some(user_id);
                                        ui_state.session_username = Some(username.clone());
                                        ui_state.login_error.clear();

                                        core_state.workflows.clear();
                                        core_state.selected_workflow = None;
                                        core_state.loaded_workflow = None;
                                        core_state.current_tasks.clear();

                                        let wf_url = format!(
                                            "https://{}:8081/workflows/{}",
                                            ui_state.coordinator_ip, user_id
                                        );
                                        if let Ok(wf_res) = client.get(&wf_url).send()
                                            && let Ok(user_wfs) =
                                                wf_res.json::<Vec<(String, String)>>()
                                        {
                                            core_state.workflows = user_wfs;
                                        }

                                        ui_state.current_view = AppView::Workflows;
                                    }
                                }
                                Ok(response) if response.status().as_u16() == 401 => {
                                    ui_state.login_error = "Credenciales incorrectas".to_string();
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
                });
            });

            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(200.0);
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

                    if !ui_state.login_error.is_empty() {
                        ui.add_space(15.0);
                        ui.label(
                            egui::RichText::new(&ui_state.login_error)
                                .color(Theme::STATUS_FAILED)
                                .size(13.0),
                        );
                    }
                });
            });
        });
}
