use super::merger::{ExponentialMerger, LinearMerger, Merger};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum MergerSettings {
    Linear {},
    Exponential {
        #[serde(default = "default_tuning_frequency")]
        tuning_frequency: f64,
        #[serde(default = "default_bars_per_octave")]
        bars_per_octave: usize,
        #[serde(default = "default_starting_note_offset")]
        starting_note_offset: isize,
    },
}

impl MergerSettings {
    pub fn build(
        &self,
        input_bins: usize,
        output_bars: usize,
        sample_rate: u32,
    ) -> Box<dyn Merger> {
        match self {
            Self::Linear {} => Box::new(LinearMerger::new(input_bins, output_bars)),
            Self::Exponential {
                tuning_frequency,
                bars_per_octave,
                starting_note_offset,
            } => Box::new(ExponentialMerger::new_custom_function(
                input_bins,
                output_bars,
                sample_rate,
                *tuning_frequency,
                *bars_per_octave,
                *starting_note_offset,
            )),
        }
    }
}

impl Default for MergerSettings {
    fn default() -> Self {
        Self::Linear {}
    }
}

const fn default_bars_per_octave() -> usize {
    12
}

const fn default_starting_note_offset() -> isize {
    -48
}

const fn default_tuning_frequency() -> f64 {
    440.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::*;
    use config::{Config, ConfigError, File, FileFormat};
    use std::path::Path;

    fn parse(yaml: &str) -> Result<Settings, ConfigError> {
        Config::builder()
            .add_source(File::from_str(yaml, FileFormat::Yaml))
            .build()?
            .try_deserialize()
    }

    #[test]
    fn loads_linear_settings_and_defaults() {
        let settings = parse("merger:\n  type: linear\n").unwrap();

        assert_eq!(settings.sample_window, 4096);
        assert!(matches!(settings.merger, MergerSettings::Linear {}));
    }

    #[test]
    fn example_configuration_loads() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.yaml");

        Settings::load(&path).unwrap();
    }

    #[test]
    fn loads_exponential_settings() {
        let settings = parse(
            "merger:\n  type: exponential\n  tuning_frequency: 432.0\n  bars_per_octave: 24\n  starting_note_offset: -72\n",
        )
        .unwrap();

        assert!(matches!(
            settings.merger,
            MergerSettings::Exponential {
                tuning_frequency: 432.0,
                bars_per_octave: 24,
                starting_note_offset: -72,
            }
        ));
    }

    #[test]
    fn rejects_settings_for_the_wrong_merger() {
        let result = parse("merger:\n  type: linear\n  bars_per_octave: 12\n");

        assert!(result.is_err());
    }
}
