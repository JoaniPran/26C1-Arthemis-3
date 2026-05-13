use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_maximized(true)
            .with_inner_size([1920.0, 1080.0])
            .with_title("Arthemis Orchestrator v1.0.0"),
        ..Default::default()
    };

    eframe::run_native(
        "Arthemis UI",
        options,
        Box::new(|cc| Box::new(ArthemisApp::new(cc))),
    )
}

#[derive(PartialEq)]
enum AppView {
    Projects,
    Workflows,
    Settings,
}

struct ArthemisApp {
    is_maximized: bool,
    current_view: AppView,
    mock_workflows: Vec<String>,
    selected_workflow: Option<String>,
}

impl ArthemisApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Self::default()
    }
}

impl Default for ArthemisApp {
    fn default() -> Self {
        Self {
            is_maximized: false,
            current_view: AppView::Workflows,
            mock_workflows: vec![
                "Main Build".to_string(),
                "Pipeline_Ejemplo".to_string(),
                "Release Flow".to_string(),
            ],
            selected_workflow: None,
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "texto_inter".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/Inter-VariableFont_opsz,wght.ttf")),
    );
    fonts.font_data.insert(
        "terminal_mono".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../assets/GoogleSansCode-VariableFont_MONO,wght.ttf"
        )),
    );
    fonts.font_data.insert(
        "iconos_nerd".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/SymbolsNerdFont-Regular.ttf")),
    );

    let proportional = fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default();
    proportional.insert(0, "texto_inter".to_owned());
    proportional.push("iconos_nerd".to_owned());

    let monospace = fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default();
    monospace.insert(0, "terminal_mono".to_owned());
    monospace.push("iconos_nerd".to_owned());

    ctx.set_fonts(fonts);
}

impl eframe::App for ArthemisApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.is_maximized {
            self.is_maximized = true;
        }

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
                            self.current_view == AppView::Projects,
                            egui::RichText::new(icon_folder).size(icon_size),
                        )
                        .on_hover_text("Proyectos")
                        .clicked()
                    {
                        self.current_view = AppView::Projects;
                    }
                    ui.add_space(20.0);
                    if ui
                        .selectable_label(
                            self.current_view == AppView::Workflows,
                            egui::RichText::new(icon_branch).size(icon_size),
                        )
                        .on_hover_text("Workflows")
                        .clicked()
                    {
                        self.current_view = AppView::Workflows;
                    }
                    ui.add_space(20.0);
                    if ui
                        .selectable_label(
                            self.current_view == AppView::Settings,
                            egui::RichText::new(icon_gear).size(icon_size),
                        )
                        .on_hover_text("Configuración")
                        .clicked()
                    {
                        self.current_view = AppView::Settings;
                    }
                });
            });

        egui::SidePanel::left("content_panel")
            .resizable(true)
            .default_width(220.0)
            .width_range(220.0..=250.0)
            .show(ctx, |ui| {
                ui.add_space(15.0);
                match self.current_view {
                    AppView::Workflows => {
                        ui.horizontal(|ui| {

                            let icon_plus = "\u{200B}\u{f067}\u{200B}";

                            ui.heading("Workflows");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button(egui::RichText::new(icon_plus).size(18.0)).clicked() {
                                        println!("Importar YAML");
                                    }
                                },
                            );
                        });
                        
                        ui.add_space(5.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                            ui.spacing_mut().item_spacing.y = 8.0;

                            for wf in &self.mock_workflows {
                                let is_selected = self.selected_workflow.as_ref() == Some(wf);
                                let label_text = egui::RichText::new(format!("  {}", wf)).size(14.0);
                                
                                if ui
                                    .selectable_label(is_selected, label_text)
                                    .clicked()
                                {
                                    self.selected_workflow = Some(wf.clone());
                                }
                            }
                        });
                    }
                    _ => {
                        ui.label("Vista en desarrollo...");
                    }
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(wf) = &self.selected_workflow {
                ui.heading(format!("Estado de las Tareas - {}", wf));
                ui.add_space(20.0);
                ui.separator();
                ui.label(egui::RichText::new("Esperando logs...").monospace());
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Selecciona un workflow.");
                });
            }
        });
    }
}
