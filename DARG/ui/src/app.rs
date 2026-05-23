use crate::fonts::setup_custom_fonts;
use crate::ui;
use crate::ui::themes::Theme;
use crate::utils::get_existing_workflows;
use eframe::egui;
use std::sync::mpsc::Receiver;

#[derive(PartialEq, Clone)]
pub enum TaskStatus {
    Pending,
    Running,
    Success,
    // Failed,
}

#[derive(Clone)]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub status: TaskStatus,
    pub logs: Vec<String>,
}

#[derive(PartialEq)]
pub enum AppView {
    Projects,
    Workflows,
    Settings,
}

pub struct UiState {
    pub is_maximized: bool,
    pub current_view: AppView,
    pub show_import_modal: bool,
    pub import_message: String,
    pub import_is_error: bool,
    pub is_importing: bool,
    pub expected_file: String,
}

pub struct CoreState {
    pub workflows: Vec<String>,
    pub selected_workflow: Option<String>,
    pub current_tasks: Vec<Task>,
}

pub struct ArthemisApp {
    pub ui: UiState,
    pub core: CoreState,
    pub backend_rx: Option<Receiver<String>>,
}

impl ArthemisApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Theme::BG_APP;
        visuals.window_fill = Theme::BG_WINDOW;
        visuals.selection.bg_fill = Theme::SELECTION_BG;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;

        cc.egui_ctx.set_visuals(visuals);

        Self::default()
    }
}

impl Default for ArthemisApp {
    fn default() -> Self {
        Self {
            ui: UiState {
                is_maximized: false,
                current_view: AppView::Workflows,
                show_import_modal: false,
                import_message: String::new(),
                import_is_error: false,
                is_importing: false,
                expected_file: String::new(),
            },
            core: CoreState {
                workflows: get_existing_workflows(),
                selected_workflow: None,
                current_tasks: vec![
                    Task {
                        id: 1,
                        name: "Ping_Check".to_string(),
                        status: TaskStatus::Success,
                        logs: vec![
                            "PING 127.0.0.1 56(84) bytes of data.".to_string(),
                            "64 bytes from 127.0.0.1: icmp_seq=1 ttl=64 time=0.123 ms".to_string(),
                        ],
                    },
                    Task {
                        id: 2,
                        name: "Compilar_Rust".to_string(),
                        status: TaskStatus::Running,
                        logs: vec![
                            "cargo build --release".to_string(),
                            "Compiling arthemis_worker v1.0.0".to_string(),
                        ],
                    },
                    Task {
                        id: 3,
                        name: "Deploy_Server".to_string(),
                        status: TaskStatus::Pending,
                        logs: vec![],
                    },
                ],
            },
            backend_rx: None,
        }
    }
}

impl eframe::App for ArthemisApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.backend_rx {
            while let Ok(msg) = rx.try_recv() {
                if msg.starts_with("LOADED:") {
                    let file_name = msg.replace("LOADED:", "");
                    
                    if self.ui.is_importing && self.ui.expected_file == file_name {
                        self.ui.is_importing = false;
                        self.ui.import_message = format!("Archivo '{}' guardado correctamente", file_name);
                        self.ui.import_is_error = false;
                        
                        if !self.core.workflows.contains(&file_name) {
                            self.core.workflows.push(file_name);
                        }
                    }
                }
                else if msg.starts_with("ERROR:") {
                    let parts: Vec<&str> = msg.splitn(3, ':').collect();
                    
                    if parts.len() == 3 {
                        let file_name = parts[1];
                        let error_reason = parts[2];

                        if self.ui.is_importing && self.ui.expected_file == file_name {
                            self.ui.is_importing = false;
                            self.ui.import_message = format!("Rechazado: {}", error_reason); 
                            self.ui.import_is_error = true;
                        }
                    }
                }
            }
        }

        if !self.ui.is_maximized {
            self.ui.is_maximized = true;
        }

        ui::sidebar::draw(ctx, &mut self.ui);
        ui::explorer::draw(ctx, &mut self.ui, &mut self.core);
        ui::central::draw(ctx, &self.ui, &self.core);

        ui::modals::draw_import_modal(ctx, &mut self.ui);
    }
}
