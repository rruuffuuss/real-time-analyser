use crate::transform::merger::aggregator::{self, Aggregator};
use crate::transform::merger::realisor::{self, Realisor};
use crate::transform::merger_settings::RealisorSetting::Imaginary;

use super::exponential_merger::ExponentialMerger;
use super::linear_merger::LinearMerger;
use super::merger::Merger;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregatorSetting {
    Mean,
    Sum,
    Max,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealisorSetting {
    Magnitude,
    MagnitudeSquared,
    Real,
    Imaginary,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum MergerTypeSettings {
    Linear,
    Exponential {
        #[serde(default = "default_tuning_frequency")]
        tuning_frequency: f64,
        #[serde(default = "default_bars_per_octave")]
        bars_per_octave: usize,
        #[serde(default = "default_starting_note_offset")]
        starting_note_offset: isize,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct MergerSettings {
    #[serde(default = "default_merger_type")]
    merger_type: MergerTypeSettings,
    #[serde(default = "default_aggregator")]
    aggregator: AggregatorSetting,
    #[serde(default = "default_realisor")]
    realisor: RealisorSetting,
}

impl MergerSettings {
    pub fn build(self, input_bins: usize, output_bars: usize, sample_rate: u32) -> Box<dyn Merger> {
        match self.realisor {
            RealisorSetting::Magnitude => self.build_with_realisor::<realisor::Magnitude>(
                input_bins,
                output_bars,
                sample_rate,
            ),
            RealisorSetting::MagnitudeSquared => self
                .build_with_realisor::<realisor::MagnitudeSquared>(
                    input_bins,
                    output_bars,
                    sample_rate,
                ),
            RealisorSetting::Real => {
                self.build_with_realisor::<realisor::Real>(input_bins, output_bars, sample_rate)
            }
            RealisorSetting::Imaginary => self.build_with_realisor::<realisor::Imaginary>(
                input_bins,
                output_bars,
                sample_rate,
            ),
        }
    }

    pub fn build_with_realisor<Rl: Realisor + 'static>(
        self,
        input_bins: usize,
        output_bars: usize,
        sample_rate: u32,
    ) -> Box<dyn Merger> {
        match self.aggregator {
            AggregatorSetting::Mean => self.build_with_aggregator::<aggregator::Mean<Rl>>(
                input_bins,
                output_bars,
                sample_rate,
            ),
            AggregatorSetting::Sum => self.build_with_aggregator::<aggregator::Sum<Rl>>(
                input_bins,
                output_bars,
                sample_rate,
            ),
            AggregatorSetting::Max => self.build_with_aggregator::<aggregator::Max<Rl>>(
                input_bins,
                output_bars,
                sample_rate,
            ),
        }
    }

    pub fn build_with_aggregator<Ag: Aggregator + 'static>(
        self,
        input_bins: usize,
        output_bars: usize,
        sample_rate: u32,
    ) -> Box<dyn Merger> {
        match self.merger_type {
            MergerTypeSettings::Linear => {
                Box::new(LinearMerger::<Ag>::new(input_bins, output_bars))
            }
            MergerTypeSettings::Exponential {
                tuning_frequency,
                bars_per_octave,
                starting_note_offset,
            } => Box::new(ExponentialMerger::<Ag>::new_custom_function(
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
    //in future potentially update this to clone settings first so that original settings are not modified
    pub fn build_single_octave(
        mut self,
        input_bins: usize,
        new_bars_per_octave: isize,
        sample_rate: u32,
    ) -> Box<dyn Merger> {
        if let MergerTypeSettings::Exponential {
            tuning_frequency,
            bars_per_octave,
            starting_note_offset,
        } = &mut self.merger_type
        {
            {
                *bars_per_octave = new_bars_per_octave as usize;
                *starting_note_offset = -1 * new_bars_per_octave;
            }
        };

        self.build(input_bins, new_bars_per_octave as usize, sample_rate)
    }
}

const fn default_merger_type() -> MergerTypeSettings {
    MergerTypeSettings::Linear
}

const fn default_aggregator() -> AggregatorSetting {
    AggregatorSetting::Mean
}

const fn default_realisor() -> RealisorSetting {
    RealisorSetting::MagnitudeSquared
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
    use rustfft::num_complex::Complex;

    fn parse(yaml: &str) -> Result<MergerSettings, ConfigError> {
        Config::builder()
            .add_source(File::from_str(yaml, FileFormat::Yaml))
            .build()?
            .try_deserialize()
    }

    #[test]
    fn loads_linear_settings_and_defaults() {
        let settings = parse("{}\n").unwrap();

        assert!(matches!(
            settings,
            MergerSettings {
                merger_type: MergerTypeSettings::Linear,
                aggregator: AggregatorSetting::Mean,
                realisor: RealisorSetting::MagnitudeSquared,
            }
        ));
    }

    #[test]
    fn loads_exponential_settings() {
        let settings = parse(
            "merger_type:\n  type: exponential\n  tuning_frequency: 432.0\n  bars_per_octave: 24\n  starting_note_offset: -72\naggregator: max\nrealisor: magnitude\n",
        )
        .unwrap();

        assert!(matches!(
            settings,
            MergerSettings {
                merger_type: MergerTypeSettings::Exponential {
                    tuning_frequency: 432.0,
                    bars_per_octave: 24,
                    starting_note_offset: -72,
                },
                aggregator: AggregatorSetting::Max,
                realisor: RealisorSetting::Magnitude,
            }
        ));
    }

    #[test]
    fn rejects_misplaced_merger_type_settings() {
        let result = parse("merger_type:\n  type: linear\nbars_per_octave: 12\n");

        assert!(result.is_err());
    }

    #[test]
    fn builds_linear_merger_for_every_aggregation_and_realisation_mode() {
        let cases = [
            ("mean", "magnitude", 7.5),
            ("mean", "magnitude_squared", 62.5),
            ("mean", "real", -2.5),
            ("mean", "imaginary", 5.0),
            ("sum", "magnitude", 15.0),
            ("sum", "magnitude_squared", 125.0),
            ("sum", "real", -5.0),
            ("sum", "imaginary", 10.0),
            ("max", "magnitude", 10.0),
            ("max", "magnitude_squared", 100.0),
            ("max", "real", 3.0),
            ("max", "imaginary", 6.0),
        ];
        let frequency_bins = [
            Complex::new(3.0, 4.0),
            Complex::new(-8.0, 6.0),
            Complex::new(100.0, 100.0),
            Complex::new(100.0, 100.0),
        ];

        for (aggregator, realisor, expected) in cases {
            let yaml = format!(
                "merger_type:\n  type: linear\naggregator: {aggregator}\nrealisor: {realisor}\n"
            );
            let merger = parse(&yaml).unwrap().build(4, 1, 48_000);
            let mut output = [0.0];

            merger.merge_into_slice(&frequency_bins, &mut output);

            assert_eq!(output[0], expected, "{aggregator} + {realisor}");
        }
    }

    #[test]
    fn builds_exponential_merger_with_selected_modes() {
        let settings = parse(
            "merger_type:\n  type: exponential\n  tuning_frequency: 2.0\n  bars_per_octave: 1\n  starting_note_offset: 0\naggregator: sum\nrealisor: real\n",
        )
        .unwrap();
        let merger = settings.build(64, 4, 64);
        let mut frequency_bins = vec![Complex::new(0.0, 0.0); 64];
        frequency_bins[0] = Complex::new(1.0, 10.0);
        frequency_bins[1] = Complex::new(2.0, 10.0);
        frequency_bins[2..5].fill(Complex::new(3.0, 10.0));
        frequency_bins[5..11].fill(Complex::new(4.0, 10.0));
        let mut output = [0.0; 4];

        merger.merge_into_slice(&frequency_bins, &mut output);

        assert_eq!(output, [1.0, 2.0, 9.0, 24.0]);
    }
}
