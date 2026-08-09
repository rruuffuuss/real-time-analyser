use std::f64;

use crate::window::window_function::Window;

use std::collections::VecDeque;

pub struct FirFilter {
    pub taps: Vec<f32>,
}

impl FirFilter {
    pub fn new(taps: u32, normalised_cutoff: f64, window: Window<f64>) -> FirFilter {
        let centre = ((taps - 1) as f64 / 2_f64);

        let ideal_impulse_response = |n: f64| sinc_cutoff(n - centre, normalised_cutoff);

        //get the window
        let mut window = window.samples;

        //multiply each tap by the ideal impulse response
        window
            .iter_mut()
            .enumerate()
            .for_each(|(n, f)| *f *= ideal_impulse_response(n as f64));

        //normalise the window so it sums to 1 and create the FirFilter
        let sum: f64 = window.iter().sum();
        FirFilter {
            taps: window.iter().map(|n| (n / sum) as f32).collect(),
        }
    }

    #[inline(always)]
    pub fn half_band_into_queue(
        &self,
        tap_num: usize,
        new_samples: usize,
        //source: &VecDeque<f32>,
        source: &[f32],
        target: &mut VecDeque<f32>,
    ) {
        /* an alternate implementation to this would be using something akin to .windows() over
         * the new samples in the source VecDeque and multiplying each sample in the window against zipped fir_filter taps
         * and then summing
         *
         * the discontinuity of a VecDeque means I can't figure out a clean way of implementing this at the moment.
         * another issue is that every other window would be discarded, since the FIR filter is half band
         */

        for end_sample in ((source.len() - new_samples)..source.len()).step_by(2) {
            target.push_back(
                source[end_sample - tap_num..end_sample]
                    .iter()
                    .zip(&self.taps)
                    .fold(0_f32, |acc, (xn, b)| b.mul_add(*xn, acc)),
            );
        }
    }
}

fn sinc_cutoff(n: f64, cutoff: f64) -> f64 {
    if n == 0_f64 {
        cutoff
    } else {
        (n * cutoff * f64::consts::PI).sin() / (n * f64::consts::PI)
    }
}
