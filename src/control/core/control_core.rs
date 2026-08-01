use crate::display::{display::Display, display_settings::DisplaySettings};

use crate::normaliser::Normaliser;
use crate::transform::merger_settings::MergerSettings;
use crate::transform::transformer::Transformer;

pub struct ControlCore {
    ///number of samples to analyse, must be power of 2
    pub(super) transform_size: usize,
    pub(super) sample_rate: u32,
    ///frequency of analysis & display update events in Hz
    pub(super) target_framerate: u16,
    /// number of output 'bins' for the output frequency spectrum
    /// just using equidistant bins for now, will update for musical notes later
    /// down the line this may end up as functions defining the window size and frequency for each output bin individually
    /// would need to buffer for the largest
    pub(super) display: Display,
    pub(super) normaliser: Normaliser,
    /*///number of output graphs
    ///display_grid: (u8, u8),
    ///channel map for input channels to output graphs
    ///input channels within an inner vector are averaged together
    // channel_map: Vec<(Vec<u8>, u8)>,*/
    pub merger_settings: MergerSettings,
    pub min_frequency: u32,
    pub max_frequency: u32,
}

impl ControlCore {
    pub fn new(
        transform_size: usize,
        sample_rate: u32,
        target_framerate: u16,
        display: DisplaySettings,
        merger_settings: MergerSettings,
    ) -> Self {
        let display: Display = display.build();

        let mut normaliser = Normaliser::new(1.0_f32);

        Self {
            transform_size,
            sample_rate,
            target_framerate,
            display,
            normaliser,
            merger_settings,
            min_frequency: 0,
            max_frequency: 0,
        }
    }
}
