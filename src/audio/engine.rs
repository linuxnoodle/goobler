use crate::audio::processor::ProcessingChain;
use crate::config::{AudioConfig, NoiseType};
use cpal::traits::{DeviceTrait, HostTrait};
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct AudioEngine {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    sink: Sink,
    processing_chain: Arc<Mutex<ProcessingChain>>,
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

    pub fn with_device(sample_rate: u32, output_device: Option<&str>) -> Result<Self, String> {
        let _stderr_guard = crate::audio::StderrGuard::new();

        let (stream, stream_handle, actual_sample_rate) =
            if let Some(device_name) = output_device {
                // Try to find and use specified device
                let host = cpal::default_host();
                let devices: Vec<_> = host
                    .output_devices()
                    .map_err(|e| format!("Failed to get output devices: {}", e))?
                    .collect();

                let target_device = devices
                    .iter()
                    .find(|d| d.name().as_ref().ok().map(|n| n.as_str()) == Some(device_name))
                    .ok_or_else(|| format!("Device '{}' not found", device_name))?;

                // Query device's supported sample rates and pick the best match
                let device_rate = Self::pick_sample_rate(&target_device, sample_rate);

                let (stream, handle) = OutputStream::try_from_device(&target_device).map_err(
                    |e| {
                        format!(
                            "Failed to open device '{}': {}",
                            device_name, e
                        )
                    },
                )?;

                (stream, handle, device_rate)
            } else {
                // Use default device — let rodio/cpal pick the format
                let (stream, handle) = OutputStream::try_default()
                    .map_err(|e| format!("Failed to create default output stream: {}", e))?;

                (stream, handle, sample_rate)
            };

        let processing_chain = Arc::new(Mutex::new(ProcessingChain::new(actual_sample_rate)));

        let sink =
            Sink::try_new(&stream_handle).map_err(|e| format!("Failed to create sink: {}", e))?;

        // Create noise source
        let noise_source = NoiseSource::new(Arc::clone(&processing_chain), actual_sample_rate);

        // Set source to loop indefinitely
        sink.append(noise_source.repeat_infinite());
        sink.set_volume(1.0);

        // Pause audio immediately to prevent startup noise with default settings
        sink.pause();

        Ok(Self {
            _stream: stream,
            _stream_handle: stream_handle,
            sink,
            processing_chain,
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
        self.sink.play();
    }

    pub fn pause(&self) {
        #[cfg(feature = "debug_audio")]
        println!("AudioEngine::pause() called");
        self.sink.pause();
    }

    pub fn is_playing(&self) -> bool {
        !self.sink.is_paused()
    }

    pub fn toggle(&self) {
        let was_paused = self.sink.is_paused();
        if was_paused {
            self.sink.play();
        } else {
            self.sink.pause();
        }
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

// Cached config to avoid locking every sample
#[derive(Clone, Copy)]
struct CachedConfig {
    smoothing_enabled: bool,
    overlay_enabled: bool,
    overlay_amount: f32,
}

impl CachedConfig {
    fn new() -> Self {
        Self {
            smoothing_enabled: false,
            overlay_enabled: false,
            overlay_amount: 0.3,
        }
    }
}

// Custom source that generates noise from the processing chain
struct NoiseSource {
    processing_chain: Arc<Mutex<ProcessingChain>>,
    sample_rate: u32,
    left_sample: Option<f32>,
    cached_config: CachedConfig,
    samples_since_last_update: u32,
    config_update_interval: u32,
}

impl NoiseSource {
    fn new(processing_chain: Arc<Mutex<ProcessingChain>>, sample_rate: u32) -> Self {
        Self {
            processing_chain,
            sample_rate,
            left_sample: None,
            cached_config: CachedConfig::new(),
            samples_since_last_update: 0,
            config_update_interval: 256, // Update config every 256 samples (~2.7ms at 48kHz)
        }
    }

    fn update_cached_config(&mut self) {
        if let Ok(chain) = self.processing_chain.lock() {
            let (smoothing, overlay, _, overlay_amount) = chain.get_smoothing_config();
            self.cached_config = CachedConfig {
                smoothing_enabled: smoothing,
                overlay_enabled: overlay,
                overlay_amount,
            };
        }
    }
}

impl Iterator for NoiseSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // If we have a left sample cached, return it and cache the right
        if let Some(cached) = self.left_sample.take() {
            return Some(cached);
        }

        // Update cached config periodically to reduce lock frequency
        self.samples_since_last_update += 1;
        if self.samples_since_last_update >= self.config_update_interval {
            self.update_cached_config();
            self.samples_since_last_update = 0;
        }

        // Generate a new stereo pair
        // Lock only for processing, not for config reading
        let (left, right) = if let Ok(mut chain) = self.processing_chain.lock() {
            chain.process_stereo(
                self.cached_config.smoothing_enabled,
                self.cached_config.overlay_enabled,
                self.cached_config.overlay_amount,
            )
        } else {
            // If lock fails, return silence
            (0.0, 0.0)
        };

        // Cache the right sample and return the left
        self.left_sample = Some(right);

        Some(left)
    }
}

impl Source for NoiseSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        2 // Stereo
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
