use std::f64;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum Window {
    Rectangular,
    Triangular,
    Hann,
    Blackman,
}

pub struct FirFilter {
    pub taps: Vec<f32>,
}

impl FirFilter {
    pub fn new(taps: u32, normalised_cutoff: f64, window_function: Window) -> FirFilter {
        let centre = (((taps - 1) / 2) as u32) as f64;

        let ideal_iir = |n: f64| sinc_cutoff(n - centre, normalised_cutoff);

        type WindowFunction = fn(f64, f64) -> f64;

        let window: WindowFunction = match window_function {
            Window::Rectangular => |_, _| 1_f64,
            Window::Triangular => |n, taps| 1_f64 - (n - taps) / taps,
            Window::Hann => {
                |n, taps| 0.5_f64 - (0.5_f64 * f64::cos(2_f64 * f64::consts::PI * n / taps))
            }
            Window::Blackman => |n, taps| {
                (7938_f64 / 18608_f64)
                    - (9240_f64 / 18608_f64) * f64::cos((2_f64 * f64::consts::PI * n) / taps)
                    + (1430_f64 / 18608_f64) * f64::cos((4_f64 * f64::consts::PI * n) / taps)
            },
        };

        let unnormalised_tap_weights: Vec<f64> = (0..taps)
            .map(|n| window(n as f64, taps as f64) * ideal_iir(n as f64))
            .collect();

        let sum: f64 = unnormalised_tap_weights.iter().sum();

        FirFilter {
            taps: unnormalised_tap_weights
                .iter()
                .map(|n| (n / sum) as f32)
                .collect(),
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
