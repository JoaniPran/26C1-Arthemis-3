use crate::app::{AppView, CoreState, UiState};
use eframe::egui;

pub fn draw(ctx: &egui::Context, ui_state: &UiState, core_state: &mut CoreState) {
    if ui_state.current_view == AppView::Workflows {
        egui::SidePanel::left("content_panel")
            .resizable(true)
            .default_width(220.0)
            .width_range(220.0..=350.0)
            .show(ctx, |ui| {
                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    let icon_plus = "\u{200B}\u{f067}\u{200B}";
                    ui.heading("Workflows");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(egui::RichText::new(icon_plus).size(18.0))
                            .clicked()
                        {
                            println!("Importar YAML");
                        }
                    });
                });

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(10.0);

                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;

                    for wf in &core_state.workflows {
                        let is_selected = core_state.selected_workflow.as_ref() == Some(wf);
                        let label_text = egui::RichText::new(format!("  {}", wf)).size(14.0);

                        if ui.selectable_label(is_selected, label_text).clicked() {
                            core_state.selected_workflow = Some(wf.clone());
                        }
                    }
                });
            });
    }
}
