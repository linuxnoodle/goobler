use crate::audio::filters::{BassBoost, Filter};
use crate::audio::noise::NoiseGenerator;
use crate::config::NoiseType;

// Simple low-pass filter for smoothing
struct SmoothingFilter {
    prev_sample: f32,
    alpha: f32,
}

impl SmoothingFilter {
    fn new(alpha: f32) -> Self {
        Self {
            prev_sample: 0.0,
            alpha: alpha.clamp(0.0, 1.0),
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.prev_sample + self.alpha * (input - self.prev_sample);
        self.prev_sample = output;
        output
    }

    fn set_alpha(&mut self, alpha: f32) {
        // IMPORTANT: alpha must never be 0 or filter will freeze!
        self.alpha = alpha.clamp(0.001, 1.0);
    }
}

pub struct AudioProcessor {
    noise_generator: NoiseGenerator,
    overlay_generator: NoiseGenerator,
    bass_boost: BassBoost,
    low_pass: Filter,
    high_pass: Filter,
    smoothing_filter: SmoothingFilter,
    volume: f32,
    sample_rate: u32,
    smoothing_amount: f32,
    overlay_enabled: bool,
    overlay_amount: f32,
}

impl AudioProcessor {
    pub fn new(sample_rate: u32) -> Self {
        let noise_generator = NoiseGenerator::new(NoiseType::Pink);
        let overlay_generator = NoiseGenerator::new(NoiseType::White);
        let bass_boost = BassBoost::new(3.0, 200.0, sample_rate);
        let low_pass = Filter::new(
            crate::audio::filters::FilterType::LowPass,
            2000.0,
            sample_rate,
        );
        let high_pass = Filter::new(
            crate::audio::filters::FilterType::HighPass,
            100.0,
            sample_rate,
        );
        let smoothing_filter = SmoothingFilter::new(0.5);

        Self {
            noise_generator,
            overlay_generator,
            bass_boost,
            low_pass,
            high_pass,
            smoothing_filter,
            volume: 0.8,
            sample_rate,
            smoothing_amount: 0.5,
            overlay_enabled: false,
            overlay_amount: 0.3,
        }
    }

    pub fn set_noise_type(&mut self, noise_type: NoiseType) {
        self.noise_generator.set_noise_type(noise_type);
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_bass_boost_db(&mut self, gain_db: f32) {
        self.bass_boost.set_gain_db(gain_db);
    }

    pub fn set_bass_boost_enabled(&mut self, enabled: bool) {
        self.bass_boost.set_enabled(enabled);
    }

    pub fn set_low_pass_enabled(&mut self, enabled: bool) {
        self.low_pass.set_enabled(enabled);
    }

    pub fn set_low_pass_frequency(&mut self, frequency: f32) {
        self.low_pass.set_frequency(frequency);
    }

    pub fn set_high_pass_enabled(&mut self, enabled: bool) {
        self.high_pass.set_enabled(enabled);
    }

    pub fn set_high_pass_frequency(&mut self, frequency: f32) {
        self.high_pass.set_frequency(frequency);
    }

    pub fn set_smoothing_enabled(&mut self, enabled: bool) {
        // smoothing_amount: 0.0 = no smoothing, 1.0 = max smoothing
        // alpha: 1.0 = no smoothing (pass through), 0.0 = max smoothing (freeze)
        // IMPORTANT: alpha must never be 0 or filter will freeze!
        let alpha = if enabled {
            // Prevent alpha from reaching exactly 0 by clamping smoothing_amount
            (1.0 - self.smoothing_amount.clamp(0.0, 0.999))
        } else {
            1.0
        };
        self.smoothing_filter.set_alpha(alpha);
    }

    pub fn set_smoothing_amount(&mut self, amount: f32) {
        // smoothing_amount: 0.0 = no smoothing, 1.0 = max smoothing
        // alpha: 1.0 = no smoothing (pass through), 0.0 = max smoothing (freeze)
        // IMPORTANT: alpha must never be 0 or filter will freeze!
        self.smoothing_amount = amount.clamp(0.0, 1.0);
        let alpha = (1.0 - self.smoothing_amount.clamp(0.0, 0.999));
        self.smoothing_filter.set_alpha(alpha);
    }

    pub fn set_overlay_type(&mut self, noise_type: NoiseType) {
        self.overlay_generator.set_noise_type(noise_type);
    }

    pub fn set_overlay_enabled(&mut self, enabled: bool) {
        self.overlay_enabled = enabled;
    }

    pub fn set_overlay_amount(&mut self, amount: f32) {
        self.overlay_amount = amount.clamp(0.0, 1.0);
    }

    // Accessor methods for smoothing config
    pub fn is_smoothing_enabled(&self) -> bool {
        self.smoothing_filter.alpha < 1.0
    }

    pub fn smoothing_amount(&self) -> f32 {
        self.smoothing_amount
    }

    pub fn is_overlay_enabled(&self) -> bool {
        self.overlay_enabled
    }

    pub fn overlay_amount(&self) -> f32 {
        self.overlay_amount
    }

    pub fn process_sample(
        &mut self,
        smoothing_enabled: bool,
        overlay_enabled: bool,
        overlay_amount: f32,
    ) -> f32 {
        // Generate primary noise
        let mut sample = self.noise_generator.generate();

        // Apply smoothing if enabled
        if smoothing_enabled {
            sample = self.smoothing_filter.process(sample);
        }

        // Add overlay noise if enabled
        if overlay_enabled {
            let overlay = self.overlay_generator.generate();
            sample = sample * (1.0 - overlay_amount) + overlay * overlay_amount;
        }

        // Apply filters in series
        sample = self.high_pass.process(sample);
        sample = self.low_pass.process(sample);
        sample = self.bass_boost.process(sample);

        // Apply volume
        sample *= self.volume;

        // Clamp to prevent clipping
        sample.clamp(-1.0, 1.0)
    }

    pub fn reset_filters(&mut self) {
        self.bass_boost.reset();
        self.low_pass.reset();
        self.high_pass.reset();
    }
}

// Macro to generate stereo delegation methods
macro_rules! stereo_method {
    ($name:ident, $type:ty) => {
        pub fn $name(&mut self, value: $type) {
            self.left.$name(value);
            self.right.$name(value);
        }
    };
    ($name:ident, $arg1:ident : $type1:ty, $arg2:ident : $type2:ty) => {
        pub fn $name(&mut self, $arg1: $type1, $arg2: $type2) {
            self.left.$name($arg1, $arg2);
            self.right.$name($arg1, $arg2);
        }
    };
}

// Processing chain for stereo output
pub struct ProcessingChain {
    left: AudioProcessor,
    right: AudioProcessor,
}

impl ProcessingChain {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            left: AudioProcessor::new(sample_rate),
            right: AudioProcessor::new(sample_rate),
        }
    }

    // Generate stereo delegation methods using macro
    stereo_method!(set_noise_type, NoiseType);
    stereo_method!(set_volume, f32);
    stereo_method!(set_bass_boost_db, f32);
    stereo_method!(set_bass_boost_enabled, bool);
    stereo_method!(set_low_pass_enabled, bool);
    stereo_method!(set_low_pass_frequency, f32);
    stereo_method!(set_high_pass_enabled, bool);
    stereo_method!(set_high_pass_frequency, f32);
    stereo_method!(set_smoothing_enabled, bool);
    stereo_method!(set_smoothing_amount, f32);
    stereo_method!(set_overlay_type, NoiseType);
    stereo_method!(set_overlay_enabled, bool);
    stereo_method!(set_overlay_amount, f32);

    pub fn process_stereo(
        &mut self,
        smoothing_enabled: bool,
        overlay_enabled: bool,
        overlay_amount: f32,
    ) -> (f32, f32) {
        let left = self
            .left
            .process_sample(smoothing_enabled, overlay_enabled, overlay_amount);
        let right = self
            .right
            .process_sample(smoothing_enabled, overlay_enabled, overlay_amount);
        (left, right)
    }

    pub fn reset_filters(&mut self) {
        self.left.reset_filters();
        self.right.reset_filters();
    }

    pub fn get_smoothing_config(&self) -> (bool, bool, f32, f32) {
        let left = &self.left;
        (
            left.is_smoothing_enabled(),
            left.is_overlay_enabled(),
            left.smoothing_amount(),
            left.overlay_amount(),
        )
    }
}
