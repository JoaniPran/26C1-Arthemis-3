use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Arthemis Orchestrator"),
        ..Default::default()
    };

    eframe::run_native(
        "Arthemis UI",
        options,
        Box::new(|_cc| Box::new(ArthemisApp::default())),
    )
}

struct ArthemisApp {
    app_version: String,
}

impl Default for ArthemisApp {
    fn default() -> Self {
        Self {
            app_version: "1.0.0".to_owned(),
        }
    }
}

impl eframe::App for ArthemisApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::CentralPanel::default().show(ctx, |ui| {
            
            ui.heading("Panel de Control - Arthemis");
            
            ui.label(format!("Versión: {}", self.app_version));

            ui.separator();

            if ui.button("Haz clic para saludar").clicked() {
                println!("¡Hola desde la consola de la UI!");
            }
        });
    }
}