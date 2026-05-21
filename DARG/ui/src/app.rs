use crate::fonts::setup_custom_fonts;
use crate::ui;
use eframe::egui;

#[derive(PartialEq)]
pub enum AppView {
    Projects,
    Workflows,
    Settings,
}

pub struct UiState {
    pub is_maximized: bool,
    pub current_view: AppView,
}

pub struct CoreState {
    pub workflows: Vec<String>,
    pub selected_workflow: Option<String>,
}

pub struct ArthemisApp {
    pub ui: UiState,
    pub core: CoreState,
}

impl ArthemisApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Self::default()
    }
}

impl Default for ArthemisApp {
    fn default() -> Self {
        Self {
            ui: UiState {
                is_maximized: false,
                current_view: AppView::Workflows,
            },
            core: CoreState {
                workflows: vec![
                    "Main Build".to_string(),
                    "Pipeline_Ejemplo".to_string(),
                    "Release Flow".to_string(),
                ],
                selected_workflow: None,
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
        ui::explorer::draw(ctx, &self.ui, &mut self.core);
        ui::central::draw(ctx, &self.ui, &self.core);
    }
}
