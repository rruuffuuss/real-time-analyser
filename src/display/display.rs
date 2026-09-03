/*
 * in the code 'height' generally refers to the height of a bar in characters
 * and increment refers to which of the 9 block chars best represents the value
 *
 * I have a hunch there is currently a bug with the increments will fix once a more significant prototype exists
 */

use std::io::{self, Write};

pub trait Display {
    fn display(&mut self, input: &[f32]);
    fn ideal_bar_count(&self) -> usize;
}

#[inline(always)]
pub fn draw_frame(frame: &str) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(frame.as_bytes())?;
    stdout.flush()
}
