use std::f64;

use crate::window::window_function::Window;

pub struct FirFilter {
    pub taps: Vec<f32>,
}

impl FirFilter {
    pub fn new(taps: u32, normalised_cutoff: f64, window: Window<f64>) -> FirFilter {
        let centre = ((taps - 1) as f64 / 2_f64);

        let ideal_impulse_response = |n: f64| sinc_cutoff(n - centre, normalised_cutoff);

        //get the window
        let mut window = window.samples;

        //multiply each tap by the ideal impulse response
        window
            .iter_mut()
            .enumerate()
            .for_each(|(n, f)| *f *= ideal_impulse_response(n as f64));

        //normalise the window so it sums to 1 and create the FirFilter
        let sum: f64 = window.iter().sum();
        FirFilter {
            taps: window.iter().map(|n| (n / sum) as f32).collect(),
        }
    }
}

fn sinc_cutoff(n: f64, cutoff: f64) -> f64 {
    if n == 0_f64 {
        cutoff
    } else {
        (n * cutoff * f64::consts::PI).sin() / (n * f64::consts::PI)
    }
}
