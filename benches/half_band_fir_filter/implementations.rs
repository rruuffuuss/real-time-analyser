/* Algorithmically distinct versions of `half_band_into_udmq` and its predecessors
 *
 * output differences (slice/queue/udmq) have all been normalised to VecDeque
 * so the core filtering implementation is benchmarked and the output structure update cost is constant between runs
*/

use std::collections::VecDeque;

pub const fn f32s_simd_max() -> usize {
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

/// Base version
/// Each subsample is calculated by iterating input samples, multiplying each by the tap coefficient, and summing the iterator
/// `.map(|(xn, b)| -> f32 { xn * b }).sum::<f32>()`
/// version present in commit `d2fc89e`: iterator map followed by sum
///
/// This was originally `DecimatingController::decimate`. Its traversal and
/// output has been updated to match the later queue-based functions rather than write into a mutable vector
pub mod map_sum {
    use super::VecDeque;

    #[inline(always)]
    pub fn half_band_into_queue(
        taps: &[f32],
        tap_num: usize,
        new_samples: usize,
        source: &[f32],
        target: &mut VecDeque<f32>,
    ) {
        for end_sample in ((source.len() - new_samples)..source.len()).step_by(2) {
            target.push_back(
                source[end_sample - tap_num..end_sample]
                    .iter()
                    .zip(taps)
                    .map(|(xn, b)| -> f32 { xn * b })
                    .sum::<f32>(),
            );
        }
    }
}

/// fused multiply add version
/// Each subsample is calculated using fold with mul_add in place of map(sample * tap_coef) followed by sum
/// This computes the multiplication & addition in one instruction on architectures with a fused multiply add instruction, rather than multiplying and summing seperately
/// this reduces the number of instructions and means only one floating point round is performed
/// `.fold(0_f32, |acc, (xn, b)| b.mul_add(*xn, acc)),`
///
/// version present in commit `179d592`: fold using mul_add
pub mod mul_add_fold {
    use super::VecDeque;

    #[inline(always)]
    pub fn half_band_into_queue(
        taps: &[f32],
        tap_num: usize,
        new_samples: usize,
        source: &[f32],
        target: &mut VecDeque<f32>,
    ) {
        for end_sample in ((source.len() - new_samples)..source.len()).step_by(2) {
            target.push_back(
                source[end_sample - tap_num..end_sample]
                    .iter()
                    .zip(taps)
                    .fold(0_f32, |acc, (xn, b)| b.mul_add(*xn, acc)),
            );
        }
    }
}

/// SIMD loop vectorisation targetting version using the original map(sample * tap_coef) followed by seperate sum internally
/// Subsamples components are calculated across n accumulators to independently compute and sum every nth tap coefficient and nth input sample
/// once accumulation is complete the accumulators are summed to obtain the final subsample
///
/// n is set to the maximum number of f32s the target architecture's SIMD features support
/// this encourages n subsamples to be processed in parallel via loop vectorisation with multiple multiplications or summations happening in a single instruction
/// this version uses the fused multiply add internally
pub mod simd_accumulators_map_sum_linear_add {
    use super::{VecDeque, f32s_simd_max};

    #[inline(always)]
    pub fn half_band_into_queue(
        taps: &[[f32; f32s_simd_max()]],
        tap_num: usize,
        new_samples: usize,
        source: &[f32],
        target: &mut VecDeque<f32>,
    ) {
        for end_sample in ((source.len() - new_samples)..source.len()).step_by(2) {
            //use multiple accumulators to utilise loop vectorisation
            let mut accumulators = [0_f32; f32s_simd_max()];

            //loop through accumulator sized chunks of source
            source[end_sample - tap_num..end_sample]
                .chunks_exact(f32s_simd_max())
                .zip(taps)
                .map(|(in_samps, taps)| {
                    in_samps
                        .iter()
                        .zip(taps)
                        .map(|(samp, tap)| (samp * tap) as f32)
                })
                .for_each(|sub_samps| {
                    accumulators
                        .iter_mut()
                        .zip(sub_samps)
                        .for_each(|(acc, sub)| *acc += sub)
                });

            //finally combine accumulators
            target.push_back(accumulators.iter().sum());
        }
    }
}

/// SIMD loop vectorisation targetting version using fused multiply add internally
/// see simd_accumulators_map_sum_linear_add for implementation details about SIMD loop vectorisation
/// this version uses the fused multiply add internally
///
/// Commit `24af2a2`: explicit accumulators intended to encourage SIMD
/// vectorisation
pub mod simd_accumulators_mul_add_fold_linear_add {
    use super::{VecDeque, f32s_simd_max};

    #[inline(always)]
    pub fn half_band_into_queue(
        taps: &[[f32; f32s_simd_max()]],
        tap_num: usize,
        new_samples: usize,
        source: &[f32],
        target: &mut VecDeque<f32>,
    ) {
        for end_sample in ((source.len() - new_samples)..source.len()).step_by(2) {
            //use multiple accumulators to utilise loop vectorisation
            let mut accumulators = [0_f32; f32s_simd_max()];

            //loop through accumulator sized chunks of source
            source[end_sample - tap_num..end_sample]
                .chunks_exact(f32s_simd_max())
                .zip(taps)
                .for_each(|(samples, taps)| {
                    //add each sample * tap to the corresponding accumulator independently
                    accumulators
                        .iter_mut()
                        .zip(samples.iter().zip(taps))
                        .for_each(|(acc, (sample, tap))| *acc = sample.mul_add(*tap, *acc))
                });

            //finally combine accumulators
            target.push_back(accumulators.iter().sum());
        }
    }
}

/// SIMD loop vectorisation targeting using fused multiply add with a reduction tree to sum accumulators
/// the reduction tree breaks the dependency chain of adding each accumulator linearly
///
/// version present in commit `63e63d6`: reduction-tree accumulator sum
pub mod simd_accumulators_mul_add_fold_reduction_tree {
    use super::{VecDeque, f32s_simd_max};

    #[inline(always)]
    pub fn half_band_into_queue(
        taps: &[[f32; f32s_simd_max()]],
        tap_num: usize,
        new_samples: usize,
        source: &[f32],
        target: &mut VecDeque<f32>,
    ) {
        for end_sample in ((source.len() - new_samples)..source.len()).step_by(2) {
            //use multiple accumulators to utilise loop vectorisation
            let mut accumulators = [0_f32; f32s_simd_max()];

            //loop through accumulator sized chunks of source
            source[end_sample - tap_num..end_sample]
                .chunks_exact(f32s_simd_max())
                .zip(taps)
                .for_each(|(samples, taps)| {
                    //add each sample * tap to the corresponding accumulator independently
                    accumulators
                        .iter_mut()
                        .zip(samples.iter().zip(taps))
                        .for_each(|(acc, (sample, tap))| *acc = sample.mul_add(*tap, *acc))
                });

            //finally combine accumulators
            target.push_back(sum_reduction_tree(accumulators));
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
}

/// SIMD loop vectorisation targeting using map sum with a reduction tree to sum accumulators
/// the reduction tree breaks the dependency chain of adding each accumulator linearly
pub mod simd_accumulators_map_sum_reduction_tree {
    use super::{VecDeque, f32s_simd_max};

    #[inline(always)]
    pub fn half_band_into_queue(
        taps: &[[f32; f32s_simd_max()]],
        tap_num: usize,
        new_samples: usize,
        source: &[f32],
        target: &mut VecDeque<f32>,
    ) {
        for end_sample in ((source.len() - new_samples)..source.len()).step_by(2) {
            //use multiple accumulators to utilise loop vectorisation
            let mut accumulators = [0_f32; f32s_simd_max()];

            //loop through accumulator sized chunks of source
            source[end_sample - tap_num..end_sample]
                .chunks_exact(f32s_simd_max())
                .zip(taps)
                .map(|(in_samps, taps)| {
                    in_samps
                        .iter()
                        .zip(taps)
                        .map(|(samp, tap)| (samp * tap) as f32)
                })
                .for_each(|sub_samps| {
                    accumulators
                        .iter_mut()
                        .zip(sub_samps)
                        .for_each(|(acc, sub)| *acc += sub)
                });
            //finally combine accumulators
            target.push_back(sum_reduction_tree(accumulators));
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
}

pub mod ilp_accumulators_simd_accumulators_mul_add_fold_linear_add {
    use super::{VecDeque, f32s_simd_max};

    const ILP_SUBSAMPLE_NUM: usize = 4;

    #[inline(always)]
    pub fn half_band_into_queue(
        taps: &[[f32; f32s_simd_max()]],
        tap_num: usize,
        new_samples: usize,
        source: &[f32],
        target: &mut VecDeque<f32>,
    ) {
        // use multiple subsamples at a time to encourage ILP
        let mut subsamples = [0_f32; ILP_SUBSAMPLE_NUM];

        let ilp_subsample_num = usize::min(ILP_SUBSAMPLE_NUM, source.len() / 2);

        for i in ((source.len() - new_samples)..source.len()).step_by(2 * ilp_subsample_num) {
            // one SIMD accumulator for each independent subsample
            let mut accumulators = [[0_f32; f32s_simd_max()]; ILP_SUBSAMPLE_NUM];

            // iterate over SIMD-sized tap chunks first
            for (tap_chunk_idx, taps) in taps.iter().enumerate() {
                let tap_offset = tap_chunk_idx * f32s_simd_max();

                // update each independent subsample for this tap chunk
                for subsample_idx in 0..ilp_subsample_num {
                    let end_sample = i + subsample_idx * 2;

                    let sample_start = end_sample - tap_num + tap_offset;

                    let samples = &source[sample_start..sample_start + f32s_simd_max()];

                    // SIMD-friendly inner loop
                    accumulators[subsample_idx]
                        .iter_mut()
                        .zip(samples.iter().zip(taps))
                        .for_each(|(acc, (sample, tap))| {
                            *acc = sample.mul_add(*tap, *acc);
                        });
                }
            }

            // reduce each independent SIMD accumulator to one output sample
            for (subsample, accumulator) in subsamples.iter_mut().zip(&accumulators) {
                *subsample = accumulator.iter().sum();
            }

            // append all independently calculated samples together
            target.extend(&subsamples);
        }
    }
}
