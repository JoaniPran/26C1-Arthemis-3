use eframe::egui::Color32;

pub struct Theme;

impl Theme {
    pub const BG_APP: Color32 = Color32::from_rgb(26, 26, 26);
    pub const BG_WINDOW: Color32 = Color32::from_rgb(30, 30, 30);
    pub const BG_SIDEBAR: Color32 = Color32::from_rgb(23, 23, 23);
    pub const BG_CENTRAL_PANEL: Color32 = Color32::from_rgb(14, 16, 20);

    pub const ACCENT_BLUE: Color32 = Color32::from_rgb(45, 90, 180);
    pub const SELECTION_BG: Color32 = Color32::from_rgb(35, 75, 150);

    pub const STATUS_SUCCESS: Color32 = Color32::from_gray(140);
    pub const STATUS_FAILED: Color32 = Color32::LIGHT_RED;
    pub const STATUS_PENDING: Color32 = Color32::from_gray(140);
    pub const STATUS_SLEEPING: Color32 = Color32::from_gray(110);
    pub const STATUS_RUNNING: Color32 = Color32::from_gray(140);

    pub const TEXT_WHITE: Color32 = Color32::WHITE;
    pub const TEXT_MUTED: Color32 = Color32::from_gray(140);
    pub const TEXT_LIGHT_GRAY: Color32 = Color32::from_gray(220);
    pub const TEXT_DARK_GRAY: Color32 = Color32::from_gray(120);

    pub const BORDER_DARK: Color32 = Color32::from_rgb(40, 40, 40);
    pub const BORDER_LIGHT: Color32 = Color32::from_rgb(45, 50, 55);

    pub const HOVER_ROW: Color32 = Color32::from_rgb(28, 32, 38);
    pub const ACTIVE_ROW: Color32 = Color32::from_rgb(45, 50, 55);
    pub const HOVER_FILE: Color32 = Color32::from_rgb(35, 35, 35);

    pub const DROPZONE_IDLE_BG: Color32 = Color32::from_rgb(24, 24, 24);
    pub const DROPZONE_IDLE_STROKE: Color32 = Color32::from_gray(60);
    pub const DROPZONE_HOVER_BG: Color32 = Color32::from_rgb(20, 30, 45);
    pub const DROPZONE_HOVER_STROKE: Color32 = Color32::from_rgb(80, 140, 255);
    pub const OVERLAY_BG: Color32 = Color32::from_black_alpha(170);
}
