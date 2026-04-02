#![warn(clippy::all, rust_2018_idioms)]

mod audio;
mod config;
mod state;
mod ui;

use eframe::egui;
use ui::GooblerApp;

fn main() -> eframe::Result<()> {
    // Configure window options
    let mut window_options = eframe::NativeOptions::default();
    window_options.viewport = egui::ViewportBuilder::default()
        .with_inner_size([500.0, 700.0])
        .with_min_inner_size([400.0, 500.0])
        .with_icon(load_icon());

    // Run the application
    eframe::run_native(
        "Goobler - Noise Generator",
        window_options,
        Box::new(|cc| Ok(Box::new(GooblerApp::new(cc)))),
    )
}

fn load_icon() -> egui::IconData {
    // Simple placeholder icon (16x16 pixel data)
    let (icon_rgba, icon_width, icon_height) = {
        let width = 64;
        let height = 64;
        let mut pixels = vec![0; width * height * 4];

        // Create a simple icon with a G shape
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;

                // Background (dark)
                pixels[idx] = 20; // R
                pixels[idx + 1] = 20; // G
                pixels[idx + 2] = 20; // B
                pixels[idx + 3] = 255; // A

                // Draw G shape
                let center_x = width / 2;
                let center_y = height / 2;
                let dx = (x as f32 - center_x as f32) / (width as f32 / 2.0);
                let dy = (y as f32 - center_y as f32) / (height as f32 / 2.0);
                let distance = (dx * dx + dy * dy).sqrt();

                if distance < 0.7 && distance > 0.5 {
                    pixels[idx] = 100;
                    pixels[idx + 1] = 160;
                    pixels[idx + 2] = 240;
                }

                // Add opening for G
                if distance < 0.6 && distance > 0.5 && dx > 0.0 && dy.abs() < 0.2 {
                    pixels[idx] = 20;
                    pixels[idx + 1] = 20;
                    pixels[idx + 2] = 20;
                }

                // Add horizontal bar for G
                if dy > -0.1 && dy < 0.1 && dx < 0.0 && distance < 0.6 && distance > 0.3 {
                    pixels[idx] = 100;
                    pixels[idx + 1] = 160;
                    pixels[idx + 2] = 240;
                }
            }
        }

        (pixels, width, height)
    };

    egui::IconData {
        rgba: icon_rgba,
        width: icon_width as u32,
        height: icon_height as u32,
    }
}
