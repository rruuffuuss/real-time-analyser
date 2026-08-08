use super::core::core_settings::CoreSettings;
use super::{decimating, monolithic};
use crate::control::Controller;
use crate::control::decimating::fir_filter_settings::FirFilterSettings;
use crate::display::display_settings::DisplaySettings;
use crate::normalise::normaliser_settings::NormaliserSettings;
use crate::transform::merger_settings::MergerSettings;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ControlSettings {
    Monolithic {
        core: CoreSettings,
    },
    Decimating {
        core: CoreSettings,
        filter: FirFilterSettings,
        #[serde(default = "default_decimations")]
        decimations: usize,
        #[serde(default = "default_displayed_decimations")]
        displayed_decimations: usize,
    },
}

impl ControlSettings {
    pub fn build(
        self,
        display_settings: DisplaySettings,
        merger_settings: MergerSettings,
        normaliser_settings: NormaliserSettings,
    ) -> Box<dyn Controller> {
        let normaliser = normaliser_settings.build();

        let display = display_settings.build();

        match self {
            Self::Monolithic { core } => {
                let merger = merger_settings.build(
                    core.transform_size,
                    display.ideal_bar_count(),
                    core.sample_rate,
                );

                Box::new(monolithic::controller::MonolithicController::new(
                    core.build(display, merger, normaliser),
                ))
            }
            Self::Decimating {
                core,
                filter,
                decimations,
                displayed_decimations,
            } => {
                let merger = merger_settings.build_single_octave(
                    core.transform_size,
                    (display.ideal_bar_count() / (displayed_decimations)) as isize,
                    core.sample_rate,
                );

                Box::new(decimating::controller::DecimatingController::new(
                    core.build(display, merger, normaliser),
                    filter.build(),
                    decimations.clone(),
                    displayed_decimations.clone(),
                ))
            }
        }
    }
}

const fn default_decimations() -> usize {
    9
}

const fn default_displayed_decimations() -> usize {
    9
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window::{
        window_function::WindowFunction, window_function_settings::WindowFunctionSettings,
    };
    use config::{Config, ConfigError, File, FileFormat};

    fn parse(yaml: &str) -> Result<ControlSettings, ConfigError> {
        Config::builder()
            .add_source(File::from_str(yaml, FileFormat::Yaml))
            .build()?
            .try_deserialize()
    }

    #[test]
    fn loads_monolithic_settings() {
        let settings = parse(
            "type: monolithic\ncore:\n  sample_rate: 44100\n  transform_size: 2048\n  framerate: 60\n",
        )
        .unwrap();

        assert!(matches!(
            settings,
            ControlSettings::Monolithic {
                core: CoreSettings {
                    sample_rate: 44100,
                    transform_size: 2048,
                    framerate: 60,
                    window: WindowFunctionSettings {
                        samples: None,
                        function: WindowFunction::Hann
                    }
                }
            }
        ));
    }

    #[test]
    fn loads_decimating_settings_and_defaults() {
        let settings = parse("type: decimating\ncore: {}\nfilter: {}\n").unwrap();

        assert!(matches!(
            settings,
            ControlSettings::Decimating {
                core: CoreSettings {
                    sample_rate: 48000,
                    transform_size: 1024,
                    framerate: 30,
                    window: WindowFunctionSettings {
                        samples: None,
                        function: WindowFunction::Hann
                    }
                },
                filter: _,
                decimations: 9,
                displayed_decimations: 9,
            }
        ));
    }
}
