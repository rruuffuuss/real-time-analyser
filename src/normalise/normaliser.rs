pub trait Normaliser {
    fn normalise(&mut self, bars: &mut [f32]);
}

#[cfg(test)]
pub mod tests {
    pub fn assert_eq_normalised_vec(result: &Vec<f32>, expected: &Vec<f32>) {
        result
            .iter()
            .zip(expected.iter())
            .for_each(|(i, o)| assert!((i - o).abs() < 1e-5));
    }
}
