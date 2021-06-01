use crate::traits::*;
use rand::{distributions::Uniform, prelude::*};
use serde::{Deserialize, Serialize};

//
// Definition
//

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct FloatyBool(f32);

impl FloatyBool {
    pub fn new(val: f32) -> FloatyBool {
        assert!(val >= 0.0);
        assert!(val <= 1.0);
        FloatyBool(val)
    }
    pub fn new_true() -> FloatyBool {
        FloatyBool(1.0)
    }
    pub fn new_false() -> FloatyBool {
        FloatyBool(0.0)
    }

    pub fn is_active(&self) -> bool {
        self.0 >= 0.5
    }
}

//
// OIDE
//

impl OIDEAdd for FloatyBool {
    fn add(&self, other: &Self) -> Self {
        let temp_res = self.0 + other.0;
        FloatyBool(if temp_res > 1.0 {
            2.0 - temp_res
        } else {
            temp_res
        })
    }
}
impl OIDEDiff for FloatyBool {
    fn difference(&self, other: &Self) -> Self {
        FloatyBool((self.0 - other.0).abs())
    }
}
impl OIDEScale for FloatyBool {
    fn scale(&self, factor: f32) -> Self {
        FloatyBool(self.0 * factor)
    }
}
impl OIDEOpposite for FloatyBool {
    fn opposite(&self) -> Self {
        FloatyBool(1.0 - self.0)
    }
}
impl OIDERandomize for FloatyBool {
    fn random(&self, rng: &mut impl Rng) -> Self {
        FloatyBool(rng.sample(Uniform::new_inclusive(0.0, 1.0)))
    }
}
impl OIDEBoundApplication for FloatyBool {
    fn apply_bounds(&self, other: &Self) -> Self {
        other.clone()
    }
}
impl Differentiable for FloatyBool {}

//
// Other
//

impl std::ops::Deref for FloatyBool {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<FloatyBool> for bool {
    fn from(other: FloatyBool) -> Self {
        other.is_active()
    }
}

impl From<f32> for FloatyBool {
    fn from(other: f32) -> Self {
        FloatyBool::new(other)
    }
}

impl From<bool> for FloatyBool {
    fn from(other: bool) -> Self {
        if other {
            FloatyBool::new_true()
        } else {
            FloatyBool::new_false()
        }
    }
}
