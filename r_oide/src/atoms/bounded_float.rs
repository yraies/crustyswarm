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
                let sum = self.offset + other.offset; // -2 self.range <= sum <= 2 self.range
                if sum > self.range {
                    -2f32 * self.range + sum // offset <= 0
                } else if sum < -self.range {
                    2f32 * self.range + sum // 0 <= offset
                } else {
                    sum // -self.range <= offset <= self.range
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
            offset: {
                let diff = self.offset - other.offset; // -2 self.range <= diff <= 2 self.range
                if diff > self.range {
                    -2f32 * self.range + diff // offset <= 0
                } else if diff < -self.range {
                    2f32 * self.range + diff // offset >= 0
                } else {
                    diff // -self.range <= 0 <= self.range
                }
            },
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
            offset: -self.offset,
        }
    }
}
impl OIDERandomize for BoundedFactor {
    fn random(&self, rng: &mut impl Rng) -> Self {
        let mut copy = self.clone();
        copy.offset = rng.sample(Uniform::new_inclusive(-self.range, self.range));
        copy
    }
}
impl OIDEBoundApplication for BoundedFactor {
    fn apply_bounds(&self, other: &Self) -> Self {
        BoundedFactor {
            base: self.base,
            range: self.range,
            offset: (other.get_value() - self.base).clamp(-self.range, self.range),
        }
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
        let base = BoundedFactor::new_with_bounds(0.0, 1.0, 0.0);
        let (width, height, bins) = (120, 30, 10);

        let values: Vec<_> = (0..100000)
            .into_iter()
            .map(|_| base.random(&mut rng).get_value() as f64)
            .collect();
        let hist = Histogram::from_slice(&values, HistogramBins::Count(bins));
        let view = ContinuousView::new().add(hist).x_range(0.0, 1.0);
        let res = view.to_text(width, height).unwrap();
        println!("\n### BASE ###\n{}", res);

        let mut rng = StdRng::seed_from_u64(123123123);
        let base = BoundedFactor::new_with_bounds(0.0, 1.0, 0.0);
        let values: Vec<_> = (0..100000)
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
        let values: Vec<_> = (0..100000)
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
        let values: Vec<_> = (0..100000)
            .into_iter()
            .map(|_| base.random(&mut rng).opposite().get_value() as f64)
            .collect();
        let hist = Histogram::from_slice(&values, HistogramBins::Count(bins));
        let view = ContinuousView::new().add(hist).x_range(0.0, 1.0);
        let res = view.to_text(width, height).unwrap();
        println!("\n### OPP ###\n{}", res);

        let mut rng = StdRng::seed_from_u64(123123123);
        let values: Vec<_> = (0..100000)
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
        let values: Vec<_> = (0..100000)
            .into_iter()
            .map(|_| {
                base.random(&mut rng)
                    .add(
                        &base
                            .random(&mut rng)
                            .difference(&base.random(&mut rng))
                            .opposite(),
                    )
                    .get_value() as f64
            })
            .collect();
        let hist = Histogram::from_slice(&values, HistogramBins::Count(bins));
        let view = ContinuousView::new().add(hist).x_range(0.0, 1.0);
        let res = view.to_text(width, height).unwrap();
        println!("\n### ADDOPPDIFF ###\n{}", res);

        assert!(false);
    }
}
