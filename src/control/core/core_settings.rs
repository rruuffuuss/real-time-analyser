use crate::{display::display::Display, transform::merger::Merger};

use super::core::ControlCore;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub struct CoreSettings {
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,
    #[serde(default = "default_transform_size")]
    pub transform_size: usize,
    #[serde(default = "default_framerate")]
    pub framerate: u16,
}

impl CoreSettings {
    pub fn build(self, display: Display, merger: Box<dyn Merger>) -> ControlCore {
        ControlCore::new(
            self.transform_size,
            self.sample_rate,
            self.framerate,
            display,
            merger,
        )
    }
}

const fn default_sample_rate() -> u32 {
    48000
}

const fn default_transform_size() -> usize {
    1024
}

const fn default_framerate() -> u16 {
    30
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config, ConfigError, File, FileFormat};

    fn parse(yaml: &str) -> Result<CoreSettings, ConfigError> {
        Config::builder()
            .add_source(File::from_str(yaml, FileFormat::Yaml))
            .build()?
            .try_deserialize()
    }

    #[test]
    fn loads_configured_settings() {
        let settings = parse("sample_rate: 44100\ntransform_size: 2048\nframerate: 60\n").unwrap();

        assert!(matches!(
            settings,
            CoreSettings {
                sample_rate: 44100,
                transform_size: 2048,
                framerate: 60,
            }
        ));
    }

    #[test]
    fn loads_default_settings() {
        let settings = parse("{}\n").unwrap();

        assert!(matches!(
            settings,
            CoreSettings {
                sample_rate: 48000,
                transform_size: 1024,
                framerate: 30,
            }
        ));
    }
}
