use super::fir_filter::{FirFilter, Window};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub struct FirFilterSettings {
    #[serde(default = "default_taps")]
    taps: u32,
    #[serde(default = "default_normalised_cutoff")]
    normalised_cutoff: f64,
    #[serde(default = "default_window_function")]
    window_function: Window,
}

impl FirFilterSettings {
    pub fn build(self) -> FirFilter {
        FirFilter::new(self.taps, self.normalised_cutoff, self.window_function)
    }
}

const fn default_taps() -> u32 {
    51
}

const fn default_normalised_cutoff() -> f64 {
    0.5
}

const fn default_window_function() -> Window {
    Window::Triangular
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
            parse("window_function: Blackman\ntaps: 131\nnormalised_cutoff: 0.5\n").unwrap();

        assert!(matches!(
            settings,
            FirFilterSettings {
                taps: 131,
                normalised_cutoff: 0.5,
                window_function: Window::Blackman,
            }
        ));
    }
}
