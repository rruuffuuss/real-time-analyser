use crate::normalise::normaliser::Normaliser;

pub struct FactorNormaliser {
    ///maximum value for the normalisation factor to grow
    max_threshold: f32,
    ///minimum value for the normalisation factor to shrink
    min_threshold: f32,
    ///amount we grow the normalisation factor by
    growth_factor: f32,
    ///amount we shrink the normalisation factor by
    decay_factor: f32,
}

impl FactorNormaliser {
    pub fn new(max_estimate: f32, growth_factor: f32, decay_factor: f32) -> FactorNormaliser {
        FactorNormaliser {
            max_threshold: max_estimate,
            min_threshold: max_estimate,
            growth_factor,
            decay_factor,
        }
    }
}

impl Normaliser for FactorNormaliser {
    fn normalise(&mut self, bars: &mut [f32]) {
        let mut max: f32 = 0.0;

        bars.iter_mut().enumerate().for_each(|(i, b)| {
            max = max.max(*b);
            *b /= self.max_threshold;
        });

        if max > self.max_threshold || max < self.min_threshold {
            self.max_threshold = max * self.growth_factor;
            self.min_threshold = max * self.decay_factor;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::normalise::factor_normaliser::FactorNormaliser;
    use crate::normalise::normaliser::{self, Normaliser};

    fn normalisation_test(
        input: &mut Vec<f32>,
        expected_output: &Vec<f32>,
        starting_threshold: Option<f32>,
        growth_factor: Option<f32>,
        decay_factor: Option<f32>,
        normaliser: Option<FactorNormaliser>,
    ) -> FactorNormaliser {
        let mut t = normaliser.unwrap_or_else(|| {
            FactorNormaliser::new(
                starting_threshold
                    .expect("starting threshold must be provided if a normaliser isn't"),
                growth_factor.expect("growth factor must be provided if a normaliser isn't"),
                decay_factor.expect("decay factor must be provided if a normaliser isn't"),
            )
        });

        t.normalise(input);

        normaliser::tests::assert_eq_normalised_vec(input, expected_output);

        t
    }

    #[test]
    fn test_normalisation_normalises_values() {
        normalisation_test(
            &mut vec![0.0_f32, 5.0, 10.0, 25.0, 49.0],
            &vec![0.0_f32, 0.1, 0.4, 1.5, 3.92],
            Some(50.0),
            Some(1.2),
            Some(0.1),
            None,
        );
    }

    #[test]
    fn test_normalisation_adjusts_max() {
        let n = normalisation_test(
            &mut vec![0.0_f32, 5.0, 10.0, 25.0, 49.0, 100.0],
            &vec![0.0_f32, 0.1, 0.4, 1.5, 3.92, 10.0],
            Some(50.0),
            Some(1.2),
            Some(0.1),
            None,
        );

        normalisation_test(
            &mut vec![0.0_f32, 12.0, 36.0, 60.0, 117.6],
            &vec![0.0_f32, 0.02, 0.12, 0.3, 0.784],
            None,
            None,
            None,
            Some(n),
        );
    }
}
