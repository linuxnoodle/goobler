use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub enum NoiseType {
    White,
    Pink,
    Brown,
    Green,
    Blue,
    Violet,
}

impl NoiseType {
    pub const ALL: [NoiseType; 6] = [
        NoiseType::White,
        NoiseType::Pink,
        NoiseType::Brown,
        NoiseType::Green,
        NoiseType::Blue,
        NoiseType::Violet,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            NoiseType::White => "White",
            NoiseType::Pink => "Pink",
            NoiseType::Brown => "Brown",
            NoiseType::Green => "Green",
            NoiseType::Blue => "Blue",
            NoiseType::Violet => "Violet",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioConfig {
    pub noise_type: NoiseType,
    pub volume: f32,
    pub bass_boost_db: f32,
    pub low_pass_enabled: bool,
    pub low_pass_freq: f32,
    pub high_pass_enabled: bool,
    pub high_pass_freq: f32,
    pub smoothing_enabled: bool,
    pub smoothing_amount: f32,
    pub overlay_enabled: bool,
    pub overlay_type: NoiseType,
    pub overlay_amount: f32,
    pub sample_rate: u32,
    pub output_device: Option<String>,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            noise_type: NoiseType::Pink,
            volume: 0.8,
            bass_boost_db: 0.0,
            low_pass_enabled: false,
            low_pass_freq: 2000.0,
            high_pass_enabled: false,
            high_pass_freq: 100.0,
            smoothing_enabled: false,
            smoothing_amount: 0.5,
            overlay_enabled: false,
            overlay_type: NoiseType::White,
            overlay_amount: 0.3,
            sample_rate: 44100,  // Standard CD quality
            output_device: None, // Use system default
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub audio: AudioConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            audio: AudioConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_type_display_names() {
        assert_eq!(NoiseType::White.display_name(), "White");
        assert_eq!(NoiseType::Pink.display_name(), "Pink");
        assert_eq!(NoiseType::Brown.display_name(), "Brown");
        assert_eq!(NoiseType::Green.display_name(), "Green");
        assert_eq!(NoiseType::Blue.display_name(), "Blue");
        assert_eq!(NoiseType::Violet.display_name(), "Violet");
    }

    #[test]
    fn test_noise_type_all_variants() {
        assert_eq!(NoiseType::ALL.len(), 6);
        assert!(NoiseType::ALL.contains(&NoiseType::White));
        assert!(NoiseType::ALL.contains(&NoiseType::Pink));
        assert!(NoiseType::ALL.contains(&NoiseType::Brown));
        assert!(NoiseType::ALL.contains(&NoiseType::Green));
        assert!(NoiseType::ALL.contains(&NoiseType::Blue));
        assert!(NoiseType::ALL.contains(&NoiseType::Violet));
    }

    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();

        assert_eq!(config.noise_type, NoiseType::Pink);
        assert_eq!(config.volume, 0.8);
        assert_eq!(config.bass_boost_db, 0.0);
        assert_eq!(config.low_pass_enabled, false);
        assert_eq!(config.low_pass_freq, 2000.0);
        assert_eq!(config.high_pass_enabled, false);
        assert_eq!(config.high_pass_freq, 100.0);
        assert_eq!(config.smoothing_enabled, false);
        assert_eq!(config.smoothing_amount, 0.5);
        assert_eq!(config.overlay_enabled, false);
        assert_eq!(config.overlay_type, NoiseType::White);
        assert_eq!(config.overlay_amount, 0.3);
        assert_eq!(config.sample_rate, 44100);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();

        assert_eq!(config.audio.noise_type, NoiseType::Pink);
        assert_eq!(config.audio.volume, 0.8);
    }

    #[test]
    fn test_audio_config_serialization() {
        let config = AudioConfig {
            noise_type: NoiseType::White,
            volume: 0.5,
            bass_boost_db: 6.0,
            low_pass_enabled: true,
            low_pass_freq: 1000.0,
            high_pass_enabled: true,
            high_pass_freq: 200.0,
            smoothing_enabled: true,
            smoothing_amount: 0.7,
            overlay_enabled: true,
            overlay_type: NoiseType::Pink,
            overlay_amount: 0.2,
            sample_rate: 48000,
            output_device: Some("Test Device".to_string()),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&config).expect("Failed to serialize");

        // Deserialize back
        let deserialized: AudioConfig = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized, config);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            audio: AudioConfig::default(),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&config).expect("Failed to serialize");

        // Deserialize back
        let deserialized: Config = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.audio.noise_type, config.audio.noise_type);
        assert_eq!(deserialized.audio.volume, config.audio.volume);
    }

    #[test]
    fn test_audio_config_clone() {
        let config = AudioConfig::default();
        let cloned = config.clone();

        assert_eq!(config, cloned);
    }

    #[test]
    fn test_config_clone() {
        let config = Config::default();
        let cloned = config.clone();

        assert_eq!(config.audio.noise_type, cloned.audio.noise_type);
        assert_eq!(config.audio.volume, cloned.audio.volume);
    }

    #[test]
    fn test_noise_type_equality() {
        assert_eq!(NoiseType::White, NoiseType::White);
        assert_ne!(NoiseType::White, NoiseType::Pink);
    }

    #[test]
    fn test_audio_config_bounds() {
        let config = AudioConfig::default();

        // Volume should be in reasonable range
        assert!(config.volume >= 0.0 && config.volume <= 1.0);

        // Frequencies should be positive
        assert!(config.low_pass_freq > 0.0);
        assert!(config.high_pass_freq > 0.0);

        // Smoothing and overlay amounts should be in [0, 1]
        assert!(config.smoothing_amount >= 0.0 && config.smoothing_amount <= 1.0);
        assert!(config.overlay_amount >= 0.0 && config.overlay_amount <= 1.0);

        // Sample rate should be standard
        assert!(config.sample_rate == 44100 || config.sample_rate == 48000);
    }

    #[test]
    fn test_all_noise_types_serializable() {
        for noise_type in &NoiseType::ALL {
            let json = serde_json::to_string(noise_type).expect("Failed to serialize");
            let deserialized: NoiseType =
                serde_json::from_str(&json).expect("Failed to deserialize");

            assert_eq!(deserialized, *noise_type);
        }
    }
}
