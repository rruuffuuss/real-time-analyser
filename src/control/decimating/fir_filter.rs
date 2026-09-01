use std::f64;

use crate::{
    control::unchecked_double_mapped_queue::UncheckedDoubleMappedQueue,
    window::window_function::Window,
};

use std::collections::VecDeque;

pub struct FirFilter {
    pub taps: Vec<[f32; f32s_simd_max()]>,
    ///tap_num should be used when the number of taps is needed
    ///taps.len() will return the number of SIMD processing arrays the taps are stored in, which is unlikely to be useful
    pub tap_num: usize,
}

impl FirFilter {
    pub fn new(taps: u32, normalised_cutoff: f64, window: Window<f64>) -> FirFilter {
        let centre = ((taps - 1) as f64 / 2_f64);

        let ideal_impulse_response = |n: f64| sinc_cutoff(n - centre, normalised_cutoff);

        //get the window
        let mut window = window.samples;

        //multiply each sample by the ideal impulse response
        window
            .iter_mut()
            .enumerate()
            .for_each(|(n, f)| *f *= ideal_impulse_response(n as f64));

        //normalise the window so it sums to 1
        let sum: f64 = window.iter().sum();
        window.iter_mut().for_each(|t| *t = *t / sum);

        // create a vector with n dummy taps where n + window.len() = m * f32s_simd_max
        // using a few dummy vectors to align the tap number with a multiple of the SIMD size is faster than accounting for overflow in a hot loop
        // when there is no SIMD (size is 1) zero dummy taps are added, having no effect.
        let mut dummy_taps =
            vec![0_f64; (f32s_simd_max() - (window.len() % f32s_simd_max())) % f32s_simd_max()];
        dummy_taps.extend(window.iter());
        let taps = dummy_taps;

        let tap_num = taps.len();

        let mut tap_arrays: Vec<[f32; f32s_simd_max()]> =
            Vec::with_capacity(taps.len() / f32s_simd_max());

        for parallel_chunk in taps.chunks_exact(f32s_simd_max()) {
            let mut parallel_array = [0_f32; f32s_simd_max()];

            for (c, a) in parallel_chunk.iter().zip(parallel_array.iter_mut()) {
                *a = *c as f32;
            }

            tap_arrays.push(parallel_array);
        }

        FirFilter {
            taps: tap_arrays,
            tap_num,
        }
    }

    #[inline(always)]
    pub fn half_band_into_udmq(
        &self,
        new_samples: usize,
        //source: &VecDeque<f32>,
        source: &[f32],
        target: &mut UncheckedDoubleMappedQueue<f32>,
    ) {
        /* an alternate implementation to this would be using something akin to .windows() over
         * the new samples in the source VecDeque and multiplying each sample in the window against zipped fir_filter taps
         * and then summing
         * issue is that every other window would be discarded, since the FIR filter is half band
         */

        for end_sample in ((source.len() - new_samples)..source.len()).step_by(2) {
            //use multiple accumulators to utilise loop vectorisation
            let mut accumulators = [0_f32; f32s_simd_max()];

            //loop through accumulator sized chunks of source
            source[end_sample - self.tap_num..end_sample]
                .chunks_exact(f32s_simd_max())
                .zip(&self.taps)
                .for_each(|(samples, taps)| {
                    //add each sample * tap to the corresponding accumulator independently
                    accumulators
                        .iter_mut()
                        .zip(samples.iter().zip(taps))
                        .for_each(|(acc, (sample, tap))| *acc = sample.mul_add(*tap, *acc))
                });

            //finally combine accumulators
            target.push_back(sum_reduction_tree(accumulators));
            //target.push_back(accumulators.iter().sum());
        }
    }
}

#[inline(always)]
fn sum_reduction_tree(accumulators: [f32; f32s_simd_max()]) -> f32 {
    if cfg!(target_feature = "avx512f") {
        (((accumulators[0] + accumulators[8]) + (accumulators[4] + accumulators[12]))
            + ((accumulators[2] + accumulators[10]) + (accumulators[6] + accumulators[14])))
            + (((accumulators[1] + accumulators[9]) + (accumulators[5] + accumulators[13]))
                + ((accumulators[3] + accumulators[11]) + (accumulators[7] + accumulators[15])))
    } else if cfg!(target_feature = "avx") {
        ((accumulators[0] + accumulators[4]) + (accumulators[2] + accumulators[6]))
            + ((accumulators[1] + accumulators[5]) + (accumulators[3] + accumulators[7]))
    } else if cfg!(target_feature = "sse") {
        (accumulators[0] + accumulators[2]) + (accumulators[1] + accumulators[3])
    } else {
        accumulators[0]
    }
}

const fn f32s_simd_max() -> usize {
    if cfg!(target_feature = "avx512f") {
        16
    } else if cfg!(target_feature = "avx") {
        8
    } else if cfg!(target_feature = "sse") {
        4
    } else {
        1
    }
}

fn sinc_cutoff(n: f64, cutoff: f64) -> f64 {
    if n == 0_f64 {
        cutoff
    } else {
        (n * cutoff * f64::consts::PI).sin() / (n * f64::consts::PI)
    }
}
