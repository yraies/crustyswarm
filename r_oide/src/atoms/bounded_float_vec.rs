use std::iter::FromIterator;

use crate::traits::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{BoolCell, BoundedFactor, FloatyBool, Util};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct BoundedFactorVec {
    pub vec: Vec<BoolCell<BoundedFactor>>,
}

impl BoundedFactorVec {
    pub fn new(lower_bound: f32, upper_bound: f32, size: usize) -> BoundedFactorVec {
        let base_bf = BoundedFactor::new_with_bounds(lower_bound, upper_bound, lower_bound);
        BoundedFactorVec {
            vec: (0..size)
                .map(|_| BoolCell {
                    active: FloatyBool::new_false(),
                    value: base_bf.clone(),
                })
                .collect::<Vec<_>>(),
        }
    }

    pub fn into_f32_vec(&self) -> Vec<f32> {
        self.vec
            .iter()
            .map(|bar| {
                if bar.is_active() {
                    bar.value.get_value()
                } else {
                    0.0
                }
            })
            .collect()
    }

    pub fn fill_to(&mut self, size: usize) {
        while self.vec.len() < size {
            self.vec.push(BoolCell {
                active: FloatyBool::new_false(),
                value: BoundedFactor::new_from_f32(0.0),
            })
        }
    }
}

impl FromIterator<(bool, f32)> for BoundedFactorVec {
    fn from_iter<I: IntoIterator<Item = (bool, f32)>>(iter: I) -> Self {
        let mut c = BoundedFactorVec { vec: vec![] };
        let mut lower_bound = f32::MAX;
        let mut upper_bound = f32::MIN;

        for i in iter {
            lower_bound = lower_bound.min(i.1);
            upper_bound = upper_bound.max(i.1);
            c.vec.push(BoolCell {
                active: i.0.into(),
                value: BoundedFactor::new_from_f32(i.1),
            });
        }

        c.vec.iter_mut().for_each(|v| {
            v.value = BoundedFactor::new_with_bounds(lower_bound, upper_bound, v.value.get_value());
        });

        c
    }
}

impl OIDEAdd for BoundedFactorVec {
    fn add(&self, other: &Self) -> Self {
        BoundedFactorVec {
            vec: self
                .vec
                .iter()
                .zip(&other.vec)
                .map(|(first, second)| BoolCell {
                    active: first.active.add(&second.active),
                    value: first.value.add(&second.value),
                })
                .collect(),
        }
    }
}
impl OIDEDiff for BoundedFactorVec {
    fn difference(&self, other: &Self) -> Self {
        BoundedFactorVec {
            vec: self
                .vec
                .iter()
                .zip(&other.vec)
                .map(|(first, second)| BoolCell {
                    active: first.active.difference(&second.active),
                    value: first.value.difference(&second.value),
                })
                .collect(),
        }
    }
}
impl OIDEScale for BoundedFactorVec {
    fn scale(&self, factor: f32) -> Self {
        BoundedFactorVec {
            vec: self
                .vec
                .iter()
                .map(|first| BoolCell {
                    active: first.active.scale(factor),
                    value: first.value.scale(factor),
                })
                .collect(),
        }
    }
}
impl OIDEOpposite for BoundedFactorVec {
    fn opposite(&self, midpoint: Option<&Self>) -> Self {
        BoundedFactorVec {
            vec: self
                .vec
                .iter()
                .zip(
                    midpoint
                        .map(|m| m.vec.iter().map(Some).collect())
                        .unwrap_or_else(|| vec![None; self.vec.len()]),
                )
                .map(|(cell, mid)| BoolCell {
                    active: cell.active.opposite(mid.map(|m| &m.active)),
                    value: cell.value.opposite(mid.map(|m| &m.value)),
                })
                .collect(),
        }
    }
}
impl OIDERandomize for BoundedFactorVec {
    fn random(&self, rng: &mut impl Rng) -> Self {
        let mut copy = self.clone();
        copy.vec = copy
            .vec
            .iter()
            .map(|first| BoolCell {
                active: first.active.random(rng),
                value: first.value.random(rng),
            })
            .collect();
        copy
    }
}
impl OIDECrossover for BoundedFactorVec {
    fn crossover(&self, other: &Self, rng: &mut impl Rng, rate: f64) -> Self {
        BoundedFactorVec {
            vec: self
                .vec
                .iter()
                .zip(other.vec.iter())
                .map(|(s, o)| Util::crossover(s, o, rng, rate))
                .collect(),
        }
    }
}
impl OIDEBoundApplication for BoundedFactorVec {
    fn apply_bounds(&self, other: &Self) -> Self {
        BoundedFactorVec {
            vec: other.vec.clone(),
        }
    }
}
impl OIDEZero for BoundedFactorVec {
    fn zero(&self) -> Self {
        BoundedFactorVec {
            vec: self.vec.iter().map(|s| s.zero()).collect(),
        }
    }
}
impl OIDEParameterCount for BoundedFactorVec {
    fn parameter_count(&self) -> usize {
        self.vec.len() * 2
    }
}
impl Visit<f32> for BoundedFactorVec {
    fn visit_with<V: Visitor<f32>>(&self, f: &mut V) -> Result<(), V::Error> {
        for elem in &self.vec {
            elem.active.visit_with(f)?;
            elem.value.visit_with(f)?;
        }
        Ok(())
    }
}
impl Visit<FeatureTraversal> for BoundedFactorVec {
    fn visit_with<V: Visitor<FeatureTraversal>>(&self, f: &mut V) -> Result<(), V::Error> {
        for (i, _) in self.vec.iter().enumerate() {
            f.handle(FeatureTraversal::Push(format!("vec{:02}", i)))?;
            f.handle(FeatureTraversal::Collect("active".to_string()))?;
            f.handle(FeatureTraversal::Collect("value".to_string()))?;
            f.handle(FeatureTraversal::Pop)?;
        }
        Ok(())
    }
}
impl std::hash::Hash for BoundedFactorVec {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vec.iter().for_each(|v| {
            v.active.hash(state);
            v.value.hash(state);
        });
    }
}
impl Differentiable for BoundedFactorVec {}
