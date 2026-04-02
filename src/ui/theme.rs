use egui::{Color32, FontId, Visuals};

pub fn setup_style(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    // Customize colors
    visuals.panel_fill = Color32::from_rgb(20, 20, 20);
    visuals.window_fill = Color32::from_rgb(25, 25, 25);
    visuals.faint_bg_color = Color32::from_rgb(30, 30, 30);

    // Selection colors
    visuals.selection.bg_fill = Color32::from_rgb(60, 120, 200);
    visuals.selection.stroke = egui::Stroke::new(2.0, Color32::from_rgb(80, 140, 220));

    // Override text color
    visuals.override_text_color = Some(Color32::from_rgb(220, 220, 220));

    // Widgets
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(30, 30, 30);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(35, 35, 35);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(45, 45, 45);
    visuals.widgets.active.bg_fill = Color32::from_rgb(55, 55, 55);

    ctx.set_visuals(visuals);

    // Font sizes
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (egui::TextStyle::Heading, FontId::proportional(28.0)),
        (egui::TextStyle::Body, FontId::proportional(18.0)),
        (egui::TextStyle::Monospace, FontId::monospace(16.0)),
        (egui::TextStyle::Button, FontId::proportional(18.0)),
        (egui::TextStyle::Small, FontId::proportional(14.0)),
    ]
    .into();
    ctx.set_style(style);
}
