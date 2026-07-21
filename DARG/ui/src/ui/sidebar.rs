use crate::app::{AppView, UiState};
use crate::ui::themes::Theme;
use eframe::egui;

pub fn draw(ctx: &egui::Context, ui_state: &mut UiState) {
    let frame = egui::Frame::none().fill(Theme::BG_SIDEBAR);

    egui::SidePanel::left("nav_panel")
        .resizable(false)
        .exact_width(60.0)
        .frame(frame)
        .show(ctx, |ui| {
            ui.add_space(20.0);

            ui.vertical_centered(|ui| {
                let icon_size = 24.0;
                let btn_size = egui::vec2(46.0, 46.0);

                let nav_items = [
                    (AppView::Projects, "\u{200B}\u{f07b}\u{200B}", "Proyectos"),
                    (AppView::Workflows, "\u{200B}\u{f126}\u{200B}", "Workflows"),
                    (
                        AppView::Settings,
                        "\u{200B}\u{f013}\u{200B}",
                        "Configuración",
                    ),
                ];

                for (view, icon, tooltip) in nav_items {
                    let is_selected = ui_state.current_view == view;
                    let (rect, response) = ui.allocate_exact_size(btn_size, egui::Sense::click());

                    let icon_color = if is_selected {
                        Theme::SELECTION_BG
                    } else if response.hovered() {
                        Theme::TEXT_WHITE
                    } else {
                        Theme::TEXT_MUTED
                    };

                    ui.allocate_ui_at_rect(rect, |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(icon).size(icon_size).color(icon_color),
                                )
                                .selectable(false),
                            );
                        });
                    });

                    if response.clicked() {
                        ui_state.current_view = view;
                    }
                    response
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text(tooltip);

                    ui.add_space(10.0);
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(ui_state.connected_workers.to_string())
                            .size(18.0)
                            .strong()
                            .color(Theme::TEXT_WHITE),
                    );
                    ui.label(
                        egui::RichText::new("workers")
                            .size(10.0)
                            .color(Theme::TEXT_MUTED),
                    );
                });
            });

            let rect = ui.max_rect();
            ui.painter().vline(
                rect.right(),
                rect.y_range(),
                egui::Stroke::new(1.0, Theme::BORDER_DARK),
            );
        });
}
