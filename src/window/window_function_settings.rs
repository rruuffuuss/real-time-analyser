use super::window_function::{Window, WindowFunction};

use num_traits::Float;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct WindowFunctionSettings {
    pub(crate) samples: u32,
    #[serde(default = "default_window_function")]
    pub(crate) function: WindowFunction,
}

impl WindowFunctionSettings {
    pub fn build_f32(self) -> Window<f32> {
        self.build_for::<f32>()
    }

    pub fn build_f64(self) -> Window<f64> {
        self.build_for::<f64>()
    }

    fn build_for<F>(self) -> Window<F>
    where
        F: Float,
    {
        self.function.build_window::<F>(self.samples)
    }
}

const fn default_window_function() -> WindowFunction {
    WindowFunction::Hann
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config, ConfigError, File, FileFormat};

    fn parse(yaml: &str) -> Result<WindowFunctionSettings, ConfigError> {
        Config::builder()
            .add_source(File::from_str(yaml, FileFormat::Yaml))
            .build()?
            .try_deserialize()
    }

    #[test]
    fn loads_test_settings() {
        let settings = parse("function: blackman\nsamples: 131\n").unwrap();

        assert!(matches!(
            settings,
            WindowFunctionSettings {
                samples: 131,
                function: WindowFunction::Blackman,
            }
        ));
    }
}
