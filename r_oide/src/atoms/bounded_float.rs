use crate::traits::*;
use rand::{distributions::Uniform, prelude::*};
use serde::{Deserialize, Serialize};

use super::Util;

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

    pub fn new_from_f32(offset: f32) -> BoundedFactor {
        BoundedFactor {
            base: offset,
            range: 0.0,
            offset: 0.0,
        }
    }

    pub fn get_value(&self) -> f32 {
        self.base + self.offset.abs()
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
        self.offset.abs()
    }

    /// Ensures that the offsets is reflected into the given bounds.
    /// Assumes: -2 range <= offset <= 2 range
    fn repair(mut self) -> BoundedFactor {
        self.offset = {
            if self.offset > self.range {
                -2f32 * self.range + self.offset // offset <= 0
            } else if self.offset < -self.range {
                2f32 * self.range + self.offset // 0 <= offset
            } else {
                self.offset // -self.range <= offset <= self.range
            }
        };
        self
    }
}

//
// OIDE
//

impl OIDEAdd for BoundedFactor {
    fn add(&self, other: &Self) -> Self {
        BoundedFactor {
            offset: self.offset + other.offset,
            ..*self
        }
        .repair()
    }
}
impl OIDEDiff for BoundedFactor {
    fn difference(&self, other: &Self) -> Self {
        BoundedFactor {
            offset: self.offset - other.offset,
            ..*self
        }
        .repair()
    }
}
impl OIDEScale for BoundedFactor {
    fn scale(&self, factor: f32) -> Self {
        BoundedFactor {
            offset: self.offset * factor, //TODO: Handle factor > 1.0
            ..*self
        }
    }
}
impl OIDEOpposite for BoundedFactor {
    fn opposite(&self, midpoint: Option<&Self>) -> Self {
        BoundedFactor {
            offset: 2.0 * midpoint.map(|m| m.offset).unwrap_or(0.0) - self.offset,
            ..*self
        }
        .repair()
    }
}
impl OIDERandomize for BoundedFactor {
    fn random(&self, rng: &mut impl Rng) -> Self {
        let mut copy = self.clone();
        copy.offset = rng.sample(Uniform::new_inclusive(-self.range, self.range));
        copy
    }
}
impl OIDECrossover for BoundedFactor {
    fn crossover(&self, other: &Self, rng: &mut impl Rng, rate: f64) -> Self {
        Util::crossover(self, other, rng, rate)
    }
}
impl OIDEBoundApplication for BoundedFactor {
    fn apply_bounds(&self, other: &Self) -> Self {
        BoundedFactor {
            offset: (other.get_value() - self.base).clamp(-self.range, self.range),
            ..*self
        }
    }
}
impl OIDEZero for BoundedFactor {
    fn zero(&self) -> Self {
        BoundedFactor {
            offset: 0.0,
            ..*self
        }
    }
}
impl OIDEParameterCount for BoundedFactor {
    fn parameter_count(&self) -> usize {
        1
    }
}
impl Visit<f32> for BoundedFactor {
    fn visit_with<V: Visitor<f32>>(&self, f: &mut V) -> Result<(), V::Error> {
        f.handle(self.get_value())
    }
}
impl Visit<FeatureTraversal> for BoundedFactor {
    fn visit_with<V: Visitor<FeatureTraversal>>(&self, f: &mut V) -> Result<(), V::Error> {
        f.handle(FeatureTraversal::Collect("factor".to_string()))
    }
}
impl std::hash::Hash for BoundedFactor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_offset().to_string().hash(state)
    }
}
impl Differentiable for BoundedFactor {}

#[cfg(test)]
mod test {
    extern crate plotlib;
    use super::BoundedFactor;
    use crate::prelude::*;
    use plotlib::{
        repr::{Histogram, HistogramBins},
        view::{ContinuousView, View},
    };
    use rand::prelude::*;

    #[test]
    fn unif_dist() {
        let mut rng = StdRng::seed_from_u64(123123123);
        let repeats = 10000;
        let base = BoundedFactor::new_with_bounds(0.0, 1.0, 0.0);
        let (width, height, bins) = (120, 30, 50);

        let values: Vec<_> = (0..repeats)
            .into_iter()
            .map(|_| base.random(&mut rng).get_value() as f64)
            .collect();
        let hist = Histogram::from_slice(&values, HistogramBins::Count(bins));
        let view = ContinuousView::new()
            .add(hist)
            .x_range(0.0, 1.0)
            .x_label("value")
            .y_label("count");
        let res = view.to_text(width, height).unwrap();
        println!("\n### BASE ###\n{}", res);

        let mut rng = StdRng::seed_from_u64(123123123);
        let base = BoundedFactor::new_with_bounds(0.0, 1.0, 0.0);
        let values: Vec<_> = (0..repeats)
            .into_iter()
            .map(|_| {
                base.random(&mut rng)
                    .difference(&base.random(&mut rng))
                    .get_value() as f64
            })
            .collect();
        let hist = Histogram::from_slice(&values, HistogramBins::Count(bins));
        let view = ContinuousView::new().add(hist);
        let res = view.to_text(width, height).unwrap();
        println!("\n### DIFF ###\n{}", res);

        let mut rng = StdRng::seed_from_u64(123123123);
        let values: Vec<_> = (0..repeats)
            .into_iter()
            .map(|_| {
                base.random(&mut rng)
                    .add(&base.random(&mut rng))
                    .get_value() as f64
            })
            .collect();
        let hist = Histogram::from_slice(&values, HistogramBins::Count(bins));
        let view = ContinuousView::new().add(hist).x_range(0.0, 1.0);
        let res = view.to_text(width, height).unwrap();
        println!("\n### ADD ###\n{}", res);

        let mut rng = StdRng::seed_from_u64(123123123);
        let values: Vec<_> = (0..repeats)
            .into_iter()
            .map(|_| base.random(&mut rng).opposite(None).get_value() as f64)
            .collect();
        let hist = Histogram::from_slice(&values, HistogramBins::Count(bins));
        let view = ContinuousView::new().add(hist).x_range(0.0, 1.0);
        let res = view.to_text(width, height).unwrap();
        println!("\n### OPP ###\n{}", res);

        let mut rng = StdRng::seed_from_u64(123123123);
        let values: Vec<_> = (0..repeats)
            .into_iter()
            .map(|_| {
                base.random(&mut rng)
                    .add(&base.random(&mut rng).difference(&base.random(&mut rng)))
                    .get_value() as f64
            })
            .collect();
        let hist = Histogram::from_slice(&values, HistogramBins::Count(bins));
        let view = ContinuousView::new().add(hist).x_range(0.0, 1.0);
        let res = view.to_text(width, height).unwrap();
        println!("\n### ADDDIFF ###\n{}", res);

        let mut rng = StdRng::seed_from_u64(123123123);
        let values: Vec<_> = (0..repeats)
            .into_iter()
            .map(|_| {
                base.random(&mut rng)
                    .add(
                        &base
                            .random(&mut rng)
                            .difference(&base.random(&mut rng))
                            .opposite(None),
                    )
                    .get_value() as f64
            })
            .collect();
        let hist = Histogram::from_slice(&values, HistogramBins::Count(bins));
        let view = ContinuousView::new().add(hist).x_range(0.0, 1.0);
        let res = view.to_text(width, height).unwrap();
        println!("\n### ADDOPPDIFF ###\n{}", res);

        //assert!(false);
    }
}
