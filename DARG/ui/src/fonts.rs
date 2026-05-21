use eframe::egui;

pub fn setup_custom_fonts(ctx: &egui::Context) {
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
