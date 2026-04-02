use crate::audio::{get_output_devices, OutputDevice};
use crate::config::NoiseType;
use crate::state::AppState;
use egui::{CentralPanel, Context, Key, ScrollArea};

#[derive(Debug, Clone, Copy, PartialEq)]
enum MenuItem {
    OutputDevice,
    NoiseType,
    Volume,
    BassBoost,
    LowPassEnabled,
    LowPassFreq,
    HighPassEnabled,
    HighPassFreq,
    SmoothingEnabled,
    SmoothingAmount,
    OverlayEnabled,
    OverlayType,
    OverlayAmount,
    SaveAndExit,
    ExitWithoutSaving,
}

impl MenuItem {
    const ALL: [MenuItem; 15] = [
        MenuItem::OutputDevice,
        MenuItem::NoiseType,
        MenuItem::Volume,
        MenuItem::BassBoost,
        MenuItem::LowPassEnabled,
        MenuItem::LowPassFreq,
        MenuItem::HighPassEnabled,
        MenuItem::HighPassFreq,
        MenuItem::SmoothingEnabled,
        MenuItem::SmoothingAmount,
        MenuItem::OverlayEnabled,
        MenuItem::OverlayType,
        MenuItem::OverlayAmount,
        MenuItem::SaveAndExit,
        MenuItem::ExitWithoutSaving,
    ];

    fn next(&self) -> MenuItem {
        let idx = Self::ALL.iter().position(|&x| x == *self).unwrap();
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    fn prev(&self) -> MenuItem {
        let idx = Self::ALL.iter().position(|&x| x == *self).unwrap();
        Self::ALL[(idx + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

pub struct SettingsScreen {
    current_item: MenuItem,
    audio_playing: bool,
    output_devices_cache: Option<Vec<OutputDevice>>,
}

impl SettingsScreen {
    pub fn new(audio_playing: bool) -> Self {
        Self {
            current_item: MenuItem::NoiseType,
            audio_playing,
            output_devices_cache: None,
        }
    }

    pub fn show(&mut self, ctx: &Context, state: &mut AppState) -> bool {
        let mut should_switch = false;
        let mut should_save = false;
        let mut should_exit = false;
        let mut should_discard = false;

        // Handle keyboard input without cloning the entire input state
        let (esc, down, up, enter, right, left) = ctx.input(|i| {
            (
                i.key_pressed(Key::Escape),
                i.key_pressed(Key::ArrowDown),
                i.key_pressed(Key::ArrowUp),
                i.key_pressed(Key::Enter),
                i.key_pressed(Key::ArrowRight),
                i.key_pressed(Key::ArrowLeft),
            )
        });

        if esc {
            should_save = true;
            should_exit = true;
        }

        if down {
            self.current_item = self.current_item.next();
        }

        if up {
            self.current_item = self.current_item.prev();
        }

        // Handle Enter key for actions and value changes
        if enter {
            match self.current_item {
                MenuItem::SaveAndExit => {
                    should_save = true;
                    should_exit = true;
                }
                MenuItem::ExitWithoutSaving => {
                    should_discard = true;
                    should_exit = true;
                }
                _ => {
                    // Toggle boolean values or cycle through options
                    self.handle_value_change(state, false);
                }
            }
        }

        // Handle arrow keys for value adjustment
        if right {
            self.handle_value_change(state, true);
        }

        if left {
            self.handle_value_change(state, false);
        }

        // Get available output devices (cached to avoid performance issues)
        if self.output_devices_cache.is_none() {
            self.output_devices_cache = Some(get_output_devices().unwrap_or_default());
        }
        let output_devices = self.output_devices_cache.as_ref().unwrap().clone();

        // Collect current values before rendering
        let (
            output_device,
            noise_type,
            volume,
            bass_boost_db,
            low_pass_enabled,
            low_pass_freq,
            high_pass_enabled,
            high_pass_freq,
            smoothing_enabled,
            smoothing_amount,
            overlay_enabled,
            overlay_type,
            overlay_amount,
            unsaved_changes,
        ) = {
            let config = &state.config.audio;
            (
                config.output_device.clone(),
                config.noise_type,
                config.volume,
                config.bass_boost_db,
                config.low_pass_enabled,
                config.low_pass_freq,
                config.high_pass_enabled,
                config.high_pass_freq,
                config.smoothing_enabled,
                config.smoothing_amount,
                config.overlay_enabled,
                config.overlay_type,
                config.overlay_amount,
                state.unsaved_changes,
            )
        };

        let current_item = self.current_item;

        // Render settings UI
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(60.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Noise Generator Settings");
                    ui.add_space(40.0);
                });

                ui.add_space(40.0);

                // Center the options container
                ui.vertical_centered(|ui| {
                    // Fixed width container for options with left-aligned content
                    ui.allocate_ui_with_layout(
                        egui::vec2(500.0, ui.available_height()),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            let is_focused = |item: MenuItem| current_item == item;

                            // Output Device
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.label("Output Device:");
                                ui.add_space(20.0);

                                // Current device text or "Default" if none selected
                                let current_device_display = match &output_device {
                                    Some(device_name) => {
                                        // Check if this is the default device
                                        if let Some(device) =
                                            output_devices.iter().find(|d| d.name == *device_name)
                                        {
                                            if device.is_default {
                                                format!("{} (Default)", device_name)
                                            } else {
                                                device_name.clone()
                                            }
                                        } else {
                                            format!("{} (Not Found)", device_name)
                                        }
                                    }
                                    None => "Default".to_string(),
                                };

                                // Device selector dropdown
                                let mut selected_device = output_device.clone();
                                egui::ComboBox::from_id_source("output_device_selector")
                                    .selected_text(current_device_display)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut selected_device, None, "Default");
                                        for device in &output_devices {
                                            let display_name = if device.is_default {
                                                format!("{} (Default)", device.name)
                                            } else {
                                                device.name.clone()
                                            };
                                            ui.selectable_value(
                                                &mut selected_device,
                                                Some(device.name.clone()),
                                                display_name,
                                            );
                                        }
                                    });

                                if selected_device != output_device {
                                    let config = state.get_audio_config_mut();
                                    config.output_device = selected_device;
                                    state.mark_unsaved();
                                }
                            });
                            ui.add_space(15.0);

                            ui.separator();
                            ui.add_space(15.0);

                            // Noise Type
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.label("Noise Type:");
                                ui.add_space(20.0);

                                if ui
                                    .selectable_label(
                                        is_focused(MenuItem::NoiseType),
                                        noise_type.display_name(),
                                    )
                                    .clicked()
                                {
                                    if is_focused(MenuItem::NoiseType) {
                                        let config = state.get_audio_config_mut();
                                        self.cycle_noise_type(config);
                                    }
                                }
                            });
                            ui.add_space(15.0);

                            // Volume
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.label("Volume:");
                                ui.add_space(20.0);

                                let volume_text = format!("{:.0}%", volume * 100.0);
                                if ui
                                    .selectable_label(is_focused(MenuItem::Volume), volume_text)
                                    .clicked()
                                {
                                    if is_focused(MenuItem::Volume) {
                                        let config = state.get_audio_config_mut();
                                        config.volume = (config.volume + 0.1).min(1.0);
                                        state.mark_unsaved();
                                    }
                                }

                                // Volume slider
                                let mut vol = volume;
                                if ui
                                    .add_sized(
                                        [300.0, 25.0],
                                        egui::Slider::new(&mut vol, 0.0..=1.0).text("percentage"),
                                    )
                                    .changed()
                                {
                                    let config = state.get_audio_config_mut();
                                    config.volume = vol;
                                    state.mark_unsaved();
                                }
                            });
                            ui.add_space(15.0);

                            // Bass Boost
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.label("Bass Boost:");
                                ui.add_space(20.0);

                                let bass_text = if bass_boost_db > 0.0 {
                                    format!("{:.1} dB", bass_boost_db)
                                } else {
                                    "Off".to_string()
                                };

                                if ui
                                    .selectable_label(is_focused(MenuItem::BassBoost), &bass_text)
                                    .clicked()
                                {
                                    if is_focused(MenuItem::BassBoost) {
                                        let config = state.get_audio_config_mut();
                                        config.bass_boost_db =
                                            (config.bass_boost_db + 0.5).clamp(-6.0, 12.0);
                                        state.mark_unsaved();
                                    }
                                }

                                // Bass boost slider
                                let mut bass = bass_boost_db;
                                if ui
                                    .add_sized(
                                        [300.0, 25.0],
                                        egui::Slider::new(&mut bass, -6.0..=12.0).text("dB"),
                                    )
                                    .changed()
                                {
                                    let config = state.get_audio_config_mut();
                                    config.bass_boost_db = bass;
                                    state.mark_unsaved();
                                }
                            });
                            ui.add_space(15.0);

                            ui.separator();
                            ui.add_space(15.0);

                            // Low Pass Filter
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.label("Low-Pass Filter:");
                                ui.add_space(20.0);

                                let low_pass_text = if low_pass_enabled {
                                    format!("{:.0} Hz", low_pass_freq)
                                } else {
                                    "Off".to_string()
                                };

                                // Enable/disable checkbox
                                let mut enabled = low_pass_enabled;
                                if ui.checkbox(&mut enabled, "").changed() {
                                    let config = state.get_audio_config_mut();
                                    config.low_pass_enabled = enabled;
                                    state.mark_unsaved();
                                }

                                if ui
                                    .selectable_label(
                                        is_focused(MenuItem::LowPassEnabled),
                                        low_pass_text,
                                    )
                                    .clicked()
                                {
                                    if is_focused(MenuItem::LowPassEnabled) {
                                        let config = state.get_audio_config_mut();
                                        config.low_pass_enabled = !config.low_pass_enabled;
                                        state.mark_unsaved();
                                    }
                                }
                            });
                            ui.add_space(15.0);

                            // Low Pass Frequency
                            if low_pass_enabled {
                                ui.horizontal(|ui| {
                                    ui.add_space(40.0);
                                    ui.label("  Frequency:");
                                    ui.add_space(20.0);

                                    // Frequency slider
                                    let mut freq = low_pass_freq;
                                    if ui
                                        .add_sized(
                                            [200.0, 20.0],
                                            egui::Slider::new(&mut freq, 20.0..=20000.0)
                                                .text("Hz")
                                                .logarithmic(true),
                                        )
                                        .changed()
                                    {
                                        let config = state.get_audio_config_mut();
                                        config.low_pass_freq = freq;
                                        state.mark_unsaved();
                                    }

                                    let freq_text = format!("{:.0} Hz", low_pass_freq);
                                    if ui
                                        .selectable_label(
                                            is_focused(MenuItem::LowPassFreq),
                                            freq_text,
                                        )
                                        .clicked()
                                    {
                                        if is_focused(MenuItem::LowPassFreq) {
                                            let config = state.get_audio_config_mut();
                                            config.low_pass_freq =
                                                (config.low_pass_freq + 100.0).min(20000.0);
                                            state.mark_unsaved();
                                        }
                                    }
                                });
                                ui.add_space(15.0);
                            }

                            ui.separator();
                            ui.add_space(15.0);

                            // High Pass Filter
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.label("High-Pass Filter:");
                                ui.add_space(20.0);

                                let high_pass_text = if high_pass_enabled {
                                    format!("{:.0} Hz", high_pass_freq)
                                } else {
                                    "Off".to_string()
                                };

                                // Enable/disable checkbox
                                let mut enabled = high_pass_enabled;
                                if ui.checkbox(&mut enabled, "").changed() {
                                    let config = state.get_audio_config_mut();
                                    config.high_pass_enabled = enabled;
                                    state.mark_unsaved();
                                }

                                if ui
                                    .selectable_label(
                                        is_focused(MenuItem::HighPassEnabled),
                                        high_pass_text,
                                    )
                                    .clicked()
                                {
                                    if is_focused(MenuItem::HighPassEnabled) {
                                        let config = state.get_audio_config_mut();
                                        config.high_pass_enabled = !config.high_pass_enabled;
                                        state.mark_unsaved();
                                    }
                                }
                            });
                            ui.add_space(15.0);

                            // High Pass Frequency
                            if high_pass_enabled {
                                ui.horizontal(|ui| {
                                    ui.add_space(40.0);
                                    ui.label("  Frequency:");
                                    ui.add_space(20.0);

                                    // Frequency slider
                                    let mut freq = high_pass_freq;
                                    if ui
                                        .add_sized(
                                            [200.0, 20.0],
                                            egui::Slider::new(&mut freq, 20.0..=1000.0)
                                                .text("Hz")
                                                .logarithmic(true),
                                        )
                                        .changed()
                                    {
                                        let config = state.get_audio_config_mut();
                                        config.high_pass_freq = freq;
                                        state.mark_unsaved();
                                    }

                                    let freq_text = format!("{:.0} Hz", high_pass_freq);
                                    if ui
                                        .selectable_label(
                                            is_focused(MenuItem::HighPassFreq),
                                            freq_text,
                                        )
                                        .clicked()
                                    {
                                        if is_focused(MenuItem::HighPassFreq) {
                                            let config = state.get_audio_config_mut();
                                            config.high_pass_freq =
                                                (config.high_pass_freq + 10.0).min(1000.0);
                                            state.mark_unsaved();
                                        }
                                    }
                                });
                                ui.add_space(15.0);
                            }

                            ui.separator();
                            ui.add_space(30.0);

                            // Smoothing Section
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.label("Smoothing:");
                                ui.add_space(20.0);

                                // Enable/disable checkbox
                                let mut enabled = smoothing_enabled;
                                if ui.checkbox(&mut enabled, "").changed() {
                                    let config = state.get_audio_config_mut();
                                    config.smoothing_enabled = enabled;
                                    state.mark_unsaved();
                                }

                                let smoothing_text = if smoothing_enabled {
                                    "Enabled".to_string()
                                } else {
                                    "Off".to_string()
                                };

                                if ui
                                    .selectable_label(
                                        is_focused(MenuItem::SmoothingEnabled),
                                        smoothing_text,
                                    )
                                    .clicked()
                                {
                                    if is_focused(MenuItem::SmoothingEnabled) {
                                        let config = state.get_audio_config_mut();
                                        config.smoothing_enabled = !config.smoothing_enabled;
                                        state.mark_unsaved();
                                    }
                                }
                            });
                            ui.add_space(15.0);

                            // Smoothing Amount
                            if smoothing_enabled {
                                ui.horizontal(|ui| {
                                    ui.add_space(40.0);
                                    ui.label("  Amount:");
                                    ui.add_space(20.0);

                                    // Smoothing slider
                                    let mut amount = smoothing_amount;
                                    if ui
                                        .add_sized(
                                            [300.0, 25.0],
                                            egui::Slider::new(&mut amount, 0.0..=1.0)
                                                .text("intensity"),
                                        )
                                        .changed()
                                    {
                                        let config = state.get_audio_config_mut();
                                        config.smoothing_amount = amount;
                                        state.mark_unsaved();
                                    }

                                    let amount_text = format!("{:.0}%", smoothing_amount * 100.0);
                                    if ui
                                        .selectable_label(
                                            is_focused(MenuItem::SmoothingAmount),
                                            amount_text,
                                        )
                                        .clicked()
                                    {
                                        if is_focused(MenuItem::SmoothingAmount) {
                                            let config = state.get_audio_config_mut();
                                            config.smoothing_amount =
                                                (config.smoothing_amount + 0.1).min(1.0);
                                            state.mark_unsaved();
                                        }
                                    }
                                });
                                ui.add_space(15.0);
                            }

                            ui.separator();
                            ui.add_space(15.0);

                            // Noise Overlay Section
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.label("Noise Overlay:");
                                ui.add_space(20.0);

                                // Enable/disable checkbox
                                let mut enabled = overlay_enabled;
                                if ui.checkbox(&mut enabled, "").changed() {
                                    let config = state.get_audio_config_mut();
                                    config.overlay_enabled = enabled;
                                    state.mark_unsaved();
                                }

                                let overlay_text = if overlay_enabled {
                                    "Enabled".to_string()
                                } else {
                                    "Off".to_string()
                                };

                                if ui
                                    .selectable_label(
                                        is_focused(MenuItem::OverlayEnabled),
                                        overlay_text,
                                    )
                                    .clicked()
                                {
                                    if is_focused(MenuItem::OverlayEnabled) {
                                        let config = state.get_audio_config_mut();
                                        config.overlay_enabled = !config.overlay_enabled;
                                        state.mark_unsaved();
                                    }
                                }
                            });
                            ui.add_space(15.0);

                            // Overlay Type
                            if overlay_enabled {
                                ui.horizontal(|ui| {
                                    ui.add_space(40.0);
                                    ui.label("  Type:");
                                    ui.add_space(20.0);

                                    if ui
                                        .selectable_label(
                                            is_focused(MenuItem::OverlayType),
                                            overlay_type.display_name(),
                                        )
                                        .clicked()
                                    {
                                        if is_focused(MenuItem::OverlayType) {
                                            let config = state.get_audio_config_mut();
                                            self.cycle_overlay_type(config);
                                        }
                                    }
                                });
                                ui.add_space(15.0);
                            }

                            // Overlay Amount
                            if overlay_enabled {
                                ui.horizontal(|ui| {
                                    ui.add_space(40.0);
                                    ui.label("  Mix Amount:");
                                    ui.add_space(20.0);

                                    // Overlay mix slider
                                    let mut amount = overlay_amount;
                                    if ui
                                        .add_sized(
                                            [300.0, 25.0],
                                            egui::Slider::new(&mut amount, 0.0..=1.0)
                                                .text("percentage"),
                                        )
                                        .changed()
                                    {
                                        let config = state.get_audio_config_mut();
                                        config.overlay_amount = amount;
                                        state.mark_unsaved();
                                    }

                                    let amount_text = format!("{:.0}%", overlay_amount * 100.0);
                                    if ui
                                        .selectable_label(
                                            is_focused(MenuItem::OverlayAmount),
                                            amount_text,
                                        )
                                        .clicked()
                                    {
                                        if is_focused(MenuItem::OverlayAmount) {
                                            let config = state.get_audio_config_mut();
                                            config.overlay_amount =
                                                (config.overlay_amount + 0.1).min(1.0);
                                            state.mark_unsaved();
                                        }
                                    }
                                });
                                ui.add_space(15.0);
                            }

                            ui.separator();
                            ui.add_space(30.0);

                            // Buttons
                            ui.add_space(20.0);

                            let save_button = if unsaved_changes {
                                "Save & Exit *".to_string()
                            } else {
                                "Save & Exit".to_string()
                            };

                            if ui.button(save_button).clicked() {
                                should_save = true;
                                should_exit = true;
                            }

                            ui.add_space(15.0);

                            if ui.button("Exit Without Saving").clicked() {
                                should_discard = true;
                                should_exit = true;
                            }

                            ui.add_space(40.0);

                            // Instructions
                            ui.vertical_centered(|ui| {
                                ui.label("Navigation:");
                                ui.label("  ↑/↓: Navigate menu");
                                ui.label("  ←/→: Adjust values");
                                ui.label("  Enter: Toggle/Select");
                                ui.label("  Escape: Save & Exit");
                                ui.label("  Mouse: Use sliders, checkboxes, and buttons");
                            });
                        },
                    );
                });
            });
        });

        // Handle actions
        if should_discard {
            // Reload config from disk to discard unsaved changes
            if let Some(saved_config) = crate::config::load_config() {
                state.config = saved_config;
            }
            state.mark_saved();
        }

        if should_save {
            if let Err(e) = crate::config::save_config(&state.config) {
                eprintln!("Failed to save config: {}", e);
            } else {
                state.mark_saved();
            }
        }

        if should_exit {
            should_switch = true;
        }

        should_switch
    }

    fn handle_value_change(&mut self, state: &mut AppState, increase: bool) {
        let config = state.get_audio_config_mut();
        let step = if increase { 1 } else { -1 };

        match self.current_item {
            MenuItem::NoiseType => {
                self.cycle_noise_type(config);
                state.mark_unsaved();
            }
            MenuItem::Volume => {
                config.volume = (config.volume + (step as f32 * 0.1)).clamp(0.0, 1.0);
                state.mark_unsaved();
            }
            MenuItem::BassBoost => {
                config.bass_boost_db =
                    (config.bass_boost_db + (step as f32 * 0.5)).clamp(-6.0, 12.0);
                state.mark_unsaved();
            }
            MenuItem::LowPassEnabled => {
                config.low_pass_enabled = !config.low_pass_enabled;
                state.mark_unsaved();
            }
            MenuItem::LowPassFreq => {
                config.low_pass_freq =
                    (config.low_pass_freq + (step as f32 * 100.0)).clamp(20.0, 20000.0);
                state.mark_unsaved();
            }
            MenuItem::HighPassEnabled => {
                config.high_pass_enabled = !config.high_pass_enabled;
                state.mark_unsaved();
            }
            MenuItem::HighPassFreq => {
                config.high_pass_freq =
                    (config.high_pass_freq + (step as f32 * 10.0)).clamp(20.0, 1000.0);
                state.mark_unsaved();
            }
            MenuItem::SmoothingEnabled => {
                config.smoothing_enabled = !config.smoothing_enabled;
                state.mark_unsaved();
            }
            MenuItem::SmoothingAmount => {
                config.smoothing_amount =
                    (config.smoothing_amount + (step as f32 * 0.1)).clamp(0.0, 1.0);
                state.mark_unsaved();
            }
            MenuItem::OverlayEnabled => {
                config.overlay_enabled = !config.overlay_enabled;
                state.mark_unsaved();
            }
            MenuItem::OverlayType => {
                self.cycle_overlay_type(config);
                state.mark_unsaved();
            }
            MenuItem::OverlayAmount => {
                config.overlay_amount =
                    (config.overlay_amount + (step as f32 * 0.1)).clamp(0.0, 1.0);
                state.mark_unsaved();
            }
            _ => {}
        }
    }

    fn cycle_noise_type(&mut self, config: &mut crate::config::AudioConfig) {
        let current_idx = NoiseType::ALL
            .iter()
            .position(|&x| x == config.noise_type)
            .unwrap();
        let next_idx = (current_idx + 1) % NoiseType::ALL.len();
        config.noise_type = NoiseType::ALL[next_idx];
    }

    fn cycle_overlay_type(&mut self, config: &mut crate::config::AudioConfig) {
        let current_idx = NoiseType::ALL
            .iter()
            .position(|&x| x == config.overlay_type)
            .unwrap();
        let next_idx = (current_idx + 1) % NoiseType::ALL.len();
        config.overlay_type = NoiseType::ALL[next_idx];
    }
}
