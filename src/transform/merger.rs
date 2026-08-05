use rustfft::num_complex::Complex;

use crate::transform::merger::realisor::Realisor;

pub trait Merger {
    //fn merge(&self, frequency_bins: &[Complex<f32>]) -> Vec<f32>;
    fn merge_into_slice(&self, frequency_bins: &[Complex<f32>], output: &mut [f32]);
}

pub(super) mod aggregator {

    use std::marker::PhantomData;

    use super::Realisor;
    use rustfft::num_complex::Complex;

    ///approach used to aggregate multiple FFT output bins into a single output bar
    pub trait Aggregator {
        fn aggregate(seperate: &[Complex<f32>]) -> f32;
    }

    pub(crate) struct Mean<Rl: Realisor> {
        _realise_strategy: PhantomData<Rl>,
    }
    impl<Rl: Realisor> Aggregator for Mean<Rl> {
        #[inline(always)]
        fn aggregate(seperate: &[Complex<f32>]) -> f32 {
            Sum::<Rl>::aggregate(seperate) / seperate.len() as f32
        }
    }

    pub(crate) struct Sum<Rl: Realisor> {
        _realise_strategy: PhantomData<Rl>,
    }
    impl<Rl: Realisor> Aggregator for Sum<Rl> {
        #[inline(always)]
        fn aggregate(seperate: &[Complex<f32>]) -> f32 {
            seperate.iter().map(|c| Rl::realise(*c)).sum::<f32>()
        }
    }

    pub(crate) struct Max<Rl: Realisor> {
        _realise_strategy: PhantomData<Rl>,
    }
    impl<Rl: Realisor> Aggregator for Max<Rl> {
        #[inline(always)]
        fn aggregate(seperate: &[Complex<f32>]) -> f32 {
            seperate
                .iter()
                .map(|c| Rl::realise(*c))
                .reduce(f32::max)
                .unwrap_or(0_f32)
        }
    }
}

///property of the complex FFT output bin to use as a float
pub(super) mod realisor {

    use rustfft::num_complex::{Complex, ComplexFloat};

    pub trait Realisor {
        fn realise(comp: Complex<f32>) -> f32;
    }

    pub(crate) struct Magnitude;
    impl Realisor for Magnitude {
        #[inline(always)]
        fn realise(comp: Complex<f32>) -> f32 {
            comp.norm()
        }
    }

    pub(crate) struct MagnitudeSquared;
    impl Realisor for MagnitudeSquared {
        #[inline(always)]
        fn realise(comp: Complex<f32>) -> f32 {
            comp.norm_sqr()
        }
    }

    pub(crate) struct Real;
    impl Realisor for Real {
        #[inline(always)]
        fn realise(comp: Complex<f32>) -> f32 {
            comp.re()
        }
    }

    pub(crate) struct Imaginary;
    impl Realisor for Imaginary {
        #[inline(always)]
        fn realise(comp: Complex<f32>) -> f32 {
            comp.im()
        }
    }
}
