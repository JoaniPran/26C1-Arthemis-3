use crate::fonts::setup_custom_fonts;
use crate::ui;
use crate::ui::themes::Theme;
use coordinator::db::Database;
use eframe::egui;
use std::sync::mpsc::Receiver;
use std::time::Instant;

#[derive(PartialEq, Clone)]
pub enum TaskStatus {
    Pending,
    Running,
    Success,
    Failed,
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
    pub workflows: Vec<(String, String)>,
    pub selected_workflow: Option<String>,
    pub current_tasks: Vec<Task>,
}

pub struct ArthemisApp {
    pub ui: UiState,
    pub core: CoreState,
    pub backend_rx: Option<Receiver<String>>,
    pub db: Option<Database>,
    pub last_db_sync: Instant,
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

        let db_instance = Database::new("arthemis.db").ok();

        let mut initial_workflows = vec![];
        if let Some(db) = &db_instance 
            && let Ok(wfs) = db.get_all_workflows() {
                initial_workflows = wfs;
            }
        

        let mut app = Self::default();
        app.core.workflows = initial_workflows;
        app.db = db_instance;
        app
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
                workflows: vec![],
                selected_workflow: None,
                current_tasks: vec![],
            },
            backend_rx: None,
            db: None,
            last_db_sync: Instant::now(),
        }
    }
}

impl eframe::App for ArthemisApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.backend_rx {
            while let Ok(msg) = rx.try_recv() {
                if msg.starts_with("LOADED:") {
                    let parts: Vec<&str> = msg.splitn(3, ':').collect();
                    if parts.len() == 3 {
                        let file_name = parts[1].to_string();
                        let display_name = parts[2].to_string();

                        if self.ui.is_importing && self.ui.expected_file == file_name {
                            self.ui.is_importing = false;
                            self.ui.import_message =
                                format!("Pipeline '{}' guardado", display_name);
                            self.ui.import_is_error = false;

                            if !self.core.workflows.iter().any(|(f, _)| f == &file_name) {
                                self.core.workflows.push((file_name, display_name));
                            }
                        }
                    }
                } else if msg.starts_with("ERROR:") {
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

        if self.last_db_sync.elapsed().as_millis() > 500 {
            self.last_db_sync = Instant::now();

            if let Some(wf) = &self.core.selected_workflow 
                && let Some(db) = &self.db 
                    && let Ok(backend_tasks) = db.get_tasks_for_ui(wf) {
                        let mut ui_tasks = Vec::new();
                        for (id, name, status_str, logs) in backend_tasks {
                            let status = match status_str.as_str() {
                                "RUNNING" => TaskStatus::Running,
                                "SUCCESS" => TaskStatus::Success,
                                "FAILED" => TaskStatus::Failed,
                                _ => TaskStatus::Pending,
                            };
                            ui_tasks.push(Task {
                                id,
                                name,
                                status,
                                logs,
                            });
                        }

                        self.core.current_tasks = ui_tasks;
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
