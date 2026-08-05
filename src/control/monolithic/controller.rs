use crate::captor;
use crate::control::Controller;

use crate::control::core::core::ControlCore;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use std::collections::VecDeque;
use std::thread::{self};
use std::usize;

pub struct MonolithicController {
    control_core: ControlCore,
}

impl MonolithicController {
    pub fn new(control_core: ControlCore) -> Self {
        Self { control_core }
    }
}

impl Controller for MonolithicController {
    fn run(&mut self) {
        let (fresh_tx, fresh_rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = mpsc::channel();
        let (stale_tx, stale_rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = mpsc::channel();

        let samples_per_frame =
            (self.control_core.sample_rate / self.control_core.target_framerate as u32) as usize;

        let mut transform_buffer = VecDeque::from(vec![0_f32; (self.control_core.transform_size)]);
        transform_buffer.reserve(self.control_core.transform_size * 2);

        let mut transform_temp = vec![0_f32; self.control_core.transform_size];

        let mut spectrum_data = vec![0_f32; self.control_core.display.ideal_bar_count()];

        let capture_buffer = vec![0_f32; samples_per_frame];

        stale_tx.send(capture_buffer).unwrap();
        thread::spawn(move || captor::run(samples_per_frame, fresh_tx, stale_rx));

        let sample_num = self.control_core.transform_size.min(samples_per_frame);

        for mut recieved in fresh_rx {
            if recieved.len() == 0 {
                stale_tx.send(recieved).unwrap();
                continue;
            }

            // remove old samples from the transform buffer and drain the capture buffer into the transform buffer
            transform_buffer.drain(..sample_num);
            transform_buffer.extend(recieved.drain(..sample_num));
            recieved.clear();
            // we return empty buffer which can be filled whilst we normalise & draw
            stale_tx.send(recieved).unwrap();

            // recieve fresh audio samples & capture thread builds capture buffer whilst we FFT

            //copy the transform buffer contiguously into transform temp
            //same amount of copying as make_contiguous but avoids buffer damage from fft treating input as scratch
            let (b1, b2) = transform_buffer.as_slices();
            transform_temp
                .iter_mut()
                .zip(b1.iter().chain(b2.iter()))
                .for_each(|(t, b)| *t = *b);
            self.control_core
                .transformer
                .transform(&mut transform_temp, &mut spectrum_data);

            self.control_core.normaliser.normalise(&mut spectrum_data);
            self.control_core
                .display
                .display(&spectrum_data[..self.control_core.display.ideal_bar_count()]);
        }
    }
}
