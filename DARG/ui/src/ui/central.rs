use crate::app::{AppView, CoreState, UiState};
use eframe::egui;

pub fn draw(ctx: &egui::Context, ui_state: &UiState, core_state: &CoreState) {
    egui::CentralPanel::default().show(ctx, |ui| match ui_state.current_view {
        AppView::Workflows => {
            if let Some(wf) = &core_state.selected_workflow {
                ui.heading(format!("Estado de las Tareas - {}", wf));
                ui.add_space(20.0);
                ui.separator();
                ui.label(egui::RichText::new("Esperando logs...").monospace());
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Selecciona un workflow del explorador para ver sus detalles.");
                });
            }
        }

        AppView::Projects => {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Panel Principal de Proyectos");
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("Aquí visualizaremos métricas a nivel entorno.")
                            .color(egui::Color32::GRAY),
                    );
                });
            });
        }

        AppView::Settings => {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Configuración del Clúster");
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("Aquí irán las opciones de los Workers y red.")
                            .color(egui::Color32::GRAY),
                    );
                });
            });
        }
    });
}
