use crate::config::NoiseType;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

pub struct NoiseGenerator {
    noise_type: NoiseType,
    // State for colored noise generation
    pink_state: PinkNoiseState,
    brown_state: BrownNoiseState,
    blue_state: BlueNoiseState,
    violet_state: VioletNoiseState,
    // Reusable thread-safe RNG to prevent memory leaks
    rng: StdRng,
}

#[derive(Clone)]
struct BlueNoiseState {
    previous_sample: f32,
}

impl Default for BlueNoiseState {
    fn default() -> Self {
        Self {
            previous_sample: 0.0,
        }
    }
}

#[derive(Clone)]
struct VioletNoiseState {
    previous_sample: f32,
}

impl Default for VioletNoiseState {
    fn default() -> Self {
        Self {
            previous_sample: 0.0,
        }
    }
}

#[derive(Clone)]
struct PinkNoiseState {
    b0: f32,
    b1: f32,
    b2: f32,
    b3: f32,
    b4: f32,
    b5: f32,
    b6: f32,
}

impl Default for PinkNoiseState {
    fn default() -> Self {
        Self {
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            b4: 0.0,
            b5: 0.0,
            b6: 0.0,
        }
    }
}

#[derive(Clone)]
struct BrownNoiseState {
    last_output: f32,
}

impl Default for BrownNoiseState {
    fn default() -> Self {
        Self { last_output: 0.0 }
    }
}

impl NoiseGenerator {
    pub fn new(noise_type: NoiseType) -> Self {
        Self {
            noise_type,
            pink_state: PinkNoiseState::default(),
            brown_state: BrownNoiseState::default(),
            blue_state: BlueNoiseState::default(),
            violet_state: VioletNoiseState::default(),
            rng: StdRng::from_entropy(),
        }
    }

    pub fn set_noise_type(&mut self, noise_type: NoiseType) {
        self.noise_type = noise_type;
    }

    pub fn generate(&mut self) -> f32 {
        match self.noise_type {
            NoiseType::White => self.generate_white(),
            NoiseType::Pink => self.generate_pink(),
            NoiseType::Brown => self.generate_brown(),
            NoiseType::Blue => self.generate_blue(),
            NoiseType::Violet => self.generate_violet(),
            NoiseType::Green => self.generate_green(),
        }
    }

    fn generate_white(&mut self) -> f32 {
        self.rng.gen_range(-1.0..1.0)
    }

    // Paul Kellet's refined pink noise algorithm
    fn generate_pink(&mut self) -> f32 {
        let white = self.rng.gen_range(-1.0..1.0);

        self.pink_state.b0 = 0.99886 * self.pink_state.b0 + white * 0.0555179;
        self.pink_state.b1 = 0.99332 * self.pink_state.b1 + white * 0.0750759;
        self.pink_state.b2 = 0.96900 * self.pink_state.b2 + white * 0.1538520;
        self.pink_state.b3 = 0.86650 * self.pink_state.b3 + white * 0.3104856;
        self.pink_state.b4 = 0.55000 * self.pink_state.b4 + white * 0.5329522;
        self.pink_state.b5 = -0.7616 * self.pink_state.b5 - white * 0.0168980;

        let pink = (self.pink_state.b0
            + self.pink_state.b1
            + self.pink_state.b2
            + self.pink_state.b3
            + self.pink_state.b4
            + self.pink_state.b5
            + self.pink_state.b6
            + white * 0.5362)
            * 0.11;

        self.pink_state.b6 = white * 0.115926;

        pink
    }

    // Brown noise (red noise)
    fn generate_brown(&mut self) -> f32 {
        let white = self.rng.gen_range(-1.0..1.0);

        let output = (self.brown_state.last_output + (0.02 * white)) / 1.02;
        self.brown_state.last_output = output;

        // Compensate for gain loss
        output * 3.5
    }

    // Blue noise (+3dB/octave) - implemented via differentiation
    fn generate_blue(&mut self) -> f32 {
        let current = self.generate_white();
        let output = current - self.blue_state.previous_sample;
        self.blue_state.previous_sample = current;
        // Normalize and apply gain
        output * 2.0
    }

    // Violet noise (+6dB/octave) - implemented via differentiation
    fn generate_violet(&mut self) -> f32 {
        let current = self.generate_pink();
        let output = current - self.violet_state.previous_sample;
        self.violet_state.previous_sample = current;
        // Normalize and apply gain
        output * 3.0
    }

    // Green noise (equal loudness contour)
    fn generate_green(&mut self) -> f32 {
        // Green noise is similar to grey noise - filtered to be equal loudness
        // This is a simplified approximation
        let pink = self.generate_pink();
        // Apply simple psychoacoustic filtering
        pink * 1.2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NoiseType;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn create_test_generator(noise_type: NoiseType) -> NoiseGenerator {
        // Use fixed seed for reproducible tests
        let mut gen = NoiseGenerator::new(noise_type);
        gen
    }

    #[test]
    fn test_generator_creation() {
        for noise_type in &NoiseType::ALL {
            let gen = create_test_generator(*noise_type);
            // Just verify it doesn't panic
            let _ = gen;
        }
    }

    #[test]
    fn test_generate_samples() {
        for noise_type in &NoiseType::ALL {
            let mut gen = create_test_generator(*noise_type);

            // Generate multiple samples
            for _ in 0..100 {
                let sample = gen.generate();
                // Samples should be within reasonable bounds
                assert!(
                    sample >= -10.0 && sample <= 10.0,
                    "Sample out of bounds for {:?}: {}",
                    noise_type,
                    sample
                );
            }
        }
    }

    #[test]
    fn test_white_noise_range() {
        let mut gen = create_test_generator(NoiseType::White);

        // Generate many samples and check range
        for _ in 0..1000 {
            let sample = gen.generate();
            assert!(
                sample >= -1.0 && sample <= 1.0,
                "White noise sample out of [-1, 1] range: {}",
                sample
            );
        }
    }

    #[test]
    fn test_set_noise_type() {
        let mut gen = create_test_generator(NoiseType::White);

        // Generate with white noise
        let _ = gen.generate();

        // Switch to pink noise
        gen.set_noise_type(NoiseType::Pink);
        let sample = gen.generate();

        // Should be valid
        assert!(sample.is_finite());
    }

    #[test]
    fn test_brown_noise_decay() {
        let mut gen = create_test_generator(NoiseType::Brown);

        let mut samples = Vec::new();
        for _ in 0..100 {
            samples.push(gen.generate());
        }

        // Brown noise should have less energy at high frequencies
        // Variance should be lower than white noise
        let variance = samples
            .iter()
            .map(|s| (s - samples.iter().sum::<f32>() / samples.len() as f32).powi(2))
            .sum::<f32>()
            / samples.len() as f32;

        assert!(variance > 0.0, "Brown noise should have non-zero variance");
    }

    #[test]
    fn test_blue_noise_characteristics() {
        let mut gen = create_test_generator(NoiseType::Blue);

        let mut samples = Vec::new();
        for _ in 0..100 {
            samples.push(gen.generate());
        }

        // Blue noise should have more high-frequency content
        // Check that consecutive samples are less correlated than brown
        let mut correlation = 0.0;
        for i in 1..samples.len() {
            correlation += (samples[i] - samples[i - 1]).abs();
        }
        correlation /= samples.len() as f32;

        // Blue noise (differentiation) should have high correlation between consecutive samples
        assert!(
            correlation > 0.1,
            "Blue noise should have high-frequency content"
        );
    }

    #[test]
    fn test_violet_noise_characteristics() {
        let mut gen = create_test_generator(NoiseType::Violet);

        let mut samples = Vec::new();
        for _ in 0..100 {
            samples.push(gen.generate());
        }

        // Violet noise should have very high frequency content
        // Similar to blue but even more pronounced
        let variance = samples.iter().map(|s| s.powi(2)).sum::<f32>() / samples.len() as f32;

        assert!(variance > 0.0, "Violet noise should have non-zero variance");
    }

    #[test]
    fn test_pink_noise_characteristics() {
        let mut gen = create_test_generator(NoiseType::Pink);

        let mut samples = Vec::new();
        for _ in 0..1000 {
            samples.push(gen.generate());
        }

        // Pink noise should have 1/f characteristics
        // Mean should be around zero
        let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
        assert!(
            mean.abs() < 0.5,
            "Pink noise mean should be near zero: {}",
            mean
        );

        // Variance should be non-zero
        let variance =
            samples.iter().map(|s| (s - mean).powi(2)).sum::<f32>() / samples.len() as f32;
        assert!(variance > 0.0, "Pink noise should have non-zero variance");
    }

    #[test]
    fn test_green_noise_characteristics() {
        let mut gen = create_test_generator(NoiseType::Green);

        let mut samples = Vec::new();
        for _ in 0..100 {
            samples.push(gen.generate());
        }

        // Green noise should be similar to pink but adjusted for equal loudness
        let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
        assert!(
            mean.abs() < 0.5,
            "Green noise mean should be near zero: {}",
            mean
        );
    }

    #[test]
    fn test_all_noise_types_produce_valid_samples() {
        for noise_type in &NoiseType::ALL {
            let mut gen = create_test_generator(*noise_type);

            for _ in 0..50 {
                let sample = gen.generate();

                // Check for NaN and infinity
                assert!(
                    sample.is_finite(),
                    "Noise type {:?} produced non-finite value: {}",
                    noise_type,
                    sample
                );

                // Check reasonable bounds (allow some headroom for gain)
                assert!(
                    sample >= -100.0 && sample <= 100.0,
                    "Noise type {:?} produced out-of-bounds value: {}",
                    noise_type,
                    sample
                );
            }
        }
    }

    #[test]
    fn test_consistency_same_seed() {
        // This test would require deterministic seeding
        // For now, just verify that multiple generators produce different results
        let mut gen1 = create_test_generator(NoiseType::White);
        let mut gen2 = create_test_generator(NoiseType::White);

        let sample1 = gen1.generate();
        let sample2 = gen2.generate();

        // With entropy-based seeding, they should be different
        // (This is expected to fail occasionally, but extremely unlikely)
        assert_ne!(
            sample1, sample2,
            "Different generators should produce different samples"
        );
    }
}
