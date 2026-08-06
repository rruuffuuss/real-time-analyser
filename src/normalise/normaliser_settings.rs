use crate::normalise::{
    constant_normaliser::ConstantNormaliser, factor_normaliser::FactorNormaliser,
    normaliser::Normaliser,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum NormaliserSettings {
    Constant {
        factor: f32,
    },
    Factor {
        growth_factor: f32,
        decay_factor: f32,
    },
}

impl NormaliserSettings {
    pub fn build(self) -> Box<dyn Normaliser> {
        match self {
            NormaliserSettings::Constant { factor } => Box::new(ConstantNormaliser::new(factor)),
            NormaliserSettings::Factor {
                growth_factor,
                decay_factor,
            } => Box::new(FactorNormaliser::new(1_f32, growth_factor, decay_factor)),
        }
    }

    pub fn default() -> NormaliserSettings {
        NormaliserSettings::Factor {
            growth_factor: 1.2,
            decay_factor: 0.5,
        }
    }
}
