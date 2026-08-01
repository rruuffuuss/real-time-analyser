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
    pub fn build(self, input_bins: usize, output_bars: usize, sample_rate: u32) -> Box<dyn Merger> {
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
                tuning_frequency,
                bars_per_octave,
                starting_note_offset,
            )),
        }
    }

    //called for decimating controller construction where each octave must be merged seperately & uniformly
    pub fn build_single_octave(
        mut self,
        input_bins: usize,
        new_bars_per_octave: isize,
        sample_rate: u32,
    ) -> Box<dyn Merger> {
        if let MergerSettings::Exponential {
            tuning_frequency,
            bars_per_octave,
            starting_note_offset,
        } = &mut self
        {
            {
                *bars_per_octave = new_bars_per_octave as usize;
                *starting_note_offset = -1 * new_bars_per_octave;
            }
        }

        self.build(input_bins, new_bars_per_octave as usize, sample_rate)
    }

    pub fn set_octave_uniform(&mut self, new_bars_per_octave: isize) {
        if let MergerSettings::Exponential {
            tuning_frequency,
            bars_per_octave,
            starting_note_offset,
        } = self
        {
            {
                *bars_per_octave = new_bars_per_octave as usize;
                *starting_note_offset = -1 * new_bars_per_octave;
            }
        }
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
    use config::{Config, ConfigError, File, FileFormat};

    fn parse(yaml: &str) -> Result<MergerSettings, ConfigError> {
        Config::builder()
            .add_source(File::from_str(yaml, FileFormat::Yaml))
            .build()?
            .try_deserialize()
    }

    #[test]
    fn loads_linear_settings_and_defaults() {
        let settings = parse("type: linear\n").unwrap();

        //assert_eq!(settings.sample_window, 4096);
        assert!(matches!(settings, MergerSettings::Linear {}));
    }

    #[test]
    fn loads_exponential_settings() {
        let settings = parse(
            "type: exponential\ntuning_frequency: 432.0\nbars_per_octave: 24\nstarting_note_offset: -72\n",
        )
        .unwrap();

        assert!(matches!(
            settings,
            MergerSettings::Exponential {
                tuning_frequency: 432.0,
                bars_per_octave: 24,
                starting_note_offset: -72,
            }
        ));
    }

    #[test]
    fn rejects_settings_for_the_wrong_merger() {
        let result = parse("type: linear\nbars_per_octave: 12\n");

        assert!(result.is_err());
    }
}
