use crate::traits::*;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

//
// Definition
//

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct Fixed<T: Clone>(T);

//
// OIDE
//

impl<T: Clone> OIDEAdd for Fixed<T> {
    fn add(&self, _other: &Self) -> Self {
        self.clone()
    }
}
impl<T: Clone> OIDEDiff for Fixed<T> {
    fn difference(&self, _other: &Self) -> Self {
        self.clone()
    }
}
impl<T: Clone> OIDEScale for Fixed<T> {
    fn scale(&self, _factor: f32) -> Self {
        self.clone()
    }
}
impl<T: Clone> OIDEOpposite for Fixed<T> {
    fn opposite(&self, _: Option<&Self>) -> Self {
        self.clone()
    }
}
impl<T: Clone> OIDERandomize for Fixed<T> {
    fn random(&self, _rng: &mut impl Rng) -> Self {
        self.clone()
    }
}
impl<T: Clone> OIDECrossover for Fixed<T> {
    fn crossover(&self, _other: &Self, _rng: &mut impl Rng, _rate: f64) -> Self {
        self.clone()
    }
}
impl<T: Clone> OIDEBoundApplication for Fixed<T> {
    fn apply_bounds(&self, other: &Self) -> Self {
        other.clone()
    }
}
impl<T: Clone> OIDEZero for Fixed<T> {
    fn zero(&self) -> Self {
        self.clone()
    }
}
impl<T: Clone> OIDEParameterCount for Fixed<T> {
    fn parameter_count(&self) -> usize {
        0
    }
}
impl<T: Clone> Visit<f32> for Fixed<T> {
    fn visit_with<V: Visitor<f32>>(&self, _f: &mut V) -> Result<(), V::Error> {
        Ok(())
    }
}
impl<T: Clone> Visit<FeatureTraversal> for Fixed<T> {
    fn visit_with<V: Visitor<FeatureTraversal>>(&self, _f: &mut V) -> Result<(), V::Error> {
        Ok(())
    }
}
impl<T: Clone> std::hash::Hash for Fixed<T> {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {}
}
impl<T: Differentiable> Differentiable for Fixed<T> {}

//
// Other
//

impl<T: Clone> std::ops::Deref for Fixed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Clone> AsRef<T> for Fixed<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: Clone> From<T> for Fixed<T> {
    fn from(other: T) -> Self {
        Fixed(other)
    }
}
