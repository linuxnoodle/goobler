mod engine;
mod filters;
mod noise;
mod processor;
mod traits;

pub use engine::{AudioEngine, AudioEngineHandle};
pub use traits::{
    AudioProcessor, Filter as FilterTrait, NoiseGenerator as NoiseGeneratorTrait, StereoProcessor,
};

use cpal::traits::{DeviceTrait, HostTrait};
use std::collections::HashMap;

/// Represents an audio output device
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputDevice {
    pub name: String,
    pub is_default: bool,
}

/// Get a list of available output devices
pub fn get_output_devices() -> Result<Vec<OutputDevice>, String> {
    // Temporarily suppress stderr to hide harmless ALSA warnings during device enumeration
    // This prevents messages like "unable to open slave" and "Cannot open device /dev/dsp"
    let _stderr_guard = StderrGuard::new();

    let host = cpal::default_host();

    let devices: Vec<_> = host
        .output_devices()
        .map_err(|e| format!("Failed to get output devices: {}", e))?
        .collect();

    let default_device = host.default_output_device().and_then(|d| d.name().ok());

    let mut device_list: Vec<OutputDevice> = Vec::new();
    let mut seen_names: HashMap<String, bool> = HashMap::new();

    for device in devices {
        if let Ok(name) = device.name() {
            // Skip duplicate device names (some systems report the same device multiple times)
            if seen_names.contains_key(&name) {
                continue;
            }
            seen_names.insert(name.clone(), true);

            let is_default = default_device.as_ref() == Some(&name);

            device_list.push(OutputDevice { name, is_default });
        }
    }

    // Sort devices: default first, then alphabetically
    device_list.sort_by(|a, b| {
        if a.is_default && !b.is_default {
            std::cmp::Ordering::Less
        } else if !a.is_default && b.is_default {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    Ok(device_list)
}

/// Helper to temporarily suppress stderr to hide ALSA warnings
pub(crate) struct StderrGuard {
    original_fd: i32,
}

impl StderrGuard {
    fn new() -> Self {
        use std::fs::File;
        use std::os::unix::io::AsRawFd;

        // Open /dev/null to discard stderr output
        let null = File::open("/dev/null").expect("Failed to open /dev/null");

        // Duplicate the original stderr file descriptor
        let original_stderr = unsafe { libc::dup(libc::STDERR_FILENO) };

        // Redirect stderr to /dev/null
        unsafe {
            libc::dup2(null.as_raw_fd(), libc::STDERR_FILENO);
        }

        Self {
            original_fd: original_stderr,
        }
    }
}

impl Drop for StderrGuard {
    fn drop(&mut self) {
        // Restore the original stderr
        unsafe {
            libc::dup2(self.original_fd, libc::STDERR_FILENO);
            libc::close(self.original_fd);
        }
    }
}
