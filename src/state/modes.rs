use crate::config::{AudioConfig, Config};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Blanked,
    Settings,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub mode: AppMode,
    pub config: Config,
    pub config_loaded: bool,
    pub unsaved_changes: bool,
    pub audio_error: Option<String>,
}

impl AppState {
    pub fn new(config: Config, config_loaded: bool) -> Self {
        Self {
            mode: if config_loaded {
                AppMode::Blanked
            } else {
                AppMode::Settings
            },
            config,
            config_loaded,
            unsaved_changes: false,
            audio_error: None,
        }
    }

    pub fn switch_to_settings(&mut self) {
        if self.mode == AppMode::Blanked {
            self.mode = AppMode::Settings;
        }
    }

    pub fn switch_to_blank(&mut self) {
        if self.mode == AppMode::Settings {
            self.mode = AppMode::Blanked;
        }
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            AppMode::Blanked => AppMode::Settings,
            AppMode::Settings => AppMode::Blanked,
        };
    }

    pub fn get_audio_config(&self) -> &AudioConfig {
        &self.config.audio
    }

    pub fn get_audio_config_mut(&mut self) -> &mut AudioConfig {
        &mut self.config.audio
    }

    pub fn mark_unsaved(&mut self) {
        self.unsaved_changes = true;
    }

    pub fn mark_saved(&mut self) {
        self.unsaved_changes = false;
    }

    pub fn set_audio_error(&mut self, msg: String) {
        self.audio_error = Some(msg);
    }

    pub fn clear_audio_error(&mut self) {
        self.audio_error = None;
    }
}
