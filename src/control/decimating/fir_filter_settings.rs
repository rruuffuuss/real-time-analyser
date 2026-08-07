use crate::window::{
    window_function::WindowFunction,
    window_function_settings::{self, WindowFunctionSettings},
};

use super::fir_filter::FirFilter;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct FirFilterSettings {
    #[serde(default = "default_normalised_cutoff")]
    normalised_cutoff: f64,
    #[serde(default = "default_window_function")]
    window: WindowFunctionSettings,
}

impl FirFilterSettings {
    pub fn build(self) -> FirFilter {
        let taps = self.window.samples;
        let window = self.window.build_f64();

        FirFilter::new(taps, self.normalised_cutoff, window)
    }
}

const fn default_normalised_cutoff() -> f64 {
    0.5
}

const fn default_window_function() -> WindowFunctionSettings {
    WindowFunctionSettings {
        samples: 51,
        function: WindowFunction::Hann,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config, ConfigError, File, FileFormat};

    fn parse(yaml: &str) -> Result<FirFilterSettings, ConfigError> {
        Config::builder()
            .add_source(File::from_str(yaml, FileFormat::Yaml))
            .build()?
            .try_deserialize()
    }

    #[test]
    fn loads_test_settings() {
        let settings =
            parse("window:\n    function: blackman\n    samples: 131\nnormalised_cutoff: 0.5\n")
                .unwrap();

        assert!(matches!(
            settings,
            FirFilterSettings {
                normalised_cutoff: 0.5,
                window: WindowFunctionSettings {
                    samples: 131,
                    function: WindowFunction::Blackman
                },
            }
        ));
    }
}
