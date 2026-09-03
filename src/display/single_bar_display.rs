use super::display::{Display, draw_frame};
use super::display_core::DisplayCore;

/*
 * in the code 'height' generally refers to the height of a bar in characters
 * and increment refers to which of the 9 block chars best represents the value
 *
 * I have a hunch there is currently a bug with the increments will fix once a more significant prototype exists
 */

pub struct SingleBarDisplay {
    core: DisplayCore,
    increment_chars: Vec<char>,
}

impl Display for SingleBarDisplay {
    fn display(&mut self, input: &[f32]) {
        draw_frame(self.render_frame(input))
            .expect("error drawing the frame, you string was likely malformed");
    }

    #[inline(always)]
    fn ideal_bar_count(&self) -> usize {
        (self.core.width / self.core.bar_width) as usize
    }
}

impl SingleBarDisplay {
    pub fn new(width: u16, height: u16, bar_width: u16, chars: Vec<char>) -> SingleBarDisplay {
        SingleBarDisplay {
            core: DisplayCore::new(width, height, bar_width, chars.len() as u16),
            increment_chars: chars,
        }
    }

    fn render_frame(&mut self, input: &[f32]) -> &String {
        self.core.calculate_heights_increments(input);

        // loop through each column
        // loop through each row in the column
        // start by setting the empty char, set the increment when current row = bar height, then set bar char until end
        for (buffer, (height, increment)) in self
            .core
            ._column_buffers
            .iter_mut()
            .zip(self.core.heights.iter().zip(self.core.increments.iter()))
        {
            let mut default = self.increment_chars.first().unwrap().clone();
            for (char, row) in (buffer).iter_mut().zip((0..self.core.height).rev()) {
                if row == *height {
                    *char = self.increment_chars[*increment as usize];
                    default = self.increment_chars.last().unwrap().clone();
                } else {
                    *char = default;
                }
            }
        }

        self.core.push_column_buffers_to_string_buffer();

        &self.core._string_buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_chars() -> Vec<char> {
        vec![' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█']
    }

    #[test]
    fn test_heights() {
        let mut d = SingleBarDisplay::new(5, 10, 1, test_chars());

        let frame = d.render_frame(&vec![0.99_f32, 0.0, 0.7, 0.4, 0.9]);

        let bars = String::from(concat!(
            "\x1b[H",
            "█    \r\n",
            "█   █\r\n",
            "█   █\r\n",
            "█ █ █\r\n",
            "█ █ █\r\n",
            "█ █ █\r\n",
            "█ ███\r\n",
            "█ ███\r\n",
            "█ ███\r\n",
            "█ ███",
        ));

        assert_eq!(&bars, frame)
    }

    #[test]
    fn test_increments_and_heights() {
        let mut d = SingleBarDisplay::new(5, 10, 1, test_chars());

        let frame = d.render_frame(&vec![0.778_f32, 0.0334, 0.7, 0.412, 0.978]);

        let bars = String::from(concat!(
            "\x1b[H",
            "    ▇\r\n",
            "    █\r\n",
            "▇   █\r\n",
            "█ █ █\r\n",
            "█ █ █\r\n",
            "█ █▁█\r\n",
            "█ ███\r\n",
            "█ ███\r\n",
            "█ ███\r\n",
            "█▃███",
        ));

        assert_eq!(&bars, frame)
    }

    #[test]
    fn test_bar_width() {
        let mut d = SingleBarDisplay::new(15, 10, 3, test_chars());

        let frame = d.render_frame(&vec![0.778_f32, 0.0334, 0.7, 0.412, 0.978]);

        let bars = String::from(concat!(
            "\x1b[H",
            "            ▇▇▇\r\n",
            "            ███\r\n",
            "▇▇▇         ███\r\n",
            "███   ███   ███\r\n",
            "███   ███   ███\r\n",
            "███   ███▁▁▁███\r\n",
            "███   █████████\r\n",
            "███   █████████\r\n",
            "███   █████████\r\n",
            "███▃▃▃█████████",
        ));

        assert_eq!(&bars, frame)
    }
}
