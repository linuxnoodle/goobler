use egui::{CentralPanel, Context};

pub struct BlankedScreen {
    fullscreen_set: bool,
}

impl BlankedScreen {
    pub fn new() -> Self {
        Self {
            fullscreen_set: false,
        }
    }

    pub fn show(&mut self, ctx: &Context) -> bool {
        // Check for ESC key without cloning the entire input state
        let should_switch = ctx.input(|i| i.key_pressed(egui::Key::Escape));

        // Render fully black screen
        CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                // Completely black background
                ui.painter()
                    .rect_filled(ui.clip_rect(), 0.0, egui::Color32::BLACK);
            });
        });

        // Request fullscreen only once when entering blanked mode
        if !self.fullscreen_set {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            self.fullscreen_set = true;
        }

        should_switch
    }

    pub fn reset_fullscreen(&mut self) {
        self.fullscreen_set = false;
    }
}
