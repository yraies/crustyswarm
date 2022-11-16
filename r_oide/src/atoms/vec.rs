use super::Util;
use crate::traits::*;
use rand::prelude::*;
use std::fmt::Debug;

impl<T: OIDEAdd> OIDEAdd for Vec<T> {
    fn add(&self, other: &Self) -> Self {
        assert_eq!(self.len(), other.len());
        self.iter()
            .zip(other.iter())
            .map(|(s, o)| s.add(o))
            .collect()
    }
}
impl<T: OIDEDiff> OIDEDiff for Vec<T> {
    fn difference(&self, other: &Self) -> Self {
        assert_eq!(self.len(), other.len());
        self.iter()
            .zip(other.iter())
            .map(|(s, o)| s.difference(o))
            .collect()
    }
}
impl<T: OIDEScale> OIDEScale for Vec<T> {
    fn scale(&self, factor: f32) -> Self {
        self.iter().map(|s| s.scale(factor)).collect()
    }
}
impl<T: OIDEOpposite> OIDEOpposite for Vec<T> {
    fn opposite(&self, midpoint: Option<&Self>) -> Self {
        self.iter()
            .zip(
                midpoint
                    .map(|m| m.iter().map(Some).collect())
                    .unwrap_or_else(|| vec![None; self.len()]),
            )
            .map(|(s, m)| s.opposite(m))
            .collect()
    }
}
impl<T: OIDERandomize> OIDERandomize for Vec<T> {
    fn random(&self, rng: &mut impl Rng) -> Self {
        self.iter().map(|s| s.random(rng)).collect()
    }
}
impl<T: OIDECrossover + Clone> OIDECrossover for Vec<T> {
    fn crossover(&self, other: &Self, rng: &mut impl Rng, rate: f64) -> Self {
        self.iter()
            .zip(other.iter())
            .map(|(s, o)| Util::crossover(s, o, rng, rate))
            .collect()
    }
}
impl<T: OIDEBoundApplication + Debug> OIDEBoundApplication for Vec<T> {
    fn apply_bounds(&self, other: &Self) -> Self {
        assert_eq!(
            self.len(),
            other.len(),
            "Left: {:?}\nRight: {:?}",
            self,
            other
        );
        self.iter()
            .zip(other.iter())
            .map(|(s, o)| s.apply_bounds(o))
            .collect()
    }
}
impl<T: OIDEZero> OIDEZero for Vec<T> {
    fn zero(&self) -> Self {
        self.iter().map(|s| s.zero()).collect()
    }
}
impl<T: OIDEParameterCount> OIDEParameterCount for Vec<T> {
    fn parameter_count(&self) -> usize {
        self.iter().map(|v| v.parameter_count()).sum()
    }
}
impl<T: Visit<f32>> Visit<f32> for Vec<T> {
    fn visit_with<V: Visitor<f32>>(&self, f: &mut V) -> Result<(), V::Error> {
        for elem in self {
            elem.visit_with(f)?;
        }
        Ok(())
    }
}
impl<T: Visit<FeatureTraversal>> Visit<FeatureTraversal> for Vec<T> {
    fn visit_with<V: Visitor<FeatureTraversal>>(&self, f: &mut V) -> Result<(), V::Error> {
        for (i, _) in self.iter().enumerate() {
            f.handle(FeatureTraversal::Push(format!("vec{:02}", i)))?;
            self[i].visit_with(f)?;
            f.handle(FeatureTraversal::Pop)?;
        }
        Ok(())
    }
}
impl<T: Differentiable + Debug> Differentiable for Vec<T> {}
