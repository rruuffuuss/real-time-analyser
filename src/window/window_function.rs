use std::f64;

use num_traits::Float;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowFunction {
    Rectangular,
    Triangular,
    Hann,
    Blackman,
}

pub struct Window<F: Float> {
    pub samples: Vec<F>,
}

impl WindowFunction {
    pub fn build_window<F>(self, samples: u32) -> Window<F>
    where
        F: Float,
    {
        type WindowFn = fn(f64, f64) -> f64;

        let window: WindowFn = match self {
            WindowFunction::Rectangular => |_, _| 1_f64,
            WindowFunction::Triangular => |n, samples| {
                if samples <= 1.0 {
                    1.0
                } else {
                    1.0 - (2.0 * n - (samples - 1.0)).abs() / (samples - 1.0)
                }
            },
            WindowFunction::Hann => |n, samples| {
                0.5_f64 - (0.5_f64 * f64::cos(2_f64 * f64::consts::PI * n / (samples - 1_f64)))
            },
            WindowFunction::Blackman => |n, samples| {
                (7938_f64 / 18608_f64)
                    - (9240_f64 / 18608_f64)
                        * f64::cos((2_f64 * f64::consts::PI * n) / (samples - 1_f64))
                    + (1430_f64 / 18608_f64)
                        * f64::cos((4_f64 * f64::consts::PI * n) / (samples - 1_f64))
            },
        };

        Window {
            samples: (0..samples)
                .map(|n| F::from(window(n as f64, samples as f64)).unwrap())
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_samples_approx_eq(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());

        actual.iter().zip(expected).for_each(|(actual, expected)| {
            assert!(
                (actual - expected).abs() < 1e-12,
                "expected {expected}, got {actual}"
            );
        });
    }

    #[test]
    fn rectangular_window_has_constant_samples() {
        let window = WindowFunction::Rectangular.build_window::<f64>(5);

        assert_eq!(window.samples, vec![1.0; 5]);
    }

    #[test]
    fn triangular_window_tapers_to_zero() {
        let window = WindowFunction::Triangular.build_window::<f64>(5);

        assert_eq!(window.samples, vec![0.0, 0.5, 1.0, 0.5, 0.0]);
        assert_eq!(
            WindowFunction::Triangular.build_window::<f64>(1).samples,
            vec![1.0]
        );
    }

    #[test]
    fn hann_window_has_expected_samples() {
        let window = WindowFunction::Hann.build_window::<f64>(5);

        assert_samples_approx_eq(&window.samples, &[0.0, 0.5, 1.0, 0.5, 0.0]);
    }

    #[test]
    fn blackman_window_has_expected_samples() {
        let window = WindowFunction::Blackman.build_window::<f64>(5);

        assert_samples_approx_eq(
            &window.samples,
            &[
                0.006878761822872,
                0.349742046431642,
                1.0,
                0.349742046431642,
                0.006878761822872,
            ],
        );
    }
}
