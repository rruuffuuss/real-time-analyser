use super::super::core::core::ControlCore;
use super::fir_filter::FirFilter;

use crate::captor;
use crate::control::Controller;
use crate::control::unchecked_double_mapped_queue::UncheckedDoubleMappedQueue;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use std::thread::{self};

pub struct DecimatingController {
    control_core: ControlCore,
    filter: FirFilter,

    decimations: usize,

    ///decimations above this are used to lower observable frequency range, but won't be transformed & displayed themselves
    displayed_decimations: usize,
}

impl DecimatingController {
    pub fn new(
        control_core: ControlCore,
        filter: FirFilter,
        decimations: usize,
        displayed_decimations: usize,
    ) -> Self {
        Self {
            control_core,
            filter,

            decimations,
            displayed_decimations,
        }
    }
}

impl Controller for DecimatingController {
    /// The way the main run loop works is slightly complicated.
    /// Audio samples are moved through stages where they are recursively low pass filtered and downsampled by factor 2 (the combined process hereon refered to as decimation) before being transformed.
    /// Naturally, it takes 2n samples from one stage to populate n samples in the following stage.
    /// Further complication is introduced by the fact that FIR filter's operate across multiple samples, meaning samples toward the end of a chunk are required across 2 distinct decimations
    /// The benefit of all this is that high frequencies can be observed across a short time window at a high sample rate, whilst low frequencies are given a longer time window at a lower sample rate,
    /// allowing an analyser that is both responsive and wide ranged whilst using smaller transform sizes
    fn run(&mut self) {
        let (fresh_tx, fresh_rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = mpsc::channel();
        let (stale_tx, stale_rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = mpsc::channel();

        let decimations = self.decimations;
        let hidden_decimations = decimations - self.displayed_decimations;
        let transform_size = self.control_core.transform_size;
        let tap_num = self.filter.tap_num;

        let chunk_size = transform_size + tap_num;
        let spectrum_chunk_size =
            self.control_core.display.ideal_bar_count() / (self.displayed_decimations);
        if spectrum_chunk_size == 0 {
            panic!(
                "Your current configuration would result in less than 1 bar for each decimation.\nReduce the number of decimations or increase the number of bars"
            )
        }

        let mut sample_buffer =
            vec![UncheckedDoubleMappedQueue::from(&vec![0_f32; chunk_size]); decimations];

        //the number of new samples added to each buffer each cycle
        let new_samples: Vec<usize> = (0..decimations - 1)
            .map(|n| (transform_size >> n) as usize)
            .collect();

        let mut spectrum_data = vec![0_f32; self.control_core.display.ideal_bar_count()];
        let transfer_buffer = vec![0_f32; self.control_core.transform_size];

        let mut fft_buffer = vec![0_f32; self.control_core.transform_size];

        let mut cycle = 1;

        thread::spawn(move || captor::run(transform_size as usize, fresh_tx, stale_rx));
        stale_tx.send(transfer_buffer).unwrap();

        let cycles_per_frame = ((self.control_core.sample_rate
            / (transform_size as u32 * self.control_core.target_framerate as u32))
            as usize)
            .max(1);

        for mut recieved in fresh_rx {
            for recieved_chunk in recieved.chunks_exact(transform_size) {
                //position incoming samples correctly in buffer
                sample_buffer[0].drain(transform_size);
                sample_buffer[0].extend(recieved_chunk);

                //windows would be cleaner but can't be mut (makes sense)
                for (i, new_size) in new_samples.iter().enumerate() {
                    let (source_slice, target_slice) = sample_buffer.split_at_mut(i + 1);
                    let source = source_slice.last_mut().unwrap();
                    let target = target_slice.first_mut().unwrap();

                    target.drain(new_size / 2);

                    self.filter
                        .half_band_into_udmq(*new_size, source.as_slice(), target);
                }
            }

            recieved.clear();
            stale_tx.send(recieved).unwrap();

            // transform each chunk
            // will need to update merger for this to result in a normal spectrum
            // only iterate chunks that have been fully updated with freshly decimated samples
            spectrum_data
                .chunks_exact_mut(spectrum_chunk_size)
                .zip(sample_buffer[hidden_decimations..].iter().rev())
                .for_each(|(spectrum_chunk, sample_chunk)| {
                    //apply window function coefficients whilst copying samples into the the fft buffer
                    fft_buffer
                        .iter_mut()
                        .zip(
                            sample_chunk.as_slice()[tap_num..]
                                .iter()
                                .zip(self.control_core.window.samples.iter()),
                        )
                        .for_each(|(buf, (sample, window_coef))| *buf = sample * window_coef);

                    //fft the sample buffer into the outpute spectrum chunk
                    self.control_core
                        .transformer
                        .transform(&mut fft_buffer, spectrum_chunk)
                });

            if cycle % cycles_per_frame == 0 {
                self.control_core.normaliser.normalise(&mut spectrum_data);
                self.control_core
                    .display
                    .display(&spectrum_data[..self.control_core.display.ideal_bar_count()]);
            }

            cycle += 1;
        }
    }
}

/*
/// splits the VecDeque sample buffer into an iterator of chunks equal to the transform size
/// since samplebuffer is created with a capacity that is an exact multiple of the capture bytes, the buffer split must occur between 2 chunks
/// if this property is not true, and a chunk is stored across the split in the circular buffer, this will not work

fn chunk_queue<'a>(&self, queue: &'a mut VecDeque<f32>) -> impl Iterator<Item = &'a mut [f32]> {
    let (q1, q2) = queue.as_mut_slices();
    q1.chunks_exact_mut(self.decimations)
        .chain(q2.chunks_exact_mut(self.decimations))
        .rev()
}
*/
