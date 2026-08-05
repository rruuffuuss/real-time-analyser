use std::marker::PhantomData;

use rustfft::num_complex::Complex;

use crate::transform::merger::aggregator::Aggregator;

use super::merger::Merger;

///linear mode where each output 'bar' is created via averaging a fixed number of fft 'frequency bins'
pub struct LinearMerger<Ag: Aggregator> {
    ///number of fft output frequency bins to merge into each bar in the output graph
    bins_per_bar: usize,
    ///index into frequency bins which are useful (as real input FFT mirros halfway so we only want first half)
    useful_bins: usize,
    _aggregate_strategy: PhantomData<Ag>,
}

impl<Ag: Aggregator> Merger for LinearMerger<Ag> {
    /*fn merge(&self, frequency_bins: &[Complex<f32>]) -> Vec<f32> {
        // copy the real part of the transform into the output
        // could potentially add other bar merging options like max or a weighted average
        // no point dividing for average since bars are normalised anyway
        frequency_bins[..self.useful_bins]
            .chunks_exact(self.bins_per_bar)
            .map(|bins| bins.iter().map(|bin| bin.norm_sqr()).sum::<f32>())
            .collect()
    }*/

    fn merge_into_slice(&self, frequency_bins: &[Complex<f32>], output: &mut [f32]) {
        frequency_bins[..self.useful_bins]
            .chunks_exact(self.bins_per_bar)
            .zip(output)
            .for_each(|(i, o)| *o = Ag::aggregate(i));
    }
}

impl<Ag: Aggregator> LinearMerger<Ag> {
    pub fn new(input_bins: usize, output_bars: usize) -> LinearMerger<Ag> {
        // only half the output frequency bins are used since fourier transform of real only input is mirrored
        let useful_bins = input_bins / 2;
        let bins_per_bar = useful_bins / output_bars;

        Self {
            bins_per_bar,
            useful_bins,
            _aggregate_strategy: PhantomData::<Ag>,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transform::merger::{aggregator, realisor};

    #[test]
    fn linear_merger_configuration_and_merge() {
        let merger = LinearMerger::<aggregator::Sum<realisor::MagnitudeSquared>>::new(12, 3);

        assert_eq!(merger.bins_per_bar, 2);
        assert_eq!(merger.useful_bins, 6);

        let frequency_bins = vec![
            Complex::new(1.0, 0.0),
            Complex::new(0.0, 2.0),
            Complex::new(3.0, 4.0),
            Complex::new(-1.0, 2.0),
            Complex::new(2.0, -2.0),
            Complex::new(0.5, 0.5),
            Complex::new(10.0, 10.0),
            Complex::new(10.0, 10.0),
            Complex::new(10.0, 10.0),
            Complex::new(10.0, 10.0),
            Complex::new(10.0, 10.0),
            Complex::new(10.0, 10.0),
        ];

        let mut output = vec![0.0; 3];
        merger.merge_into_slice(&frequency_bins, &mut output);

        assert_eq!(output, vec![5.0, 30.0, 8.5]);
    }
}
