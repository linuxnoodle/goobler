//! Trait abstractions for audio processing components

use crate::config::{AudioConfig, NoiseType};

/// Trait for noise generation algorithms
pub trait NoiseGenerator: Send + Sync {
    /// Generate a single noise sample
    fn generate(&mut self) -> f32;

    /// Set the noise type (if supported)
    fn set_noise_type(&mut self, noise_type: NoiseType);

    /// Reset the generator state
    fn reset(&mut self);
}

/// Trait for audio filters
pub trait Filter: Send + Sync {
    /// Process a single sample through the filter
    fn process(&mut self, input: f32) -> f32;

    /// Enable or disable the filter
    fn set_enabled(&mut self, enabled: bool);

    /// Check if filter is enabled
    fn is_enabled(&self) -> bool;

    /// Reset the filter state (clear history)
    fn reset(&mut self);
}

/// Trait for audio processors (combines noise generation and filtering)
pub trait AudioProcessor: Send + Sync {
    /// Process a single audio sample
    fn process_sample(&mut self) -> f32;

    /// Set the master volume (0.0 to 1.0)
    fn set_volume(&mut self, volume: f32);

    /// Get the current master volume
    fn volume(&self) -> f32;

    /// Reset all processor state
    fn reset(&mut self);
}

/// Trait for stereo audio processing chains
pub trait StereoProcessor: Send + Sync {
    /// Process a stereo sample pair
    fn process_stereo(&mut self) -> (f32, f32);

    /// Apply configuration to both channels
    fn apply_config(&mut self, config: &AudioConfig);

    /// Reset both channels
    fn reset(&mut self);
}
