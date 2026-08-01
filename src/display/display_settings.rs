use crate::display::display::Display;
use crossterm::terminal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DisplaySettings {
    display_width: u16,
    display_height: u16,
}

impl DisplaySettings {
    pub fn build(&self) -> Display {
        Display::new(self.display_width, self.display_height)
    }
}

impl Default for DisplaySettings {
    fn default() -> Self {
        let (display_width, display_height) = terminal::size().unwrap_or((80, 24));

        Self {
            display_width,
            display_height,
        }
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
        assert_eq!(display.render_frame(&[0.0; 3]), "\x1b[H   \r\n   ");
    }

    #[test]
    fn rejects_unknown_display_settings() {
        let result = parse("display_width: 3\ndisplay_height: 2\ncolour: blue\n");

        assert!(result.is_err());
    }
}
