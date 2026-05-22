use crate::app::{AppView, UiState};
use eframe::egui;

pub fn draw(ctx: &egui::Context, ui_state: &mut UiState) {
    let frame = egui::Frame::none().fill(egui::Color32::from_rgb(23, 23, 23));

    egui::SidePanel::left("nav_panel")
        .resizable(false)
        .exact_width(60.0)
        .frame(frame)
        .show(ctx, |ui| {
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                let icon_size = 24.0;
                let btn_size = [46.0, 46.0];
                
                let icon_folder = "\u{200B}\u{f07b}\u{200B}";
                let icon_branch = "\u{200B}\u{f126}\u{200B}";
                let icon_gear = "\u{200B}\u{f013}\u{200B}";

                let fill_projects = if ui_state.current_view == AppView::Projects { ui.visuals().selection.bg_fill } else { egui::Color32::TRANSPARENT };
                let btn_projects = egui::Button::new(egui::RichText::new(icon_folder).size(icon_size))
                    .frame(ui_state.current_view == AppView::Projects)
                    .fill(fill_projects)
                    .rounding(egui::Rounding::same(10.0));

                if ui.add_sized(btn_size, btn_projects)
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text("Proyectos")
                    .clicked()
                {
                    ui_state.current_view = AppView::Projects;
                }
                
                ui.add_space(15.0);
                
                let fill_workflows = if ui_state.current_view == AppView::Workflows { ui.visuals().selection.bg_fill } else { egui::Color32::TRANSPARENT };
                let btn_workflows = egui::Button::new(egui::RichText::new(icon_branch).size(icon_size))
                    .frame(ui_state.current_view == AppView::Workflows)
                    .fill(fill_workflows)
                    .rounding(egui::Rounding::same(10.0));

                if ui.add_sized(btn_size, btn_workflows)
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text("Workflows")
                    .clicked()
                {
                    ui_state.current_view = AppView::Workflows;
                }
                
                ui.add_space(15.0);
                
                let fill_settings = if ui_state.current_view == AppView::Settings { ui.visuals().selection.bg_fill } else { egui::Color32::TRANSPARENT };
                let btn_settings = egui::Button::new(egui::RichText::new(icon_gear).size(icon_size))
                    .frame(ui_state.current_view == AppView::Settings)
                    .fill(fill_settings)
                    .rounding(egui::Rounding::same(10.0));

                if ui.add_sized(btn_size, btn_settings)
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text("Configuración")
                    .clicked()
                {
                    ui_state.current_view = AppView::Settings;
                }

                let rect = ui.max_rect();
                ui.painter().vline(rect.right(), rect.y_range(), egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 40)));
            });
        });
}
