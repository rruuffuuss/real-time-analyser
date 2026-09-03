use super::display::{Display, draw_frame};

/*
 * in the code 'height' generally refers to the height of a bar in characters
 * and increment refers to which of the 9 block chars best represents the value
 *
 * I have a hunch there is currently a bug with the increments will fix once a more significant prototype exists
 */

use std::io::{self, Write};

pub struct DisplayCore {
    pub(super) width: u16,
    pub(super) height: u16,
    pub(super) _string_buffer: String,
    pub(super) _column_buffers: Vec<Vec<char>>,

    ///the 'height' of each bar
    pub(super) heights: Vec<u16>,
    ///the 'increment' within the character at the heights height
    pub(super) increments: Vec<u16>,

    // easier to keep as f32 rather than convert for every input
    pub(super) _height_multiplier: f32,
    pub(super) _increment_multiplier: f32,
    pub(super) _char_increments: f32,

    pub(super) bar_width: u16,
}

impl DisplayCore {
    pub fn new(width: u16, height: u16, bar_width: u16, char_increments: u16) -> DisplayCore {
        //charset may be non ascii, so allocating with capacity is futile here
        //buffer should be reallocated to a useful size in the first few iterations
        let mut _string_buffer = String::new();

        let bar_count = width / bar_width;
        let mut _column_buffers: Vec<Vec<char>> =
            (0..bar_count).map(|_| vec![' '; height as usize]).collect();

        let heights = vec![0; bar_count as usize];
        let increments = heights.clone();
        let _height_multiplier: f32 = height.into();
        let _increment_multiplier: f32 = (height * char_increments).into();
        let _char_increments: f32 = char_increments.into();

        DisplayCore {
            width,
            height,
            _string_buffer,
            _column_buffers,
            heights,
            increments,
            _height_multiplier,
            _increment_multiplier,
            _char_increments,
            bar_width,
        }
    }

    ///calculate the height (in rows) and increment (char height within a row) for each bar
    #[inline(always)]
    pub(super) fn calculate_heights_increments(&mut self, input: &[f32]) {
        if (input.len() != self.heights.len()) {
            panic!("graph width cannot currently change at runtime")
        };

        // construct vectors holding the heights of each bar
        // heights consist of the number of characters high the bar is and the "increment" char to use
        input
            .iter()
            .zip(self.heights.iter_mut().zip(self.increments.iter_mut()))
            .for_each(|(i, (b, ib))| {
                *b = (self._height_multiplier * i) as u16;
                *ib = ((self._increment_multiplier * i) % self._char_increments) as u16;
            });
    }

    #[inline(always)]
    pub fn push_column_buffers_to_string_buffer(&mut self) {
        self._string_buffer = String::from("\x1b[H");

        for row in 0..self.height {
            for buffer in self._column_buffers.iter() {
                for _ in 0..self.bar_width {
                    self._string_buffer.push(buffer[row as usize]);
                }
            }
            self._string_buffer.push_str("\r\n");
        }

        &self._string_buffer.pop();
        &self._string_buffer.pop();
    }
}
