use eframe::egui::Color32;

#[derive(Clone, Copy)]
pub struct Palette {
    pub background: Color32,
    pub toolbar: Color32,
    pub foreground: Color32,
    pub secondary: Color32,
    pub accent: Color32,
    pub divider: Color32,
    pub selection: Color32,
    pub current_line: Color32,
    pub danger: Color32,
    pub on_danger: Color32,
}
