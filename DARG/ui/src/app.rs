use crate::fonts::setup_custom_fonts;
use crate::ui;
use eframe::egui;

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
}

pub struct CoreState {
    pub workflows: Vec<String>,
    pub selected_workflow: Option<String>,
    pub current_tasks: Vec<Task>,
}

pub struct ArthemisApp {
    pub ui: UiState,
    pub core: CoreState,
}

impl ArthemisApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(26, 26, 26);
        visuals.window_fill = egui::Color32::from_rgb(30, 30, 30);
        visuals.selection.bg_fill = egui::Color32::from_rgb(35, 75, 150);
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
            },
            core: CoreState {
                workflows: vec![
                    "Main Build".to_string(),
                    "Pipeline_Ejemplo".to_string(),
                    "Release Flow".to_string(),
                ],
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
        }
    }
}

impl eframe::App for ArthemisApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.ui.is_maximized {
            self.ui.is_maximized = true;
        }

        ui::sidebar::draw(ctx, &mut self.ui);
        ui::explorer::draw(ctx, &mut self.ui, &mut self.core);
        ui::central::draw(ctx, &self.ui, &self.core);

        ui::modals::draw_import_modal(ctx, &mut self.ui);
    }
}
