use crate::fonts::setup_custom_fonts;
use crate::ui;
use crate::ui::themes::Theme;
use coordinator::db::Database;
use eframe::egui;
use std::sync::mpsc::Receiver;

#[derive(PartialEq, Clone)]
pub enum TaskStatus {
    Sleeping,
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
    pub connected_workers: usize,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum WorkflowExecutionState {
    Idle,    // Nadie tocó nada, listo para dar Play
    Running, // Corriendo tareas, UI completamente bloqueada en modo Pausa
}
pub struct CoreState {
    pub workflows: Vec<(String, String)>,
    pub selected_workflow: Option<String>,
    pub loaded_workflow: Option<String>,
    pub current_tasks: Vec<Task>,
    pub workflow_running: bool,
    pub workflow_execution_state: WorkflowExecutionState,
}

pub struct ArthemisApp {
    pub ui: UiState,
    pub core: CoreState,
    pub backend_rx: Option<Receiver<String>>,
    pub db: Option<Database>,
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
            && let Ok(wfs) = db.get_all_workflows()
        {
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
                connected_workers: 0,
            },
            core: CoreState {
                workflows: vec![],
                selected_workflow: None,
                loaded_workflow: None,
                current_tasks: vec![],
                workflow_running: false,
                workflow_execution_state: WorkflowExecutionState::Idle,
            },
            backend_rx: None,
            db: None,
        }
    }
}

impl eframe::App for ArthemisApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.backend_rx {
            while let Ok(msg) = rx.try_recv() {
                if msg.starts_with("WORKERS:") {
                    if let Some(count) = msg.split(':').nth(1)
                        && let Ok(parsed) = count.parse::<usize>()
                    {
                        self.ui.connected_workers = parsed;
                    }
                } else if msg.starts_with("LOADED:") {
                    let parts: Vec<&str> = msg.splitn(3, ':').collect();
                    if parts.len() == 3 {
                        let file_name = parts[1].to_string();
                        let display_name = parts[2].to_string();

                        if self.ui.is_importing && self.ui.expected_file == file_name {
                            self.ui.is_importing = false;
                            self.ui.import_message = format!("Pipeline '{}' guardado", file_name);
                            self.ui.import_is_error = false;
                        }

                        if let Some(existing) = self
                            .core
                            .workflows
                            .iter_mut()
                            .find(|(f, _)| f == &file_name)
                        {
                            existing.1 = display_name;
                        } else {
                            self.core.workflows.push((file_name.clone(), display_name));
                        }

                        if self.core.selected_workflow.as_ref() == Some(&file_name) {
                            self.core.loaded_workflow = None;
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
                } else if msg.starts_with("LOG:") {
                    let parts: Vec<&str> = msg.splitn(3, ':').collect();
                    if parts.len() == 3
                        && let Ok(task_id) = parts[1].parse::<i32>()
                    {
                        let log_content = parts[2].to_string();

                        if let Some(task) =
                            self.core.current_tasks.iter_mut().find(|t| t.id == task_id)
                        {
                            task.logs.push(log_content);
                        }
                    }
                } else if msg.starts_with("STATUS:") {
                    let parts: Vec<&str> = msg.splitn(3, ':').collect();
                    if parts.len() == 3
                        && let Ok(task_id) = parts[1].parse::<i32>()
                    {
                        let status_str = parts[2];
                        let status_normalized = status_str.to_ascii_uppercase();

                        let new_status = match status_normalized.as_str() {
                            "PENDING" => TaskStatus::Pending,
                            "RUNNING" => TaskStatus::Running,
                            "SUCCESS" => TaskStatus::Success,
                            "FAILED" => TaskStatus::Failed,
                            _ => TaskStatus::Sleeping,
                        };

                        if let Some(task) =
                            self.core.current_tasks.iter_mut().find(|t| t.id == task_id)
                        {
                            task.status = new_status.clone();
                            if new_status == TaskStatus::Running {
                                task.logs.clear();
                                self.core.workflow_running = true;
                                self.core.workflow_execution_state =
                                    WorkflowExecutionState::Running;
                            }
                        }
                    }
                }
            }
        }

        if self.core.selected_workflow != self.core.loaded_workflow {
            if let Some(wf) = &self.core.selected_workflow {
                if let Some(db) = &self.db
                    && let Ok(backend_tasks) = db.get_tasks_for_ui(wf)
                {
                    let mut ui_tasks = Vec::new();
                    for (id, name, status_str, logs) in backend_tasks {
                        let status_normalized = status_str.to_ascii_uppercase();
                        let status = match status_normalized.as_str() {
                            "RUNNING" => TaskStatus::Running,
                            "SUCCESS" => TaskStatus::Success,
                            "FAILED" => TaskStatus::Failed,
                            _ => TaskStatus::Sleeping,
                        };
                        ui_tasks.push(Task {
                            id,
                            name,
                            status,
                            logs,
                        });
                    }

                    self.core.current_tasks = ui_tasks;

                    self.core.loaded_workflow = Some(wf.clone());
                }
            } else {
                self.core.current_tasks.clear();
                self.core.loaded_workflow = None;
            }
        }

        if self.core.workflow_execution_state == WorkflowExecutionState::Running {
            let has_running = self
                .core
                .current_tasks
                .iter()
                .any(|t| t.status == TaskStatus::Running);
            let has_pending = self
                .core
                .current_tasks
                .iter()
                .any(|t| t.status == TaskStatus::Pending);
            if !has_running && !has_pending {
                self.core.workflow_running = false;
                self.core.workflow_execution_state = WorkflowExecutionState::Idle;
            }
        }

        if !self.ui.is_maximized {
            self.ui.is_maximized = true;
        }

        ui::sidebar::draw(ctx, &mut self.ui);
        ui::explorer::draw(ctx, &mut self.ui, &mut self.core, self.db.as_ref());
        ui::central::draw(ctx, &self.ui, &mut self.core, self.db.as_ref());

        ui::modals::draw_import_modal(ctx, &mut self.ui);
    }
}
