use crate::captor;
use crate::display::display::Display;
use crate::display::display_settings::DisplaySettings;

use super::control_core::ControlCore;

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

    pub fn run(&mut self) {
        let (fresh_tx, fresh_rx): (Sender<VecDeque<f32>>, Receiver<VecDeque<f32>>) =
            mpsc::channel();
        let (stale_tx, stale_rx): (Sender<VecDeque<f32>>, Receiver<VecDeque<f32>>) =
            mpsc::channel();

        let transform_buffer = VecDeque::from(vec![0.0_f32; self.control_core.sample_window]);

        stale_tx.send(transform_buffer).unwrap();

        let framerate = self.control_core.target_framerate;

        thread::spawn(move || {
            captor::run(
                self.control_core.transform_size / framerate,
                fresh_tx,
                stale_rx,
            )
        });

        for recieved in fresh_rx {
            // recieve fresh audio samples & capture thread builds capture buffer whilst we FFT
            let mut spectrum_data = self
                .control_core
                .transformer
                .transform_split(recieved.as_slices());

            // we return empty buffer which can be filled whilst we normalise & draw
            stale_tx.send(recieved).unwrap();

            self.control_core.normaliser.normalise(&mut spectrum_data);
            self.control_core
                .display
                .display(&spectrum_data[..self.control_core.display.ideal_bar_count()]);
        }
    }
}
