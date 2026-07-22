use std::path::Path;

use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;

use crate::display::display_settings::DisplaySettings;
use crate::transform::merger_settings::MergerSettings;

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub sample_window: usize,
    pub sample_rate: u32,
    pub min_frequency: usize,
    pub max_frequency: usize,
    pub framerate: u16,
    pub display: DisplaySettings,
    pub merger: MergerSettings,
}

impl Settings {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::from(path).format(FileFormat::Yaml))
            .build()?
            .try_deserialize()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sample_window: 4096,
            sample_rate: 48_000,
            min_frequency: 1,
            max_frequency: 4096,
            framerate: 30,
            display: DisplaySettings::default(),
            merger: MergerSettings::default(),
        }
    }
}
