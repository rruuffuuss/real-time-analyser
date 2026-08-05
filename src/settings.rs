use std::path::Path;

use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;

use crate::control::control_settings::ControlSettings;
use crate::display::display_settings::DisplaySettings;
use crate::transform::merger_settings::MergerSettings;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub controller: ControlSettings,
    pub merger: MergerSettings,
    #[serde(default = "default_display")]
    pub display: DisplaySettings,
}

impl Settings {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::from(path).format(FileFormat::Yaml))
            .build()?
            .try_deserialize()
    }
}

fn default_display() -> DisplaySettings {
    DisplaySettings::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::core::core_settings::CoreSettings;

    fn parse(yaml: &str) -> Result<Settings, ConfigError> {
        Config::builder()
            .add_source(File::from_str(yaml, FileFormat::Yaml))
            .build()?
            .try_deserialize()
    }

    #[test]
    fn loads_configured_settings() {
        let settings = parse(
            "controller:\n  type: monolithic\n  core:\n    sample_rate: 44100\n    transform_size: 2048\n    framerate: 60\nmerger:\n  merger_type:\n    type: linear\n  aggregator: sum\n  realisor: magnitude_squared\ndisplay:\n  display_width: 3\n  display_height: 2\n",
        )
        .unwrap();

        assert!(matches!(
            settings.controller,
            ControlSettings::Monolithic {
                core: CoreSettings {
                    sample_rate: 44100,
                    transform_size: 2048,
                    framerate: 60,
                }
            }
        ));
        let merger = settings.merger.build(4, 1, 44_100);
        let mut merged = [0.0];
        merger.merge_into_slice(
            &[
                rustfft::num_complex::Complex::new(1.0, 0.0),
                rustfft::num_complex::Complex::new(0.0, 2.0),
                rustfft::num_complex::Complex::new(10.0, 10.0),
                rustfft::num_complex::Complex::new(10.0, 10.0),
            ],
            &mut merged,
        );
        assert_eq!(merged, [5.0]);
        assert_eq!(settings.display.build().ideal_bar_count(), 3);
    }

    #[test]
    fn rejects_unknown_settings() {
        let result = parse(
            "controller:\n  type: monolithic\n  core: {}\nmerger:\n  merger_type:\n    type: linear\ndisplay:\n  display_width: 3\n  display_height: 2\ncolour: blue\n",
        );

        assert!(result.is_err());
    }
}
