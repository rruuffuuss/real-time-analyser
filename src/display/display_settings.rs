use crate::display::{
    display::Display, double_bar_display::DoubleBarDisplay, single_bar_display::SingleBarDisplay,
};
use crossterm::{execute, terminal};
use serde::Deserialize;
use std::{
    default,
    io::{self, stdout},
};

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DisplaySettings {
    #[serde(default = "default_width")]
    display_width: u16,
    #[serde(default = "default_height")]
    display_height: u16,

    #[serde(default = "default_char_set")]
    char_set: DisplayCharSet,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum DisplayCharSet {
    SingleBar {
        increments: Vec<char>,
        #[serde(default = "default_bar_width")]
        bar_width: u16,
    },
    DoubleBar {
        increments: Vec<Vec<char>>,
    },
}

impl DisplaySettings {
    pub fn build(mut self) -> Box<dyn Display> {
        execute!(
            stdout(),
            terminal::SetSize(self.display_width, self.display_height)
        )
        .unwrap_or_else(|error| {
            eprintln!(
                "using current terminal size as terminal resize failed: {}",
                error
            );
            self.display_height = default_height();
            self.display_width = default_width();
        });

        match self.char_set {
            DisplayCharSet::SingleBar {
                increments,
                bar_width,
            } => Box::new(SingleBarDisplay::new(
                self.display_width,
                self.display_height,
                bar_width,
                increments,
            )),
            DisplayCharSet::DoubleBar { increments } => Box::new(DoubleBarDisplay::new(
                self.display_width,
                self.display_height,
                increments,
            )),
        }
    }
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            display_width: default_width(),
            display_height: default_height(),
            char_set: default_char_set(),
        }
    }
}

fn default_width() -> u16 {
    let (display_width, _) = terminal::size().unwrap_or((80, 24));
    display_width
}

fn default_height() -> u16 {
    let (_, display_height) = terminal::size().unwrap_or((80, 24));
    display_height
}

fn default_bar_width() -> u16 {
    1
}

fn default_char_set() -> DisplayCharSet {
    DisplayCharSet::SingleBar {
        increments: vec![' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'],
        bar_width: default_bar_width(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config, ConfigError, File, FileFormat};

    fn parse(yaml: &str) -> Result<DisplaySettings, ConfigError> {
        Config::builder()
            .add_source(File::from_str(yaml, FileFormat::Yaml))
            .build()?
            .try_deserialize()
    }

    #[test]
    fn loads_display_settings() {
        let settings = parse("display_width: 3\ndisplay_height: 2\n").unwrap();

        assert_eq!(settings.display_width, 3);
        assert_eq!(settings.display_height, 2);
    }

    #[test]
    fn builds_display_with_configured_dimensions() {
        let settings = parse("display_width: 3\ndisplay_height: 2\n").unwrap();
        let mut display = settings.build();

        assert_eq!(display.ideal_bar_count(), 3);
    }

    #[test]
    fn rejects_unknown_display_settings() {
        let result = parse("display_width: 3\ndisplay_height: 2\ncolour: blue\n");

        assert!(result.is_err());
    }
}
