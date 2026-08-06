use crate::normalise::normaliser::Normaliser;

pub struct ConstantNormaliser {
    factor: f32,
}

impl ConstantNormaliser {
    pub fn new(factor: f32) -> ConstantNormaliser {
        ConstantNormaliser { factor }
    }
}

impl Normaliser for ConstantNormaliser {
    fn normalise(&mut self, bars: &mut [f32]) {
        bars.iter_mut().enumerate().for_each(|(i, b)| {
            *b = (*b / self.factor) * (i as f32);
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::normalise::constant_normaliser::ConstantNormaliser;
    use crate::normalise::normaliser::Normaliser;
    use crate::normalise::normaliser::tests;

    #[test]
    fn test_normalisation_normalises_values() {
        let mut input = vec![0_f32, 1_f32, 10_f32];

        let mut n = ConstantNormaliser::new(10_f32);
        n.normalise(&mut input);

        let expected_output = vec![0_f32, 0.1, 2_f32];
        tests::assert_eq_normalised_vec(&input, &expected_output);
    }
}
