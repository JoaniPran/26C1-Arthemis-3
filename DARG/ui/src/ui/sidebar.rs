use crate::app::{AppView, UiState};
use eframe::egui;

pub fn draw(ctx: &egui::Context, ui_state: &mut UiState) {
    egui::SidePanel::left("nav_panel")
        .resizable(false)
        .exact_width(60.0)
        .show(ctx, |ui| {
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                let icon_size = 24.0;
                let icon_folder = "\u{200B}\u{f07b}\u{200B}";
                let icon_branch = "\u{200B}\u{f126}\u{200B}";
                let icon_gear = "\u{200B}\u{f013}\u{200B}";

                if ui
                    .selectable_label(
                        ui_state.current_view == AppView::Projects,
                        egui::RichText::new(icon_folder).size(icon_size),
                    )
                    .on_hover_text("Proyectos")
                    .clicked()
                {
                    ui_state.current_view = AppView::Projects;
                }
                ui.add_space(20.0);
                if ui
                    .selectable_label(
                        ui_state.current_view == AppView::Workflows,
                        egui::RichText::new(icon_branch).size(icon_size),
                    )
                    .on_hover_text("Workflows")
                    .clicked()
                {
                    ui_state.current_view = AppView::Workflows;
                }
                ui.add_space(20.0);
                if ui
                    .selectable_label(
                        ui_state.current_view == AppView::Settings,
                        egui::RichText::new(icon_gear).size(icon_size),
                    )
                    .on_hover_text("Configuración")
                    .clicked()
                {
                    ui_state.current_view = AppView::Settings;
                }
            });
        });
}
