use crate::traits::*;
use rand::{distributions::Uniform, prelude::*};
use serde::{Deserialize, Serialize};

use super::Util;

//
// Definition
//

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct FloatyBool(f64);

impl FloatyBool {
    pub fn new(val: f32) -> FloatyBool {
        assert!(val >= 0.0);
        assert!(val <= 1.0);
        FloatyBool(val as f64)
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
        FloatyBool(self.0 * factor as f64)
    }
}
impl OIDEOpposite for FloatyBool {
    fn opposite(&self, midpoint: Option<&Self>) -> Self {
        let val = midpoint.map(|m| 2.0 * m.0).unwrap_or(1.0) - self.0;
        if val < 0.0 {
            FloatyBool(-val)
        } else if val > 1.0 {
            FloatyBool(2.0 - val)
        } else {
            FloatyBool(val)
        }
    }
}
impl OIDERandomize for FloatyBool {
    fn random(&self, rng: &mut impl Rng) -> Self {
        FloatyBool(rng.sample(Uniform::new_inclusive(0.0, 1.0)))
    }
}
impl OIDECrossover for FloatyBool {
    fn crossover(&self, other: &Self, rng: &mut impl Rng, rate: f64) -> Self {
        Util::crossover(self, other, rng, rate)
    }
}
impl OIDEBoundApplication for FloatyBool {
    fn apply_bounds(&self, other: &Self) -> Self {
        other.clone()
    }
}
impl OIDEZero for FloatyBool {
    fn zero(&self) -> Self {
        FloatyBool(0.0)
    }
}
impl OIDEParameterCount for FloatyBool {
    fn parameter_count(&self) -> usize {
        1
    }
}
impl Visit<f32> for FloatyBool {
    fn visit_with<V: Visitor<f32>>(&self, f: &mut V) -> Result<(), V::Error> {
        f.handle(self.0 as f32)
    }
}
impl Visit<FeatureTraversal> for FloatyBool {
    fn visit_with<V: Visitor<FeatureTraversal>>(&self, f: &mut V) -> Result<(), V::Error> {
        f.handle(FeatureTraversal::Collect("fbool".to_string()))
    }
}
impl std::hash::Hash for FloatyBool {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_string().hash(state);
    }
}
impl Differentiable for FloatyBool {}

//
// Other
//

impl From<FloatyBool> for bool {
    fn from(other: FloatyBool) -> Self {
        other.is_active()
    }
}

impl From<&FloatyBool> for f32 {
    fn from(other: &FloatyBool) -> Self {
        other.0 as f32
    }
}

impl From<&FloatyBool> for f64 {
    fn from(other: &FloatyBool) -> Self {
        other.0
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

#[cfg(test)]
mod test {
    use crate::prelude::*;
    #[test]
    fn testfoo() {
        let f = FloatyBool::new_false();
        let t = FloatyBool::new_true();
        let fff = f
            .scale(0.333333333)
            .add(&f.scale(0.333333333))
            .add(&f.scale(0.333333333));
        let ttt = t
            .scale(0.333333333)
            .add(&t.scale(0.333333333))
            .add(&t.scale(0.333333333));
        let tf = t.scale(0.5).add(&f.scale(0.5));
        assert_eq!(f32::from(&ttt), 1.0f32);
        assert_eq!(f32::from(&fff), 0.0f32);
        assert_eq!(f32::from(&tf), 0.5f32);

        let ot = t.opposite(Some(&tf));
        let of = f.opposite(Some(&tf));
        assert_eq!(f32::from(&ot), 0.0f32);
        assert_eq!(f32::from(&of), 1.0f32);

        let tish = FloatyBool::new(0.875);
        let fish = FloatyBool::new(0.125);

        let ottish = t.opposite(Some(&tish));
        let otfish = t.opposite(Some(&fish));
        let oftish = f.opposite(Some(&tish));
        let offish = f.opposite(Some(&fish));
        assert_eq!(f32::from(&ottish), 0.75f32);
        assert_eq!(f32::from(&otfish), 0.75f32);
        assert_eq!(f32::from(&oftish), 0.25f32);
        assert_eq!(f32::from(&offish), 0.25f32);
    }
}
