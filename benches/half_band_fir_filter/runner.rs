use super::implementations::{
    f32s_simd_max, ilp_accumulators_simd_accumulators_mul_add_fold_linear_add, map_sum,
    mul_add_fold, simd_accumulators_map_sum_linear_add, simd_accumulators_map_sum_reduction_tree,
    simd_accumulators_mul_add_fold_linear_add, simd_accumulators_mul_add_fold_reduction_tree,
};

const BENCHMARK_DATA: [BenchData; 2] = [
    BenchData {
        tap_count: 131,
        max_sample_count: 1024,
        decimations: 7,
    },
    BenchData {
        tap_count: 51,
        max_sample_count: 512,
        decimations: 7,
    },
];

macro_rules! run_bench {
    ($name:ident, $func:path, false) => {
        #[divan::bench(sample_count = 50, sample_size = 5000, args = BENCHMARK_DATA)]
        fn $name(bencher: divan::Bencher, benchmark_data: BenchData) {
            run_bench(bencher, $func, benchmark_data);
        }
    };

    ($name:ident, $func:path, true) => {
        #[divan::bench(sample_count = 50, sample_size = 5000, args = BENCHMARK_DATA)]
        fn $name(bencher: divan::Bencher, benchmark_data: BenchData) {
            run_bench_simd(bencher, $func, benchmark_data);
        }
    };
}

use std::collections::VecDeque;

use super::implementations;

#[derive(Debug, Copy, Clone)]
struct BenchData {
    tap_count: usize,
    max_sample_count: usize,
    decimations: usize,
}

fn bench_setup(bench_data: &BenchData) -> (Vec<VecDeque<f32>>, Vec<Vec<f32>>, Vec<usize>) {
    let target_queues: Vec<VecDeque<f32>> =
        vec![
            VecDeque::with_capacity(bench_data.max_sample_count + bench_data.tap_count);
            bench_data.decimations
        ];

    // the input samples for the benchmark will be stored in vectors so they can easily be accessed contiguously
    // we want to benchmark the fir filter logic without measuring VecDeque::make_contiguous() or including a custome double mapped queue implementation
    let source_samples = vec![
        vec![1_f32; bench_data.max_sample_count + bench_data.tap_count];
        bench_data.decimations
    ];

    let new_samples_count: Vec<usize> = (0..bench_data.decimations - 1)
        .map(|i| (bench_data.max_sample_count >> i) as usize)
        .collect();

    (target_queues, source_samples, new_samples_count)
}

fn run_bench<F>(bencher: divan::Bencher, f: F, bench_data: BenchData)
where
    F: Fn(&[f32], usize, usize, &[f32], &mut VecDeque<f32>),
{
    let decimations = bench_data.decimations.clone();
    let taps = vec![1_f32; bench_data.tap_count];

    let (mut target_queues, source_samples, new_samples_count) = bench_setup(&bench_data);

    bencher.bench_local(|| {
        for i in 0..(decimations - 1) {
            f(
                divan::black_box(&taps),
                divan::black_box(taps.len()),
                divan::black_box(new_samples_count[i]),
                divan::black_box(&source_samples[i]),
                divan::black_box(&mut target_queues[i]),
            );

            target_queues[i].clear()
        }
    });
}

fn run_bench_simd<F>(bencher: divan::Bencher, f: F, mut bench_data: BenchData)
where
    F: Fn(&[[f32; f32s_simd_max()]], usize, usize, &[f32], &mut VecDeque<f32>),
{
    //round up for simd aligment
    let padded_tap_count = bench_data.tap_count
        + (f32s_simd_max() - (bench_data.tap_count % f32s_simd_max())) % f32s_simd_max();

    bench_data.tap_count = padded_tap_count;

    let taps = vec![[1_f32; f32s_simd_max()]; padded_tap_count / f32s_simd_max()];

    let (mut target_queues, source_samples, new_samples_count) = bench_setup(&bench_data);

    bencher.bench_local(|| {
        for i in 0..(bench_data.decimations - 1) {
            f(
                divan::black_box(&taps),
                divan::black_box(padded_tap_count),
                divan::black_box(new_samples_count[i]),
                divan::black_box(&source_samples[i]),
                divan::black_box(&mut target_queues[i]),
            );

            target_queues[i].clear()
        }
    });
}

run_bench!(v0_map_sum, map_sum::half_band_into_queue, false);
run_bench!(v1_mul_add_fold, mul_add_fold::half_band_into_queue, false);
run_bench!(
    v2_simd_accumulators__map_sum__linear_add,
    simd_accumulators_map_sum_linear_add::half_band_into_queue,
    true
);
run_bench!(
    v3_simd_accumulators__map_sum__reduction_tree,
    simd_accumulators_map_sum_reduction_tree::half_band_into_queue,
    true
);
run_bench!(
    v4_simd_accumulators__mul_add_fold__linear_add,
    simd_accumulators_mul_add_fold_linear_add::half_band_into_queue,
    true
);
run_bench!(
    v5_simd_accumulators__mul_add_fold__reduction_tree,
    simd_accumulators_mul_add_fold_reduction_tree::half_band_into_queue,
    true
);
run_bench!(
    v6_ilp_accumulators__simd_accumulators__mul_add_fold__linear_add,
    ilp_accumulators_simd_accumulators_mul_add_fold_linear_add::half_band_into_queue,
    true
);
