use crate::audio::AudioEngineHandle;
use crate::config::types::Config;
use crate::config::{load_config, save_config};
use crate::state::AppState;
use crate::ui::screens::{BlankedScreen, SettingsScreen};
use crate::ui::theme::setup_style;
use eframe::egui;
use std::sync::Mutex;

pub struct GooblerApp {
    state: AppState,
    audio_handle: Option<AudioEngineHandle>,
    blanked_screen: BlankedScreen,
    settings_screen: SettingsScreen,
    previous_output_device: Option<String>,
    previous_cursor_visible: bool,
}

impl GooblerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Apply custom theme
        setup_style(&cc.egui_ctx);

        // Load or create config
        let (config, config_loaded) = match load_config() {
            Some(loaded_config) => {
                println!("Loaded config from disk");
                (loaded_config, true)
            }
            None => {
                println!("No config found, using defaults");
                (Config::default(), false)
            }
        };

        // Create app state
        let mut state = AppState::new(config, config_loaded);

        // Initialize audio engine
        let previous_output_device = state.config.audio.output_device.clone();
        let audio_handle = Self::initialize_audio(&mut state);

        Self {
            state,
            audio_handle,
            blanked_screen: BlankedScreen::new(),
            settings_screen: SettingsScreen::new(true),
            previous_output_device,
            previous_cursor_visible: true,
        }
    }

    fn initialize_audio(state: &mut AppState) -> Option<AudioEngineHandle> {
        use crate::audio::AudioEngine;
        use std::sync::Arc;

        // Get the device name as a string slice
        let device_name = state.config.audio.output_device.as_deref();

        match AudioEngine::with_device(state.config.audio.sample_rate, device_name) {
            Ok(engine) => {
                println!(
                    "Audio engine initialized successfully at {}Hz",
                    state.config.audio.sample_rate
                );
                state.clear_audio_error();
                let engine = Arc::new(Mutex::new(engine));

                // Apply initial config
                {
                    let mut eng = engine.lock().unwrap();
                    eng.apply_config(&state.config.audio);
                    eng.play();
                }

                Some(AudioEngineHandle::new(engine))
            }
            Err(e) => {
                eprintln!("Failed to initialize audio engine: {}", e);
                state.set_audio_error(e);
                None
            }
        }
    }

    fn update_audio_config(&mut self) {
        // Check if output device has changed
        let current_device = self.state.config.audio.output_device.clone();
        if self.previous_output_device != current_device {
            // Re-initialize audio engine with new device
            println!("Output device changed, re-initializing audio engine...");
            self.previous_output_device = current_device.clone();

            // Store whether audio was playing
            let was_playing = self
                .audio_handle
                .as_ref()
                .map(|h| h.is_playing())
                .unwrap_or(false);

            // Re-initialize audio (always starts playing)
            self.audio_handle = Self::initialize_audio(&mut self.state);

            // initialize_audio always calls play(), so pause if it wasn't playing before
            if !was_playing {
                if let Some(handle) = &self.audio_handle {
                    handle.pause();
                }
            }
        } else if let Some(handle) = &self.audio_handle {
            // Just update config if device hasn't changed
            handle.apply_config(&self.state.config.audio);
        }
    }

    fn switch_mode(&mut self) {
        #[cfg(feature = "debug_ui")]
        println!("switch_mode called, current mode: {:?}", self.state.mode);
        match self.state.mode {
            crate::state::AppMode::Blanked => {
                self.state.switch_to_settings();
                // Reset fullscreen flag when leaving blanked mode
                self.blanked_screen.reset_fullscreen();
            }
            crate::state::AppMode::Settings => {
                self.state.switch_to_blank();
                // Ensure audio is playing when entering blanked mode
                if let Some(handle) = &self.audio_handle {
                    let is_playing = handle.is_playing();
                    if !is_playing {
                        handle.play();
                    }
                }
            }
        }
    }
}

impl eframe::App for GooblerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // In blanked mode, continuously re-send cursor hidden to prevent
        // the compositor or fullscreen transition from restoring it.
        // In settings mode, only send on transition (cursor should be visible).
        match self.state.mode {
            crate::state::AppMode::Blanked => {
                ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(false));
                self.previous_cursor_visible = false;
            }
            crate::state::AppMode::Settings => {
                if !self.previous_cursor_visible {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(true));
                    self.previous_cursor_visible = true;
                }
            }
        }

        // Handle mode switching
        match self.state.mode {
            crate::state::AppMode::Blanked => {
                let should_switch = self.blanked_screen.show(ctx);
                if should_switch {
                    self.switch_mode();
                }
            }
            crate::state::AppMode::Settings => {
                let should_switch = self.settings_screen.show(ctx, &mut self.state);
                // Update audio immediately when in settings mode for real-time feedback
                self.update_audio_config();
                if should_switch {
                    self.switch_mode();
                }
            }
        }

        // In settings mode, repaint continuously for real-time slider feedback.
        // In blanked mode, repaint at a low rate to keep processing input events
        // (ESC key, mouse movement) while minimizing CPU usage.
        if matches!(self.state.mode, crate::state::AppMode::Settings) {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // Only save config if there are unsaved changes
        if self.state.unsaved_changes {
            if let Err(e) = save_config(&self.state.config) {
                eprintln!("Failed to save config on exit: {}", e);
            } else {
                println!("Config saved on exit");
            }
        }
    }
}
