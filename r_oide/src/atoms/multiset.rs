use std::iter::FromIterator;

use crate::traits::*;
use rand::{distributions::Uniform, prelude::*};
use serde::{Deserialize, Serialize};

use super::Util;

//
// Definition
//

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct IndexMultiset(Vec<f32>);

impl IndexMultiset {
    pub fn new_with_size(size: usize) -> IndexMultiset {
        IndexMultiset(vec![0.0; size])
    }
    pub fn get_indices(&self) -> Vec<usize> {
        self.0
            .iter()
            .map(|s| s.abs())
            .enumerate()
            .flat_map(|(idx, s)| vec![idx; s.trunc() as usize])
            .collect()
    }
}

//
// OIDE
//

/// Returns v in [0,inf[
impl OIDEAdd for IndexMultiset {
    fn add(&self, other: &Self) -> Self {
        assert_eq!(self.0.len(), other.0.len());
        self.0
            .iter()
            .zip(&other.0)
            .map(|(s, o)| (s + o).abs())
            .collect()
    }
}

/// Returns v in ]-inf,inf[
impl OIDEDiff for IndexMultiset {
    fn difference(&self, other: &Self) -> Self {
        assert_eq!(self.0.len(), other.0.len());
        self.0.iter().zip(&other.0).map(|(s, o)| (s - o)).collect()
    }
}

/// Returns v in ]-inf,inf[
impl OIDEScale for IndexMultiset {
    fn scale(&self, factor: f32) -> Self {
        self.0.iter().map(|s| s * factor).collect()
    }
}

/// Returns v in ]-inf,inf[
impl OIDEOpposite for IndexMultiset {
    fn opposite(&self, _midpoint: Option<&Self>) -> Self {
        self.0.iter().map(|s| -s).collect()

        //let (max_val, target_sum) = self
        //    .0
        //    .iter()
        //    .map(|s| s.abs())
        //    .fold((0.0f32, 0.0f32), |(max, total), next| {
        //        (max.max(next), total + next)
        //    });

        //let mut opposite: Vec<f32> = self
        //    .0
        //    .iter()
        //    .map(|s| (max_val - s.abs()).copysign(*s))
        //    .collect();
        //let normalization_factor: f32 = target_sum / opposite.iter().map(|o| o.abs()).sum::<f32>();
        //for i in 0..opposite.len() {
        //    opposite[i] = opposite[i] * normalization_factor;
        //}
        //opposite.into()
    }
}

/// Returns v in ]-inf,inf[
impl OIDEBoundApplication for IndexMultiset {
    fn apply_bounds(&self, other: &Self) -> Self {
        assert_eq!(self.0.len(), other.0.len());
        other.clone()

        //let current_sum: f32 = other.0.iter().map(|s| s.abs()).sum();

        //if current_sum == 0.0 {
        //    return other.clone();
        //}

        //let target_sum: f32 = self.0.iter().map(|s| s.abs()).sum();

        //let normalization_factor: f32 = target_sum / current_sum;
        //other.0.iter().map(|s| s * normalization_factor).collect()
    }
}

/// Returns v in [0,1]
impl OIDERandomize for IndexMultiset {
    fn random(&self, rng: &mut impl Rng) -> Self {
        let p = 1.0 + (1.0 / self.0.len() as f32);
        self.0
            .iter()
            .map(|_| rng.sample(Uniform::new_inclusive(0.0, p)))
            .collect()
    }
}

/// Returns v in ]-inf,inf[
impl OIDECrossover for IndexMultiset {
    fn crossover(&self, other: &Self, rng: &mut impl Rng, rate: f64) -> Self {
        self.0
            .iter()
            .zip(other.0.iter())
            .map(|(s, o)| Util::crossover(s, o, rng, rate))
            .collect()
    }
}

impl OIDEZero for IndexMultiset {
    fn zero(&self) -> Self {
        self.0.iter().map(|_| 0.0).collect()
    }
}

impl OIDEParameterCount for IndexMultiset {
    fn parameter_count(&self) -> usize {
        self.0.len()
    }
}

impl Visit<f32> for IndexMultiset {
    fn visit_with<V: Visitor<f32>>(&self, f: &mut V) -> Result<(), V::Error> {
        for elem in &self.0 {
            f.handle(elem.abs())?;
        }
        Ok(())
    }
}

impl Visit<FeatureTraversal> for IndexMultiset {
    fn visit_with<V: Visitor<FeatureTraversal>>(&self, f: &mut V) -> Result<(), V::Error> {
        for (i, _) in self.0.iter().enumerate() {
            f.handle(FeatureTraversal::Collect(format!("mset{:02}", i)))?;
        }
        Ok(())
    }
}

impl std::hash::Hash for IndexMultiset {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.iter().for_each(|v| v.abs().to_string().hash(state))
    }
}

impl Differentiable for IndexMultiset {}

//
// Other
//

impl From<Vec<f32>> for IndexMultiset {
    fn from(base: Vec<f32>) -> Self {
        IndexMultiset(base)
    }
}

impl From<IndexMultiset> for Vec<f32> {
    fn from(base: IndexMultiset) -> Self {
        base.0
    }
}

impl FromIterator<f32> for IndexMultiset {
    fn from_iter<I: IntoIterator<Item = f32>>(iter: I) -> Self {
        iter.into_iter().collect::<Vec<f32>>().into()
    }
}
