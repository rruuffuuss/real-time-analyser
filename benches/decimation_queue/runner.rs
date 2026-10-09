use super::implementations::{
    f32s_simd_max, fir_filter_double_mapped_queue_always_wrap,
    fir_filter_double_mapped_queue_intelligent_wrap, fir_filter_vecdeque_contiguous,
    fir_filter_vecdeque_split,
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

fn bench_vecdeque_fir_filter(
    bencher: divan::Bencher,
    bench_data: BenchData,
    mut filter_stage: impl FnMut(&[SimdTapChunk], usize, usize, &mut VecDeque<f32>, &mut VecDeque<f32>),
) {
    let setup = setup_bench(bench_data);
    let mut queues = initialized_vecdeques(&setup);

    bencher.bench_local(|| {
        for (stage, &new_sample_count) in setup.new_sample_counts.iter().enumerate() {
            let (sources, targets) = queues.split_at_mut(stage + 1);
            let source = sources.last_mut().unwrap();
            let target = targets.first_mut().unwrap();
            target.drain(0..new_sample_count / 2);

            filter_stage(
                divan::black_box(&setup.taps),
                divan::black_box(setup.padded_tap_count),
                divan::black_box(new_sample_count),
                divan::black_box(source),
                divan::black_box(target),
            );
        }
    });
}

#[divan::bench(sample_count = 5, sample_size = 50, args = BENCHMARK_DATA)]
fn vecdeque_fir_filter_contiguous(bencher: divan::Bencher, bench_data: BenchData) {
    bench_vecdeque_fir_filter(bencher, bench_data, vecdequeue_make_contiguous_inner());
}

fn vecdequeue_make_contiguous_inner()
-> fn(&[SimdTapChunk], usize, usize, &mut VecDeque<f32>, &mut VecDeque<f32>) {
    |taps, tap_count, new_sample_count, source, target| {
        let source = source.make_contiguous();

        fir_filter_vecdeque_contiguous::half_band_into_queue(
            taps,
            tap_count,
            new_sample_count,
            source,
            target,
        );
    }
}

#[divan::bench(sample_count = 5, sample_size = 50, args = BENCHMARK_DATA)]
fn vecdeque_fir_filter_wrap_split(bencher: divan::Bencher, bench_data: BenchData) {
    bench_vecdeque_fir_filter(bencher, bench_data, vecdequeue_wrap_split_inner());
}

fn vecdequeue_wrap_split_inner()
-> fn(&[SimdTapChunk], usize, usize, &mut VecDeque<f32>, &mut VecDeque<f32>) {
    |taps, tap_count, new_sample_count, source, target| {
        let (source1, source2) = source.as_slices();

        fir_filter_vecdeque_split::half_band_into_queue(
            taps,
            tap_count,
            new_sample_count,
            source1,
            source2,
            target,
        );
    }
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
    bench_double_mapped_queue(bencher, bench_data, double_mapped_intelligent_wrap_inner());
}

fn double_mapped_intelligent_wrap_inner()
-> fn(&[SimdTapChunk], usize, usize, &[f32], &mut UncheckedDoubleMappedQueue<f32>) {
    |taps, tap_count, new_sample_count, source, target| {
        fir_filter_double_mapped_queue_intelligent_wrap::half_band_into_queue(
            taps,
            tap_count,
            new_sample_count,
            source,
            target,
        );

        target.wrap();
    }
}

#[divan::bench(sample_count = 5, sample_size = 50, args = BENCHMARK_DATA)]
fn _double_mapped_queue_fir_filter_always_wrap(bencher: divan::Bencher, bench_data: BenchData) {
    bench_double_mapped_queue(bencher, bench_data, double_mapped_always_wrap_inner());
}

fn double_mapped_always_wrap_inner()
-> fn(&[SimdTapChunk], usize, usize, &[f32], &mut UncheckedDoubleMappedQueue<f32>) {
    fir_filter_double_mapped_queue_always_wrap::half_band_into_queue
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_ITERATIONS: usize = 10;

    #[test]
    fn filter_implementations_identical() {
        let benchdata = BenchData::new(131, 1024);

        let benchsetup = setup_bench(benchdata);

        let start_vecdequeues = initialized_vecdeques(&benchsetup);
        let start_doublemappedqueues = initialized_double_mapped_queues(&benchsetup);

        let mut vecdequeue_make_contig_queues = start_vecdequeues.clone();
        let mut vecdequeue_split_wrap = start_vecdequeues.clone();
        let vecdequeuequeues = [
            (
                &mut vecdequeue_make_contig_queues,
                vecdequeue_make_contiguous_inner(),
            ),
            (&mut vecdequeue_split_wrap, vecdequeue_wrap_split_inner()),
        ];

        let mut doublemapped_always_wrap = start_doublemappedqueues.clone();
        let mut doublemapped_smart_wrap = start_doublemappedqueues.clone();
        let mut doublemappedqueuequeues = [
            (
                &mut doublemapped_always_wrap,
                double_mapped_always_wrap_inner(),
            ),
            (
                &mut doublemapped_smart_wrap,
                double_mapped_intelligent_wrap_inner(),
            ),
        ];

        //repeatedly apply decimation stages to vecdequeue buffers
        for (queuequeue, filter) in vecdequeuequeues {
            for _ in 0..TEST_ITERATIONS {
                for (stage, &new_samples) in benchsetup.new_sample_counts.iter().enumerate() {
                    let (sources, targets) = queuequeue.split_at_mut(stage + 1);
                    let source = sources.last_mut().unwrap();
                    let target = targets.first_mut().unwrap();
                    target.drain(0..new_samples / 2);

                    filter(
                        divan::black_box(&benchsetup.taps),
                        divan::black_box(benchsetup.padded_tap_count),
                        divan::black_box(new_samples),
                        divan::black_box(source),
                        divan::black_box(target),
                    );
                }
            }
        }

        //repeatedly apply decimation stages to double mapped queue buffers
        for (queuequeue, filter) in doublemappedqueuequeues {
            for _ in 0..TEST_ITERATIONS {
                for (stage, &new_samples) in benchsetup.new_sample_counts.iter().enumerate() {
                    let (sources, targets) = queuequeue.split_at_mut(stage + 1);
                    let source = sources.last_mut().unwrap();
                    let target = targets.first_mut().unwrap();
                    target.drain(new_samples / 2);

                    filter(
                        divan::black_box(&benchsetup.taps),
                        divan::black_box(benchsetup.padded_tap_count),
                        divan::black_box(new_samples),
                        divan::black_box(source.as_slice()),
                        divan::black_box(target),
                    );
                }
            }
        }

        let get_vecdequeuequeue = |queue: &Vec<VecDeque<f32>>, i: usize, j: usize| {
            queue.get(i).unwrap().get(j).unwrap().clone()
        };

        let get_doublemappedqueue =
            |queue: &Vec<UncheckedDoubleMappedQueue<f32>>, i: usize, j: usize| {
                queue.get(i).unwrap().get(j).unwrap().clone()
            };

        let mut difference_count: usize = 0;

        for i in 0..start_vecdequeues.len() {
            for j in 0..start_vecdequeues.get(i).unwrap().len() {
                //check all queue systems have the same final values
                assert_eq!(
                    get_vecdequeuequeue(&vecdequeue_make_contig_queues, i, j),
                    get_vecdequeuequeue(&vecdequeue_split_wrap, i, j),
                );

                assert_eq!(
                    get_vecdequeuequeue(&vecdequeue_make_contig_queues, i, j),
                    get_doublemappedqueue(&doublemapped_always_wrap, i, j),
                );

                assert_eq!(
                    get_doublemappedqueue(&doublemapped_smart_wrap, i, j),
                    get_doublemappedqueue(&doublemapped_always_wrap, i, j),
                );

                //check queues have been updated
                if (get_vecdequeuequeue(&start_vecdequeues, i, j)
                    != get_vecdequeuequeue(&vecdequeue_split_wrap, i, j))
                {
                    difference_count += 1;
                }
            }
        }

        assert_ne!(difference_count, 0_usize);
        println!("{}", difference_count);
    }
}
