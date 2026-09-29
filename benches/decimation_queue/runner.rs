use super::implementations::{
    f32s_simd_max, fir_filter_double_mapped_queue_always_wrap,
    fir_filter_double_mapped_queue_intelligent_wrap, fir_filter_vecdeque,
};
use super::unchecked_double_mapped_queue::UncheckedDoubleMappedQueue;
use std::collections::VecDeque;

const DECIMATIONS: usize = 7;

type SimdTapChunk = [f32; f32s_simd_max()];

const BENCHMARK_DATA: [BenchData; 2] = [BenchData::new(131, 1024), BenchData::new(51, 512)];

#[derive(Debug, Copy, Clone)]
struct BenchData {
    tap_count: usize,
    max_sample_count: usize,
    decimations: usize,
}

impl BenchData {
    const fn new(tap_count: usize, max_sample_count: usize) -> Self {
        Self {
            tap_count,
            max_sample_count,
            decimations: DECIMATIONS,
        }
    }
}

struct BenchSetup {
    decimations: usize,
    queue_len: usize,
    padded_tap_count: usize,
    new_sample_counts: Vec<usize>,
    taps: Vec<SimdTapChunk>,
}

fn setup_bench(bench_data: BenchData) -> BenchSetup {
    // Round the taps up to a whole SIMD vector.
    let padded_tap_count = bench_data.tap_count
        + (f32s_simd_max() - (bench_data.tap_count % f32s_simd_max())) % f32s_simd_max();
    let taps = vec![[1_f32; f32s_simd_max()]; padded_tap_count / f32s_simd_max()];
    let new_sample_counts = (0..bench_data.decimations - 1)
        .map(|stage| bench_data.max_sample_count >> stage)
        .collect();

    BenchSetup {
        decimations: bench_data.decimations,
        queue_len: bench_data.max_sample_count + padded_tap_count,
        padded_tap_count,
        new_sample_counts,
        taps,
    }
}

fn initialized_vecdeques(setup: &BenchSetup) -> Vec<VecDeque<f32>> {
    let queue = VecDeque::from(vec![1_f32; setup.queue_len]);
    vec![queue; setup.decimations]
}

fn initialized_double_mapped_queues(setup: &BenchSetup) -> Vec<UncheckedDoubleMappedQueue<f32>> {
    let samples = vec![1_f32; setup.queue_len];
    let queue = UncheckedDoubleMappedQueue::from(&samples);
    vec![queue; setup.decimations]
}

#[divan::bench(sample_count = 5, sample_size = 50, args = BENCHMARK_DATA)]
fn vecdeque_fir_filter(bencher: divan::Bencher, bench_data: BenchData) {
    let setup = setup_bench(bench_data);
    let mut queues = initialized_vecdeques(&setup);

    bencher.bench_local(|| {
        for (stage, &new_sample_count) in setup.new_sample_counts.iter().enumerate() {
            let (sources, targets) = queues.split_at_mut(stage + 1);
            let source = sources.last_mut().unwrap();
            let target = targets.first_mut().unwrap();
            target.drain(0..new_sample_count / 2);

            let source = source.make_contiguous();

            fir_filter_vecdeque::half_band_into_queue(
                divan::black_box(&setup.taps),
                divan::black_box(setup.padded_tap_count),
                divan::black_box(new_sample_count),
                divan::black_box(source),
                divan::black_box(target),
            );
        }
    });
}

fn bench_double_mapped_queue(
    bencher: divan::Bencher,
    bench_data: BenchData,
    mut filter_stage: impl FnMut(
        &[SimdTapChunk],
        usize,
        usize,
        &[f32],
        &mut UncheckedDoubleMappedQueue<f32>,
    ),
) {
    let setup = setup_bench(bench_data);
    let mut queues = initialized_double_mapped_queues(&setup);

    bencher.bench_local(|| {
        for (stage, &new_sample_count) in setup.new_sample_counts.iter().enumerate() {
            let (sources, targets) = queues.split_at_mut(stage + 1);
            let source = sources.last_mut().unwrap();
            let target = targets.first_mut().unwrap();
            target.drain(new_sample_count / 2);

            filter_stage(
                divan::black_box(&setup.taps),
                divan::black_box(setup.padded_tap_count),
                divan::black_box(new_sample_count),
                divan::black_box(source.as_slice()),
                divan::black_box(target),
            );
        }
    });
}

#[divan::bench(sample_count = 5, sample_size = 50, args = BENCHMARK_DATA)]
fn _double_mapped_queue_fir_filter_intelligent_wrap(
    bencher: divan::Bencher,
    bench_data: BenchData,
) {
    bench_double_mapped_queue(
        bencher,
        bench_data,
        |taps, tap_count, new_sample_count, source, target| {
            fir_filter_double_mapped_queue_intelligent_wrap::half_band_into_queue(
                taps,
                tap_count,
                new_sample_count,
                source,
                target,
            );

            target.wrap();
        },
    );
}

#[divan::bench(sample_count = 5, sample_size = 50, args = BENCHMARK_DATA)]
fn _double_mapped_queue_fir_filter_always_wrap(bencher: divan::Bencher, bench_data: BenchData) {
    bench_double_mapped_queue(
        bencher,
        bench_data,
        fir_filter_double_mapped_queue_always_wrap::half_band_into_queue,
    );
}
