use super::display::{Display, draw_frame};
use super::display_core::DisplayCore;

pub struct DoubleBarDisplay {
    core: DisplayCore,
    increment_chars: Vec<Vec<char>>,
}

impl Display for DoubleBarDisplay {
    fn display(&mut self, input: &[f32]) {
        draw_frame(self.render_frame(input))
            .expect("error drawing the frame, you string was likely malformed");
    }

    #[inline(always)]
    fn ideal_bar_count(&self) -> usize {
        (self.core.width / self.core.bar_width) as usize
    }
}

impl DoubleBarDisplay {
    pub fn new(width: u16, height: u16, chars: Vec<Vec<char>>) -> DoubleBarDisplay {
        let mut core = DisplayCore::new(width * 2, height, 1, chars.len() as u16);

        core._column_buffers.drain((width) as usize..);

        DoubleBarDisplay {
            core,
            increment_chars: chars,
        }
    }

    fn render_frame(&mut self, input: &[f32]) -> &String {
        self.core.calculate_heights_increments(input);

        // loop through each column
        // loop through each row in the column
        // start by setting the empty char, set the increment when current row = bar height, then set bar char until end
        for (buffer, (height, increment)) in self.core._column_buffers.iter_mut().zip(
            self.core
                .heights
                .chunks_exact(2)
                .zip(self.core.increments.chunks_exact(2)),
        ) {
            let mut default_index = (0 as usize, 0);
            let mut default = get_at_2d(&self.increment_chars, (0, 0));

            //if both sub bars are within the same character height we can process them together
            if height[0] == height[1] {
                for (char, row) in (buffer).iter_mut().zip((0..self.core.height).rev()) {
                    if row == height[0] {
                        *char = get_at_2d(
                            &self.increment_chars,
                            (increment[1] as usize, increment[0] as usize),
                        );
                        default = get_at_2d(
                            &self.increment_chars,
                            (
                                self.increment_chars.len() - 1,
                                self.increment_chars.len() - 1,
                            ),
                        );
                    } else {
                        *char = default;
                    }
                }
            }
            //otherwise we need to process them independently
            else {
                for (char, row) in (buffer).iter_mut().zip((0..self.core.height).rev()) {
                    if row == height[1] {
                        *char = get_at_2d(
                            &self.increment_chars,
                            (increment[1] as usize, default_index.1),
                        );
                        default_index.0 = self.increment_chars.len() - 1;
                        default = get_at_2d(&self.increment_chars, default_index);
                    } else if row == height[0] {
                        *char = get_at_2d(
                            &self.increment_chars,
                            (default_index.0, increment[0] as usize),
                        );
                        default_index.1 = self.increment_chars.len() - 1;
                        default = get_at_2d(&self.increment_chars, default_index);
                    } else {
                        *char = default;
                    }
                }
            }
        }

        self.core.push_column_buffers_to_string_buffer();

        &self.core._string_buffer
    }
}

fn get_at_2d<T>(v: &Vec<Vec<T>>, (x, y): (usize, usize)) -> T
where
    T: Copy,
{
    v[x][y]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_chars() -> Vec<Vec<char>> {
        vec![
            vec![' ', '⡀', '⡄', '⡆', '⡇'],
            vec!['⢀', '⣀', '⣄', '⣆', '⣇'],
            vec!['⢠', '⣠', '⣤', '⣦', '⣧'],
            vec!['⢰', '⣰', '⣴', '⣶', '⣷'],
            vec!['⢸', '⣸', '⣼', '⣾', '⣿'],
        ]
    }

    #[test]
    fn test_heights() {
        let mut d = DoubleBarDisplay::new(5, 10, test_chars());

        let frame = d.render_frame(&[0.99_f32, 0.99, 0.0, 0.0, 0.7, 0.7, 0.4, 0.4, 0.9, 0.9]);

        let bars = String::from(concat!(
            "\x1b[H",
            "⣿    \r\n",
            "⣿   ⣿\r\n",
            "⣿   ⣿\r\n",
            "⣿ ⣿ ⣿\r\n",
            "⣿ ⣿ ⣿\r\n",
            "⣿ ⣿ ⣿\r\n",
            "⣿ ⣿⣿⣿\r\n",
            "⣿ ⣿⣿⣿\r\n",
            "⣿ ⣿⣿⣿\r\n",
            "⣿ ⣿⣿⣿",
        ));

        assert_eq!(&bars, frame)
    }

    #[test]
    fn test_increments_and_heights() {
        let mut d = DoubleBarDisplay::new(5, 10, test_chars());

        let frame = d.render_frame(&[
            0.778_f32, 0.0334, 0.7, 0.412, 0.978, 0.778, 0.0334, 0.7, 0.412, 0.978,
        ]);

        let bars = String::from(concat!(
            "\x1b[H",
            "  ⡆ ⢰\r\n",
            "  ⡇ ⢸\r\n",
            "⡆ ⣷ ⢸\r\n",
            "⡇⡇⣿⢸⢸\r\n",
            "⡇⡇⣿⢸⢸\r\n",
            "⡇⡇⣿⢸⢸\r\n",
            "⡇⣿⣿⢸⣿\r\n",
            "⡇⣿⣿⢸⣿\r\n",
            "⡇⣿⣿⢸⣿\r\n",
            "⣇⣿⣿⣸⣿",
        ));

        assert_eq!(&bars, frame)
    }

    #[test]
    fn test_two_bars_per_character() {
        let mut d = DoubleBarDisplay::new(5, 1, test_chars());

        assert_eq!(d.ideal_bar_count(), 10);

        let frame = d.render_frame(&[0.99_f32, 0.0, 0.0, 0.99, 0.99, 0.99, 0.0, 0.0, 0.5, 0.25]);

        assert_eq!("\x1b[H⡇⢸⣿ ⣄", frame)
    }
}
