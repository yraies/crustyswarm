use crate::traits::*;
use rand::{distributions::Uniform, prelude::*};
use serde::{Deserialize, Serialize};

//
// Definition
//

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct BoundedFactor {
    base: f32,
    range: f32,
    offset: f32,
}

impl BoundedFactor {
    pub fn new_with_bounds(lower: f32, upper: f32, value: f32) -> BoundedFactor {
        BoundedFactor {
            base: lower,
            range: upper - lower,
            offset: value - lower,
        }
    }

    pub fn new_with_base(base: f32, range: f32, offset: f32) -> BoundedFactor {
        assert!(range >= 0.0);
        BoundedFactor {
            base,
            range,
            offset,
        }
    }

    pub fn new_from_f32(offsetue: f32) -> BoundedFactor {
        BoundedFactor {
            base: offsetue,
            range: 0.0,
            offset: 0.0,
        }
    }

    pub fn get_value(&self) -> f32 {
        self.base + self.offset
    }
    pub fn get_upper_bound(&self) -> f32 {
        self.base + self.range
    }
    pub fn get_lower_bound(&self) -> f32 {
        self.base
    }
    pub fn get_range(&self) -> f32 {
        self.range
    }
    pub fn get_offset(&self) -> f32 {
        self.offset
    }
}

//
// OIDE
//

impl OIDEAdd for BoundedFactor {
    fn add(&self, other: &Self) -> Self {
        BoundedFactor {
            base: self.base,
            range: self.range,
            offset: {
                let sum = self.offset + other.offset; // 0 <= sum <= 2 self.range
                if sum > self.range {
                    2f32 * self.range - sum
                } else {
                    sum
                }
            },
        }
    }
}
impl OIDEDiff for BoundedFactor {
    fn difference(&self, other: &Self) -> Self {
        BoundedFactor {
            base: self.base,
            range: self.range,
            offset: (self.offset - other.offset).abs(), // - self.range <= diff <= self.range
        }
    }
}
impl OIDEScale for BoundedFactor {
    fn scale(&self, factor: f32) -> Self {
        BoundedFactor {
            base: self.base,
            range: self.range,
            offset: self.offset * factor, //TODO: Handle factor > 1.0
        }
    }
}
impl OIDEOpposite for BoundedFactor {
    fn opposite(&self) -> Self {
        BoundedFactor {
            base: self.base,
            range: self.range,
            offset: self.range - self.offset,
        }
    }
}
impl OIDERandomize for BoundedFactor {
    fn random(&self, rng: &mut impl Rng) -> Self {
        let mut copy = self.clone();
        copy.offset = rng.sample(Uniform::new_inclusive(0.0, self.range));
        copy
    }
}
impl OIDEBoundApplication for BoundedFactor {
    fn apply_bounds(&self, other: &Self) -> Self {
        BoundedFactor {
            base: self.base,
            range: self.range,
            offset: (other.get_value() - self.base).clamp(0.0, self.range),
        }
    }
}
impl Differentiable for BoundedFactor {}
