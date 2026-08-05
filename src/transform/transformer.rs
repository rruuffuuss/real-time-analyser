use crate::transform::merger::Merger;

use realfft::{RealFftPlanner, RealToComplex};
use rustfft::num_complex::Complex;
use std::sync::Arc;

pub struct Transformer {
    pub(super) input_samples: usize,
    merger: Box<dyn Merger>,
    fft: Arc<dyn RealToComplex<f32>>,
    scratch: Box<Vec<Complex<f32>>>,
    output: Box<Vec<Complex<f32>>>,
}

impl Transformer {
    pub fn new(input_samples: usize, merger: Box<dyn Merger>) -> Self {
        let mut planner = RealFftPlanner::new();
        let fft = planner.plan_fft_forward(input_samples);

        let scratch = Box::new(fft.make_scratch_vec());
        let output = Box::new(fft.make_output_vec());

        Self {
            input_samples,
            merger,
            fft,
            scratch,
            output,
        }
    }

    #[inline(always)]
    pub fn transform(&mut self, input: &mut [f32], spectrum: &mut [f32]) {
        /*if self.input_buffer.len() != self.input_samples {
            panic!("input size is unexpected")
        };*/

        self.fft
            .process_with_scratch(input, &mut self.output, &mut self.scratch);

        self.merger.merge_into_slice(&self.output, spectrum)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::transform::linear_merger::LinearMerger;
    use crate::transform::merger::{aggregator, realisor};

    #[test]
    fn test_single_frequency_7_hz() {
        let mut input = vec![
            0.0_f32, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0,
            0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0,
        ];

        let mut t = Transformer::new(
            input.len(),
            Box::new(
                LinearMerger::<aggregator::Sum<realisor::MagnitudeSquared>>::new(
                    input.len(),
                    input.len() / 2,
                ),
            ),
        );

        let mut result = vec![0_f32; input.len() / 2 + 1];
        t.transform(&mut input, &mut result);

        print!("{:?}", result);

        let max_index = result
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index)
            .unwrap();

        assert_eq!(7_usize, max_index)
    }

    /*
    #[test]
    fn test_single_frequency_7_hz_split_slices() {
        let input1 = vec![0.0_f32, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0];

        let input2 = vec![
            -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0,
            -1.0, 0.0,
        ];

        let mut t = Transformer::new(
            input1.len() + input2.len(),
            Box::new(LinearMerger::new(
                (input1.len() + input2.len()),
                (input1.len() + input2.len()) / 2,
            )),
        );

        let result = t.transform_split((&input1, &input2));

        print!("{:?}", result);

        let max_index = result
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index)
            .unwrap();

        assert_eq!(7_usize, max_index)
    }
    */
}
