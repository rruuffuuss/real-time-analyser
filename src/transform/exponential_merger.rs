use std::marker::PhantomData;

use rustfft::num_complex::Complex;

use crate::transform::merger::Merger;

use super::merger::{aggregator::Aggregator, realisor::Realisor};

///exponential mode where the number of fft 'frequency bins' per output 'bar' increases exponentially
///this replicates the way the human ear hears sounds and thus traditional musical notes as the frequency of sound doubles with each octave
pub struct ExponentialMerger<Ag: Aggregator> {
    ///number of fft output frequency bins to merge for each bar in the output graph
    bins_per_bar: Vec<usize>,

    /// the number of output bars (equal to bins_per_bar.len())
    output_bars: usize,

    /// first frequency bin to use
    start_bin: usize,
    /// last frequency bin to use
    ///index into frequency bins which are useful (as real input FFT mirrors halfway so we only want first half)
    end_bin: usize,

    _aggregate_strategy: PhantomData<Ag>,
}

impl<Ag: Aggregator> Merger for ExponentialMerger<Ag> {
    fn merge_into_slice(&self, frequency_bins: &[Complex<f32>], output: &mut [f32]) {
        let mut start: usize = self.start_bin;
        let mut end: usize = start;

        for (bar_width, output) in self.bins_per_bar.iter().zip(output.iter_mut()) {
            end += bar_width;
            *output = Ag::aggregate(&frequency_bins[start..end]);
            start = end;
        }
    }

    /*fn merge(&self, frequency_bins: &[Complex<f32>]) -> Vec<f32> {
        let mut bars: Vec<f32> = Vec::with_capacity(self.output_bars);

        let mut start: usize = self.start_bin;
        let mut end: usize = start;

        for bar_width in &self.bins_per_bar {
            end += bar_width;
            bars.push(
                frequency_bins[start..end]
                    .iter()
                    .map(|c| c.norm_sqr())
                    .sum::<f32>()
                    / bar_width.clone() as f32,
            );
            start = end;
        }

        bars
    }*/
}

impl<Ag: Aggregator> ExponentialMerger<Ag> {
    ///create an exponential merger with default settings.
    ///it is unlikely that the default settings will produce perfect tone or semitone bars
    pub fn new_auto(
        input_bins: usize,
        output_bars: usize,
        sample_rate: u32,
    ) -> ExponentialMerger<Ag> {
        Self::new_custom_function(
            input_bins,
            output_bars,
            sample_rate,
            DEFAULT_TUNING_FREQUENCY,
            output_bars / DEFAULT_OCTAVE_RANGE as usize,
            DEFAULT_STARTING_NOTE_OFFSET,
        )
    }

    ///create a new exponential merger defining a custom function for distributing frequency bins to bars
    ///the highest frequency will be equal to the lowest frequency * 2^(number of output bars / bars per octave)
    /// the lowest frequency will be the frequency of the note offset below the tuning frequency
    pub fn new_custom_function(
        input_bins: usize,
        output_bars: usize,
        sample_rate: u32,
        tuning_frequency: f64,
        bars_per_octave: usize,
        starting_note_offset: isize,
    ) -> ExponentialMerger<Ag> {
        let builder = ExponentialMergerBuildHelper {
            input_bins,
            output_bars,
            sample_rate,
            tuning_frequency,
            bars_per_octave,
            starting_note_offset,
            starting_bin_offset: starting_note_offset as f64 - 0.5,
        };

        let mut bins_per_bar: Vec<usize> = Vec::with_capacity(output_bars);

        let useful_bins = input_bins / 2;
        let start = builder.frequency_to_bin(builder.note_to_frequency(-1_f64));

        let mut start_bin = start;

        let mut bar = 0;

        loop {
            let end_bin = builder.frequency_to_bin(builder.note_to_frequency(bar as f64));

            if bar >= output_bars || end_bin > useful_bins {
                break;
            }

            bins_per_bar.push(end_bin - start_bin);
            start_bin = end_bin;
            bar += 1;
        }

        ExponentialMerger {
            bins_per_bar,
            output_bars,

            start_bin: start,
            end_bin: useful_bins,
            _aggregate_strategy: PhantomData::<Ag>,
        }
    }
}

const DEFAULT_OCTAVE_RANGE: u32 = 8;
const DEFAULT_TUNING_FREQUENCY: f64 = 440_f64;
const DEFAULT_STARTING_NOTE_OFFSET: isize = -48;

///"notes" and "bars" are sort of used interchageably here since the assumption is that the output bars represent notes
struct ExponentialMergerBuildHelper {
    input_bins: usize,
    output_bars: usize,

    ///pipewire sample rate
    sample_rate: u32,

    /// the number of notes in an octave (i.e. the number of notes for each doubling of frequency)
    /// set this to 12 to display semitones or 6 to average to full tones etc
    bars_per_octave: usize,

    ///frequency of a note that all other notes will set relative to
    tuning_frequency: f64,

    ///how many bars below the tuning frequency your first note is
    ///this only really exists so that the tuning frequency can be set independently of the first note to display- e.g. tune A4 at 440hz but the "0th" note is A0 (offset -48) at 27.5hz
    ///there is no difference in practice between doing this and setting the tuning frequency at 27.5hz with zero offset
    starting_note_offset: isize,

    ///this is just the starting note offset -0.5 since we want a bar to represent frequency bins from halfway between the previous note to halfway until the next note.
    starting_bin_offset: f64,
}

impl ExponentialMergerBuildHelper {
    fn note_to_frequency(&self, frequency: f64) -> f64 {
        self.tuning_frequency
            * 2_f64
                .powf((frequency + self.starting_bin_offset as f64) / self.bars_per_octave as f64)
    }

    fn frequency_to_note(&self, note: f64) -> f64 {
        (self.bars_per_octave as f64 * f64::ln(note / self.tuning_frequency as f64) / 2.0_f64.ln())
            - self.starting_bin_offset as f64
    }

    fn bin_to_frequency(&self, bin: usize) -> f64 {
        ((self.sample_rate as usize / self.input_bins) * bin) as f64
    }

    fn frequency_to_bin(&self, frequency: f64) -> usize {
        ((self.input_bins as f64 / self.sample_rate as f64) * frequency as f64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transform::merger::{aggregator, realisor};

    #[test]
    fn exponential_merger_configuration_and_merge() {
        let merger: ExponentialMerger<aggregator::Mean<realisor::MagnitudeSquared>> =
            ExponentialMerger {
                bins_per_bar: vec![1, 2, 3],
                output_bars: 3,
                start_bin: 0,
                end_bin: 6,
                _aggregate_strategy: PhantomData,
            };

        let frequency_bins = vec![
            Complex::new(3.0, 4.0),
            Complex::new(1.0, 2.0),
            Complex::new(2.0, 0.0),
            Complex::new(0.5, 0.5),
            Complex::new(0.0, 3.0),
            Complex::new(-2.0, -1.0),
        ];

        let mut output = vec![0.0; 3];
        merger.merge_into_slice(&frequency_bins, &mut output);

        assert_eq!(output, vec![25.0, 4.5, 4.8333335]);
    }

    #[test]
    fn exponential_merger_builder_configuration_merge() {
        let merger =
            ExponentialMerger::<aggregator::Mean<realisor::MagnitudeSquared>>::new_custom_function(
                64, 4, 64, 2.0, 1, 0,
            );

        assert_eq!(merger.bins_per_bar, vec![1, 1, 3, 6]);
        assert_eq!(merger.output_bars, 4);
        assert_eq!(merger.start_bin, 0);
        assert_eq!(merger.end_bin, 32);

        let mut frequency_bins = vec![Complex::new(0.0, 0.0); 64];
        frequency_bins[0] = Complex::new(1.0, 0.0);
        frequency_bins[1] = Complex::new(2.0, 0.0);
        frequency_bins[2..5].fill(Complex::new(3.0, 0.0));
        frequency_bins[5..11].fill(Complex::new(4.0, 0.0));

        let mut output = vec![0.0; 4];
        merger.merge_into_slice(&frequency_bins, &mut output);

        assert_eq!(output, vec![1.0, 4.0, 9.0, 16.0]);
    }
}
