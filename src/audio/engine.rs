use crate::audio::processor::ProcessingChain;
use crate::config::{AudioConfig, NoiseType};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub struct AudioEngine {
    _stream: cpal::Stream,
    processing_chain: Arc<Mutex<ProcessingChain>>,
    paused: Arc<AtomicBool>,
}

impl AudioEngine {
    pub fn new(sample_rate: u32) -> Result<Self, String> {
        Self::with_device(sample_rate, None)
    }

    /// Pick the best sample rate for a device.
    /// Returns the configured rate if the device supports it, otherwise the device's default rate.
    fn pick_sample_rate(device: &cpal::Device, configured_rate: u32) -> u32 {
        let Ok(configs) = device.supported_output_configs() else {
            return configured_rate;
        };

        let mut best_match: Option<u32> = None;
        let mut default_rate: Option<u32> = None;

        for config in configs {
            let rate = config.min_sample_rate().0;

            // Prefer exact match (check full range)
            if configured_rate >= config.min_sample_rate().0
                && configured_rate <= config.max_sample_rate().0
            {
                return configured_rate;
            }

            // Track the first (default) config's rate as fallback
            if default_rate.is_none() {
                default_rate = Some(rate);
            }

            // Track closest rate as second fallback
            if best_match.is_none()
                || (rate as i32 - configured_rate as i32).abs()
                    < (best_match.unwrap() as i32 - configured_rate as i32).abs()
            {
                best_match = Some(rate);
            }
        }

        // Prefer the device's default config rate, then closest match, then configured rate
        default_rate
            .or(best_match)
            .unwrap_or(configured_rate)
    }

    /// Find a supported stream config range that uses f32 sample format with stereo channels.
    fn find_supported_config_range(
        device: &cpal::Device,
    ) -> Result<cpal::SupportedStreamConfigRange, String> {
        let configs: Vec<_> = device
            .supported_output_configs()
            .map_err(|e| format!("Failed to query device configs: {}", e))?
            .collect();

        // Prefer f32 stereo configs
        if let Some(config) = configs.iter().find(|c| {
            c.sample_format() == cpal::SampleFormat::F32 && c.channels() == 2
        }) {
            return Ok(config.clone());
        }

        // Fall back to f32 with any channel count
        if let Some(config) = configs
            .iter()
            .find(|c| c.sample_format() == cpal::SampleFormat::F32)
        {
            return Ok(config.clone());
        }

        // Fall back to default config range
        Err("No f32 config found".to_string())
    }

    pub fn with_device(sample_rate: u32, output_device: Option<&str>) -> Result<Self, String> {
        let _stderr_guard = crate::audio::StderrGuard::new();

        let (device, device_name) = if let Some(name) = output_device {
            let host = cpal::default_host();
            let devices: Vec<_> = host
                .output_devices()
                .map_err(|e| format!("Failed to get output devices: {}", e))?
                .collect();

            let target = devices
                .iter()
                .find(|d| d.name().as_ref().ok().map(|n| n.as_str()) == Some(name))
                .ok_or_else(|| format!("Device '{}' not found", name))?;

            (target.clone(), name.to_string())
        } else {
            let host = cpal::default_host();
            let dev = host
                .default_output_device()
                .ok_or("No default output device")?;
            let name = dev.name().unwrap_or_default();
            (dev, name)
        };

        let actual_rate = Self::pick_sample_rate(&device, sample_rate);

        let supported_config_range = Self::find_supported_config_range(&device)?;
        let config: cpal::StreamConfig = supported_config_range
            .with_sample_rate(cpal::SampleRate(actual_rate))
            .into();

        let processing_chain = Arc::new(Mutex::new(ProcessingChain::new(actual_rate)));
        let paused = Arc::new(AtomicBool::new(true)); // Start paused

        let chain_ref = Arc::clone(&processing_chain);
        let paused_ref = Arc::clone(&paused);
        let channels = config.channels;

        let stream = device
            .build_output_stream::<f32, _, _>(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    if paused_ref.load(Ordering::Relaxed) {
                        for sample in data.iter_mut() {
                            *sample = 0.0;
                        }
                        return;
                    }

                    if let Ok(mut chain) = chain_ref.lock() {
                        if channels == 2 {
                            for chunk in data.chunks_exact_mut(2) {
                                let (smoothing, overlay, _, overlay_amount) =
                                    chain.get_smoothing_config();
                                let (left, right) =
                                    chain.process_stereo(smoothing, overlay, overlay_amount);
                                chunk[0] = left;
                                chunk[1] = right;
                            }
                        } else {
                            // Mono or other channel counts: generate stereo, take first channel
                            for sample in data.iter_mut() {
                                let (smoothing, overlay, _, overlay_amount) =
                                    chain.get_smoothing_config();
                                let (left, _) =
                                    chain.process_stereo(smoothing, overlay, overlay_amount);
                                *sample = left;
                            }
                        }
                    } else {
                        for sample in data.iter_mut() {
                            *sample = 0.0;
                        }
                    }
                },
                move |err| {
                    eprintln!("Audio output error on '{}': {}", device_name, err);
                },
                None,
            )
            .map_err(|e| format!("Failed to build output stream: {}", e))?;

        stream.play().map_err(|e| format!("Failed to start output stream: {}", e))?;

        Ok(Self {
            _stream: stream,
            processing_chain,
            paused,
        })
    }

    pub fn apply_config(&mut self, config: &AudioConfig) {
        let mut chain = self.processing_chain.lock().unwrap();

        chain.set_noise_type(config.noise_type.clone());
        chain.set_volume(config.volume);
        chain.set_bass_boost_db(config.bass_boost_db);
        chain.set_bass_boost_enabled(config.bass_boost_db > 0.0);
        chain.set_low_pass_enabled(config.low_pass_enabled);
        chain.set_low_pass_frequency(config.low_pass_freq);
        chain.set_high_pass_enabled(config.high_pass_enabled);
        chain.set_high_pass_frequency(config.high_pass_freq);
        chain.set_smoothing_enabled(config.smoothing_enabled);
        chain.set_smoothing_amount(config.smoothing_amount);
        chain.set_overlay_enabled(config.overlay_enabled);
        chain.set_overlay_type(config.overlay_type);
        chain.set_overlay_amount(config.overlay_amount);
    }

    pub fn update_noise_type(&mut self, noise_type: NoiseType) {
        self.processing_chain
            .lock()
            .unwrap()
            .set_noise_type(noise_type);
    }

    pub fn update_volume(&mut self, volume: f32) {
        self.processing_chain.lock().unwrap().set_volume(volume);
    }

    pub fn update_bass_boost(&mut self, gain_db: f32) {
        self.processing_chain
            .lock()
            .unwrap()
            .set_bass_boost_db(gain_db);
        self.processing_chain
            .lock()
            .unwrap()
            .set_bass_boost_enabled(gain_db > 0.0);
    }

    pub fn update_low_pass(&mut self, enabled: bool, frequency: f32) {
        let mut chain = self.processing_chain.lock().unwrap();
        chain.set_low_pass_enabled(enabled);
        chain.set_low_pass_frequency(frequency);
    }

    pub fn update_high_pass(&mut self, enabled: bool, frequency: f32) {
        let mut chain = self.processing_chain.lock().unwrap();
        chain.set_high_pass_enabled(enabled);
        chain.set_high_pass_frequency(frequency);
    }

    pub fn play(&self) {
        #[cfg(feature = "debug_audio")]
        println!("AudioEngine::play() called");
        self.paused.store(false, Ordering::Relaxed);
    }

    pub fn pause(&self) {
        #[cfg(feature = "debug_audio")]
        println!("AudioEngine::pause() called");
        self.paused.store(true, Ordering::Relaxed);
    }

    pub fn is_playing(&self) -> bool {
        !self.paused.load(Ordering::Relaxed)
    }

    pub fn toggle(&self) {
        let was_paused = self.paused.load(Ordering::Relaxed);
        self.paused.store(!was_paused, Ordering::Relaxed);
    }
}

pub struct AudioEngineHandle {
    engine: Arc<Mutex<AudioEngine>>,
}

impl AudioEngineHandle {
    pub fn new(engine: Arc<Mutex<AudioEngine>>) -> Self {
        Self { engine }
    }

    pub fn apply_config(&self, config: &AudioConfig) {
        if let Ok(mut engine) = self.engine.lock() {
            engine.apply_config(config);
        }
    }

    pub fn update_noise_type(&self, noise_type: NoiseType) {
        if let Ok(mut engine) = self.engine.lock() {
            engine.update_noise_type(noise_type);
        }
    }

    pub fn update_volume(&self, volume: f32) {
        if let Ok(mut engine) = self.engine.lock() {
            engine.update_volume(volume);
        }
    }

    pub fn update_bass_boost(&self, gain_db: f32) {
        if let Ok(mut engine) = self.engine.lock() {
            engine.update_bass_boost(gain_db);
        }
    }

    pub fn update_low_pass(&self, enabled: bool, frequency: f32) {
        if let Ok(mut engine) = self.engine.lock() {
            engine.update_low_pass(enabled, frequency);
        }
    }

    pub fn update_high_pass(&self, enabled: bool, frequency: f32) {
        if let Ok(mut engine) = self.engine.lock() {
            engine.update_high_pass(enabled, frequency);
        }
    }

    pub fn toggle(&self) {
        if let Ok(engine) = self.engine.lock() {
            engine.toggle();
        }
    }

    pub fn play(&self) {
        if let Ok(engine) = self.engine.lock() {
            engine.play();
        }
    }

    pub fn pause(&self) {
        if let Ok(engine) = self.engine.lock() {
            engine.pause();
        }
    }

    pub fn set_playing(&self, playing: bool) {
        if let Ok(engine) = self.engine.lock() {
            if playing {
                engine.play();
            } else {
                engine.pause();
            }
        }
    }

    pub fn is_playing(&self) -> bool {
        if let Ok(engine) = self.engine.lock() {
            engine.is_playing()
        } else {
            false
        }
    }
}
