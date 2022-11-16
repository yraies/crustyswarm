use crate::traits::*;
use rand::{distributions::Uniform, Rng};
use serde::{Deserialize, Serialize};

use super::FloatyBool;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct BoolCell<T> {
    pub active: FloatyBool,
    pub value: T,
}

impl<T> BoolCell<T> {
    pub fn is_active(&self) -> bool {
        self.active.clone().into()
    }
}
impl<T: OIDEZero> BoolCell<T> {
    pub fn zero(&self) -> Self {
        BoolCell {
            active: self.active.zero(),
            value: self.value.zero(),
        }
    }
}

impl BoolCell<usize> {
    pub fn new() -> Self {
        BoolCell {
            active: 0.0.into(),
            value: 0usize,
        }
    }

    pub fn add(&self, other: &Self, index_count: usize) -> Self {
        Self {
            active: self.active.add(&other.active),
            value: (other.value + self.value) % (index_count + 1),
        }
    }

    pub fn diff(&self, other: &Self) -> Self {
        Self {
            active: self.active.difference(&other.active),
            value: (self.value as i64 - other.value as i64).unsigned_abs() as usize,
        }
    }

    pub fn scale(&self, factor: f32) -> Self {
        Self {
            active: self.active.scale(factor), //TODO: deal with factors > 1.0
            value: (self.value as f32 * factor).round() as usize,
        }
    }
    pub fn opposite(&self, index_count: usize, midpoint: Option<&BoolCell<usize>>) -> Self {
        Self {
            active: self.active.opposite(match midpoint {
                Some(m) => Some(&m.active),
                None => None,
            }),
            value: index_count - self.value,
        }
    }
    pub fn random(&self, rng: &mut impl Rng, lower_bound: usize, upper_bound: usize) -> Self {
        BoolCell {
            active: rng.sample(Uniform::new_inclusive(0.0, 1.0)).into(),
            value: rng.gen_range(lower_bound, upper_bound + 1),
        }
    }
    pub fn zero(&self) -> Self {
        BoolCell {
            active: self.active.zero(),
            value: 0,
        }
    }
}
