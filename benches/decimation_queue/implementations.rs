pub mod fir_filter_vecdeque_contiguous {
    use super::f32s_simd_max;
    use std::collections::VecDeque;

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

pub mod fir_filter_vecdeque_split {
    use super::f32s_simd_max;
    use std::collections::VecDeque;

    #[inline(always)]
    pub fn half_band_into_queue(
        taps: &[[f32; f32s_simd_max()]],
        tap_num: usize,
        new_samples: usize,
        source1: &[f32],
        source2: &[f32],
        target: &mut VecDeque<f32>,
    ) {
        let total_len = source1.len() + source2.len();

        for end_sample in ((total_len - new_samples)..total_len).step_by(2) {
            //use multiple accumulators to utilise loop vectorisation
            let mut accumulators = [0_f32; f32s_simd_max()];

            //loop through accumulator sized chunks
            for (sample_idx, simd_taps) in ((end_sample - tap_num)..end_sample).step_by(4).zip(taps)
            {
                //for each accumulator select sample values from the correct source
                for ((accumulator, sample), tap) in (accumulators
                    .iter_mut()
                    .zip((0..).map(|(i)| -> f32 {
                        let ix = i + sample_idx;
                        if ix < source1.len() {
                            source1[ix]
                        } else {
                            source2[ix - source1.len()]
                        }
                    }))
                    .zip(simd_taps))
                {
                    *accumulator = sample.mul_add(*tap, *accumulator)
                }
            }

            //finally combine accumulators
            target.push_back(accumulators.iter().sum());
        }
    }
}

pub mod fir_filter_double_mapped_queue_intelligent_wrap {
    use super::super::unchecked_double_mapped_queue::UncheckedDoubleMappedQueue;

    use super::f32s_simd_max;

    #[inline(always)]
    pub fn half_band_into_queue(
        taps: &[[f32; f32s_simd_max()]],
        tap_num: usize,
        new_samples: usize,
        source: &[f32],
        target: &mut UncheckedDoubleMappedQueue<f32>,
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
            target.push_back_no_wrap(accumulators.iter().sum());
        }
    }
}

pub mod fir_filter_double_mapped_queue_always_wrap {
    use super::super::unchecked_double_mapped_queue::UncheckedDoubleMappedQueue;

    use super::f32s_simd_max;

    #[inline(always)]
    pub fn half_band_into_queue(
        taps: &[[f32; f32s_simd_max()]],
        tap_num: usize,
        new_samples: usize,
        source: &[f32],
        target: &mut UncheckedDoubleMappedQueue<f32>,
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
