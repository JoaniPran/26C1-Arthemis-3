use crate::app::{Task, TaskStatus};
use crate::ui::themes::Theme;
use eframe::egui;

pub fn draw(ctx: &egui::Context, ui: &mut egui::Ui, task: &mut Task) {
    let id = ui.make_persistent_id(task.id);
    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
        ctx,
        id,
        task.status == TaskStatus::Running,
    );
    let is_open = state.is_open();

    let row_height = 36.0;
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), row_height),
        egui::Sense::click(),
    );

    if response.clicked() {
        state.toggle(ui);
    }

    let bg_color = if is_open {
        Theme::ACTIVE_ROW
    } else if response.hovered() {
        Theme::HOVER_ROW
    } else {
        egui::Color32::TRANSPARENT
    };

    if bg_color != egui::Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, 4.0, bg_color);
    }

    response.on_hover_cursor(egui::CursorIcon::PointingHand);

    ui.allocate_ui_at_rect(rect, |ui| {
        ui.horizontal_centered(|ui| {
            ui.add_space(10.0);

            let icon_arrow = if is_open { "\u{f078}" } else { "\u{f054}" };
            ui.add_sized(
                [12.0, ui.available_height()],
                egui::Label::new(
                    egui::RichText::new(icon_arrow)
                        .size(11.0)
                        .color(Theme::TEXT_MUTED),
                )
                .selectable(false),
            );

            ui.add_space(15.0);

            let (icon, color) = match task.status {
                TaskStatus::Sleeping => ("\u{f017}", Theme::STATUS_SLEEPING),
                TaskStatus::Pending => ("\u{f017}", Theme::STATUS_PENDING),
                TaskStatus::Running => ("", Theme::STATUS_RUNNING),
                TaskStatus::Success => ("\u{f058}", Theme::STATUS_SUCCESS),
                TaskStatus::Failed => ("\u{f057}", Theme::STATUS_FAILED),
            };

            if task.status == TaskStatus::Running {
                ctx.request_repaint();
                ui.add_sized(
                    [16.0, ui.available_height()],
                    egui::Spinner::new().size(14.0).color(color),
                );
            } else {
                ui.add_sized(
                    [16.0, ui.available_height()],
                    egui::Label::new(egui::RichText::new(icon).size(14.0).color(color))
                        .selectable(false),
                );
            }

            ui.add_space(5.0);
            ui.add(
                egui::Label::new(
                    egui::RichText::new(&task.name)
                        .size(14.0)
                        .color(Theme::TEXT_LIGHT_GRAY),
                )
                .selectable(false),
            );
        });
    });

    state.show_body_unindented(ui, |ui| {
        egui::Frame::none()
            .inner_margin(egui::Margin {
                left: 47.0,
                right: 10.0,
                top: 5.0,
                bottom: 15.0,
            })
            .show(ui, |ui| {
                if task.logs.is_empty() {
                    ui.label(
                        egui::RichText::new("No hay logs disponibles.")
                            .color(Theme::TEXT_DARK_GRAY)
                            .monospace()
                            .size(12.0),
                    );
                } else {
                    for log in &task.logs {
                        ui.label(
                            egui::RichText::new(log)
                                .color(Theme::TEXT_MUTED)
                                .monospace()
                                .size(12.0),
                        );
                    }
                }
            });
    });
}
