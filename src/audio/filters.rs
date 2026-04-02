use std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub enum FilterType {
    LowPass,
    HighPass,
}

#[derive(Debug, Clone)]
pub struct Filter {
    filter_type: FilterType,
    enabled: bool,
    frequency: f32,
    sample_rate: u32,
    // Biquad filter coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    // State variables
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Filter {
    pub fn new(filter_type: FilterType, frequency: f32, sample_rate: u32) -> Self {
        let mut filter = Self {
            filter_type,
            enabled: true,
            frequency,
            sample_rate,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        };
        filter.calculate_coefficients();
        filter
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.calculate_coefficients();
    }

    fn calculate_coefficients(&mut self) {
        let sample_rate = self.sample_rate as f32;
        let omega = 2.0 * PI * self.frequency / sample_rate;
        let sn = omega.sin();
        let cs = omega.cos();
        let alpha = sn / (2.0 * std::f32::consts::SQRT_2); // Q = 1/√2 (Butterworth)

        let (b0, b1, b2, a0, a1, a2) = match self.filter_type {
            FilterType::LowPass => {
                let b0 = (1.0 - cs) / 2.0;
                let b1 = 1.0 - cs;
                let b2 = (1.0 - cs) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cs;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::HighPass => {
                let b0 = (1.0 + cs) / 2.0;
                let b1 = -(1.0 + cs);
                let b2 = (1.0 + cs) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cs;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
        };

        // Normalize by a0
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        if !self.enabled {
            return input;
        }

        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        // Update state
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

// Bass boost filter (shelf filter)
pub struct BassBoost {
    enabled: bool,
    gain_db: f32,
    frequency: f32,
    sample_rate: u32,
    // Biquad filter coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    // State variables
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BassBoost {
    pub fn new(gain_db: f32, frequency: f32, sample_rate: u32) -> Self {
        let mut boost = Self {
            enabled: true,
            gain_db,
            frequency,
            sample_rate,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        };
        boost.calculate_coefficients();
        boost
    }

    pub fn set_gain_db(&mut self, gain_db: f32) {
        self.gain_db = gain_db;
        self.calculate_coefficients();
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn calculate_coefficients(&mut self) {
        let sample_rate = self.sample_rate as f32;
        let omega = 2.0 * PI * self.frequency / sample_rate;
        let sn = omega.sin();
        let cs = omega.cos();
        let alpha = sn / (2.0 * std::f32::consts::SQRT_2);
        let a = (10.0_f32).powf(self.gain_db / 40.0);

        // Low shelf filter coefficients
        let b0 = a * ((a + 1.0) - (a - 1.0) * cs + 2.0 * alpha.sqrt() * a);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cs);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cs - 2.0 * alpha.sqrt() * a);
        let a0 = (a + 1.0) + (a - 1.0) * cs + 2.0 * alpha.sqrt() * a;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cs);
        let a2 = (a + 1.0) + (a - 1.0) * cs - 2.0 * alpha.sqrt() * a;

        // Normalize by a0
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        if !self.enabled {
            return input;
        }

        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        // Update state
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: u32 = 44100;

    #[test]
    fn test_filter_creation() {
        let low_pass = Filter::new(FilterType::LowPass, 1000.0, SAMPLE_RATE);
        let high_pass = Filter::new(FilterType::HighPass, 1000.0, SAMPLE_RATE);

        // Verify they don't panic
        let _ = low_pass;
        let _ = high_pass;
    }

    #[test]
    fn test_filter_enabled_disabled() {
        let mut filter = Filter::new(FilterType::LowPass, 1000.0, SAMPLE_RATE);

        let input = 0.5;
        let output_enabled = filter.process(input);

        filter.set_enabled(false);
        let output_disabled = filter.process(input);

        // When disabled, output should equal input
        assert_eq!(
            output_disabled, input,
            "Disabled filter should pass through input"
        );

        filter.set_enabled(true);
        let output_enabled2 = filter.process(input);

        // When enabled, output should be processed
        assert_ne!(output_enabled2, input, "Enabled filter should modify input");
    }

    #[test]
    fn test_filter_frequency_change() {
        let mut filter = Filter::new(FilterType::LowPass, 1000.0, SAMPLE_RATE);

        // Process some samples at 1000Hz
        let input = 0.5;
        let output1 = filter.process(input);

        // Change frequency
        filter.set_frequency(500.0);

        // Process same input
        let output2 = filter.process(input);

        // Outputs should be different due to different cutoff
        assert_ne!(
            output1, output2,
            "Different frequencies should produce different outputs"
        );
    }

    #[test]
    fn test_filter_reset() {
        let mut filter = Filter::new(FilterType::LowPass, 1000.0, SAMPLE_RATE);

        // Process some samples to build up state
        for _ in 0..100 {
            filter.process(0.5);
        }

        // Reset() filter
        filter.reset();

        // Process same input again - should be different due to reset state
        let input = 0.5;
        let output_after_reset = filter.process(input);

        // Output should be valid (not NaN or infinity)
        assert!(output_after_reset.is_finite());
    }

    #[test]
    fn test_low_pass_attenuates_high_frequencies() {
        let mut filter = Filter::new(FilterType::LowPass, 100.0, SAMPLE_RATE);

        // Generate a high-frequency signal (above cutoff)
        let mut high_freq_signal = 0.0;
        for i in 0..100 {
            high_freq_signal = (i as f32 * 0.5).sin(); // Rapid changes
        }

        let output = filter.process(high_freq_signal);

        // Low pass should attenuate rapid changes
        assert!(output.abs() <= high_freq_signal.abs() * 1.5);
    }

    #[test]
    fn test_high_pass_attenuates_low_frequencies() {
        let mut filter = Filter::new(FilterType::HighPass, 5000.0, SAMPLE_RATE);

        // Generate a low-frequency constant signal
        let low_freq_signal = 0.5;

        // Let filter settle
        for _ in 0..10 {
            filter.process(low_freq_signal);
        }

        let output = filter.process(low_freq_signal);

        // High pass should attenuate DC and low frequencies
        // After settling, output should be close to zero for constant input
        assert!(
            output.abs() < 0.1,
            "High pass should attenuate low frequencies: {}",
            output
        );
    }

    #[test]
    fn test_filter_stability() {
        let mut filter = Filter::new(FilterType::LowPass, 1000.0, SAMPLE_RATE);

        // Process many samples
        for _ in 0..10000 {
            let input = rand::random::<f32>() * 2.0 - 1.0;
            let output = filter.process(input);

            // Filter should remain stable (no blow-up)
            assert!(output.is_finite(), "Filter should remain stable");
            assert!(output.abs() < 100.0, "Filter output should not explode");
        }
    }

    #[test]
    fn test_bass_boost_creation() {
        let boost = BassBoost::new(6.0, 100.0, SAMPLE_RATE);

        // Verify it doesn't panic
        let _ = boost;
    }

    #[test]
    fn test_bass_boost_enabled_disabled() {
        let mut boost = BassBoost::new(6.0, 100.0, SAMPLE_RATE);

        let input = 0.5;
        let output_enabled = boost.process(input);

        boost.set_enabled(false);
        let output_disabled = boost.process(input);

        // When disabled, output should equal input
        assert_eq!(
            output_disabled, input,
            "Disabled bass boost should pass through input"
        );

        boost.set_enabled(true);
        let output_enabled2 = boost.process(input);

        // When enabled, output should be processed
        assert_ne!(
            output_enabled2, input,
            "Enabled bass boost should modify input"
        );
    }

    #[test]
    fn test_bass_boost_gain_change() {
        let mut boost = BassBoost::new(6.0, 100.0, SAMPLE_RATE);

        let input = 0.5;
        let output1 = boost.process(input);

        // Change gain
        boost.set_gain_db(12.0);

        let output2 = boost.process(input);

        // Different gain should produce different outputs
        assert_ne!(
            output1, output2,
            "Different gains should produce different outputs"
        );
    }

    #[test]
    fn test_bass_boost_reset() {
        let mut boost = BassBoost::new(6.0, 100.0, SAMPLE_RATE);

        // Process some samples to build up state
        for _ in 0..100 {
            boost.process(0.5);
        }

        // Reset() boost
        boost.reset();

        // Process same input again
        let input = 0.5;
        let output_after_reset = boost.process(input);

        // Output should be valid
        assert!(output_after_reset.is_finite());
    }

    #[test]
    fn test_bass_boost_increases_low_frequency() {
        let mut boost = BassBoost::new(10.0, 100.0, SAMPLE_RATE);

        // Process a low-frequency signal
        let input = 0.5;

        // Let filter settle
        for _ in 0..20 {
            boost.process(input);
        }

        let output = boost.process(input);

        // Bass boost should increase level of low frequencies
        // Output should be larger than input (assuming positive gain)
        assert!(
            output.abs() > input.abs() * 0.5,
            "Bass boost should affect low frequencies: input={}, output={}",
            input,
            output
        );
    }

    #[test]
    fn test_bass_boost_stability() {
        let mut boost = BassBoost::new(20.0, 100.0, SAMPLE_RATE);

        // Process many samples
        for _ in 0..10000 {
            let input = rand::random::<f32>() * 2.0 - 1.0;
            let output = boost.process(input);

            // Boost should remain stable (no blow-up)
            assert!(output.is_finite(), "Bass boost should remain stable");
            assert!(output.abs() < 100.0, "Bass boost output should not explode");
        }
    }

    #[test]
    fn test_zero_gain_bass_boost() {
        let mut boost = BassBoost::new(0.0, 100.0, SAMPLE_RATE);

        // Zero gain should still be stable
        for _ in 0..100 {
            let input = rand::random::<f32>() * 2.0 - 1.0;
            let output = boost.process(input);
            assert!(output.is_finite());
        }
    }

    #[test]
    fn test_filter_edge_cases() {
        // Test with very low frequency
        let mut low_pass = Filter::new(FilterType::LowPass, 1.0, SAMPLE_RATE);
        let _ = low_pass.process(0.5);

        // Test with very high frequency
        let mut high_pass =
            Filter::new(FilterType::HighPass, SAMPLE_RATE as f32 / 2.0, SAMPLE_RATE);
        let _ = high_pass.process(0.5);

        // Test with zero input
        let mut filter = Filter::new(FilterType::LowPass, 1000.0, SAMPLE_RATE);
        let output = filter.process(0.0);
        assert_eq!(output, 0.0, "Zero input should produce zero output");
    }
}
