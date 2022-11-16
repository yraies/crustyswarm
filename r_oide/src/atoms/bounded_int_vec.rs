use std::iter::FromIterator;

use crate::traits::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{BoolCell, Util};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct BoundedIdxVec {
    pub vec: Vec<BoolCell<usize>>,
    pub upper_bound: usize,
}

impl BoundedIdxVec {
    pub fn new_by_idx_count(index_count: usize, size: usize) -> BoundedIdxVec {
        BoundedIdxVec {
            vec: vec![BoolCell::<usize>::new(); size],
            upper_bound: index_count - 1,
        }
    }

    pub fn get_activation_vec(&self) -> Vec<bool> {
        self.vec.iter().map(|bar| bar.is_active()).collect()
    }

    pub fn fill_to(&mut self, size: usize) {
        while self.vec.len() < size {
            self.vec.push(BoolCell::<usize>::new())
        }
    }
}

impl FromIterator<(bool, usize)> for BoundedIdxVec {
    fn from_iter<I: IntoIterator<Item = (bool, usize)>>(iter: I) -> Self {
        let mut c = BoundedIdxVec {
            vec: vec![],
            upper_bound: 0,
        };

        for i in iter {
            c.upper_bound = c.upper_bound.max(i.1);
            c.vec.push(BoolCell {
                active: i.0.into(),
                value: i.1,
            });
        }

        c
    }
}

impl OIDEAdd for BoundedIdxVec {
    fn add(&self, other: &Self) -> Self {
        BoundedIdxVec {
            vec: self
                .vec
                .iter()
                .zip(&other.vec)
                .map(|(first, second)| first.add(second, self.upper_bound))
                .collect(),
            upper_bound: self.upper_bound,
        }
    }
}
impl OIDEDiff for BoundedIdxVec {
    fn difference(&self, other: &Self) -> Self {
        BoundedIdxVec {
            vec: self
                .vec
                .iter()
                .zip(&other.vec)
                .map(|(first, second)| first.diff(second))
                .collect(),
            upper_bound: self.upper_bound,
        }
    }
}
impl OIDEScale for BoundedIdxVec {
    fn scale(&self, factor: f32) -> Self {
        BoundedIdxVec {
            vec: self.vec.iter().map(|cell| cell.scale(factor)).collect(),
            upper_bound: self.upper_bound,
        }
    }
}
impl OIDEOpposite for BoundedIdxVec {
    fn opposite(&self, midpoint: Option<&Self>) -> Self {
        BoundedIdxVec {
            vec: self
                .vec
                .iter()
                .zip(
                    midpoint
                        .map(|m| m.vec.iter().map(Some).collect())
                        .unwrap_or_else(|| vec![None; self.vec.len()]),
                )
                .map(|(cell, mid)| cell.opposite(self.upper_bound, mid))
                .collect(),
            upper_bound: self.upper_bound,
        }
    }
}
impl OIDERandomize for BoundedIdxVec {
    fn random(&self, rng: &mut impl Rng) -> Self {
        let mut copy = self.clone();
        copy.vec = copy
            .vec
            .iter()
            .map(|cell| cell.random(rng, 0, self.upper_bound))
            .collect();
        copy
    }
}
impl OIDECrossover for BoundedIdxVec {
    fn crossover(&self, other: &Self, rng: &mut impl Rng, rate: f64) -> Self {
        BoundedIdxVec {
            vec: self
                .vec
                .iter()
                .zip(other.vec.iter())
                .map(|(s, o)| Util::crossover(s, o, rng, rate))
                .collect(),
            upper_bound: self.upper_bound,
        }
    }
}
impl OIDEBoundApplication for BoundedIdxVec {
    fn apply_bounds(&self, other: &Self) -> Self {
        BoundedIdxVec {
            vec: other
                .vec
                .iter()
                .map(|v| {
                    let mut new_val = v.clone();
                    new_val.value = v.value % (self.upper_bound + 1);
                    new_val
                })
                .collect(),
            upper_bound: self.upper_bound,
        }
    }
}
impl OIDEZero for BoundedIdxVec {
    fn zero(&self) -> Self {
        BoundedIdxVec {
            vec: self.vec.iter().map(|s| s.zero()).collect(),
            upper_bound: self.upper_bound,
        }
    }
}
impl OIDEParameterCount for BoundedIdxVec {
    fn parameter_count(&self) -> usize {
        self.vec.len() * 2
    }
}
impl Visit<f32> for BoundedIdxVec {
    fn visit_with<V: Visitor<f32>>(&self, f: &mut V) -> Result<(), V::Error> {
        for elem in &self.vec {
            elem.active.visit_with(f)?;
            f.handle(elem.value as f32)?;
        }
        Ok(())
    }
}
impl Visit<FeatureTraversal> for BoundedIdxVec {
    fn visit_with<V: Visitor<FeatureTraversal>>(&self, f: &mut V) -> Result<(), V::Error> {
        for i in 0..self.vec.len() {
            f.handle(FeatureTraversal::Push(format!("vec{:02}", i)))?;
            f.handle(FeatureTraversal::Collect("active".to_string()))?;
            f.handle(FeatureTraversal::Collect("value".to_string()))?;
            f.handle(FeatureTraversal::Pop)?;
        }
        Ok(())
    }
}
impl std::hash::Hash for BoundedIdxVec {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vec.iter().for_each(|v| {
            v.active.hash(state);
            v.value.hash(state);
        });
    }
}
impl Differentiable for BoundedIdxVec {}
