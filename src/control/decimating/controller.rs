use super::super::core::core::ControlCore;
use super::fir_filter::FirFilter;

use crate::captor;
use crate::control::Controller;

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

    //#[inline(always)]
    fn decimate(&self, tap_num: usize, source: &[f32], target: &mut [f32]) {
        target.iter_mut().enumerate().for_each(|(i, yn)| {
            *yn = source[2 * i..2 * i + tap_num]
                .iter()
                .zip(&self.filter.taps)
                .map(|(xn, b)| -> f32 { xn * b })
                .sum::<f32>();
        });
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
        let tap_num = self.filter.taps.len();

        // each 'chunk' is composed of 2 times the number of captured samples + an extra tap_num to hold samples from the prior iteration which weren't fully used in the filter
        // in hindsight the whole decimation thing could have been more easily implemented with a ring buffer at each decimation level but this feels faster
        let decimation_size = transform_size / 2; //this is also the amount of samples we capture in one cycle
        let chunk_size = transform_size + tap_num;
        let spectrum_chunk_size =
            self.control_core.display.ideal_bar_count() / (self.displayed_decimations + 1);
        let mut sample_buffer = vec![0_f32; chunk_size * decimations];
        let mut spectrum_data = vec![0_f32; self.control_core.display.ideal_bar_count()];
        let transfer_buffer = vec![0_f32; self.control_core.transform_size];

        let mut cycle = 1;
        let mut cur_chunk;

        thread::spawn(move || captor::run(transform_size as usize, fresh_tx, stale_rx));
        stale_tx.send(transfer_buffer).unwrap();

        let cycles_per_frame = ((self.control_core.sample_rate
            / (transform_size as u32 * self.control_core.target_framerate as u32))
            as usize)
            .max(1);

        let (mut target, mut source): (&mut [f32], &mut [f32]);

        for mut recieved in fresh_rx {
            //may need to update the "partially used tap sample" logic if FFT mutates input chunks

            for recieved_chunk in recieved.chunks_exact(transform_size) {
                //position incoming samples correctly in buffer
                sample_buffer[tap_num..transform_size + tap_num].copy_from_slice(recieved_chunk);

                cur_chunk = 0;

                while cur_chunk < decimations - 1 {
                    // move (copy) the existing samples in the the back half the next chunk to the front half
                    // front ..... back
                    //   f^^^^l<-f^^^^l
                    (target, source) = sample_buffer
                        .split_at_mut((cur_chunk + 1) * chunk_size + tap_num + decimation_size);
                    target[(cur_chunk + 1) * chunk_size + tap_num..]
                        .copy_from_slice(&source[..decimation_size]);

                    // decimate into the back half of the next chunk
                    (source, target) = sample_buffer.split_at_mut((cur_chunk + 1) * chunk_size);
                    self.decimate(
                        tap_num,
                        // decimate from the current buffer
                        &source[(cur_chunk * chunk_size)..],
                        // into the second half of the next buffer
                        &mut target[tap_num + decimation_size..chunk_size],
                    );

                    // copy 'partially decimated' samples from the end front half of the chunk to the start
                    // since each sample is decimated twice in this algorithm (so that each stage is updated every cycle),
                    // these samples are located at the end of the front half, but were partially used when they were in the back half
                    (target, source) = sample_buffer.split_at_mut(chunk_size * cur_chunk + tap_num);
                    target[chunk_size * cur_chunk..]
                        .copy_from_slice(&source[chunk_size - tap_num..chunk_size]);

                    cur_chunk += 1;
                }
            }

            recieved.clear();
            stale_tx.send(recieved).unwrap();

            // transform each chunk
            // will need to update merger for this to result in a normal spectrum
            // only iterate chunks that have been fully updated with freshly decimated samples
            spectrum_data
                .chunks_exact_mut(spectrum_chunk_size)
                .zip(
                    sample_buffer[hidden_decimations..]
                        .chunks_exact_mut(chunk_size)
                        .rev(),
                )
                .for_each(|(s, f)| {
                    self.control_core
                        .transformer
                        .transform(&mut f[tap_num..], s)
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
