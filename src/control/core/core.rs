use crate::display::display::Display;
use crate::normalise::normaliser::Normaliser;
use crate::transform::merger::Merger;
use crate::transform::transformer::Transformer;

pub struct ControlCore {
    ///number of samples to analyse, must be power of 2
    pub(crate) transform_size: usize,
    pub(crate) sample_rate: u32,
    ///frequency of analysis & display update events in Hz
    pub(crate) target_framerate: u16,
    /// number of output 'bins' for the output frequency spectrum
    /// just using equidistant bins for now, will update for musical notes later
    /// down the line this may end up as functions defining the window size and frequency for each output bin individually
    /// would need to buffer for the largest
    pub(crate) display: Display,
    pub(crate) normaliser: Normaliser,
    /*///number of output graphs
    ///display_grid: (u8, u8),
    ///channel map for input channels to output graphs
    ///input channels within an inner vector are averaged together
    // channel_map: Vec<(Vec<u8>, u8)>,*/
    pub transformer: Transformer,
}

impl ControlCore {
    pub fn new(
        transform_size: usize,
        sample_rate: u32,
        target_framerate: u16,
        display: Display,
        merger: Box<dyn Merger>,
    ) -> Self {
        let mut normaliser = Normaliser::new(1.0_f32, 1.2, 0.1);

        let transformer = Transformer::new(transform_size, merger);

        Self {
            transform_size,
            sample_rate,
            target_framerate,
            display,
            normaliser,
            transformer,
        }
    }
}
