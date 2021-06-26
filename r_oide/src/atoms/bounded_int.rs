use crate::traits::*;
use rand::{distributions::Uniform, prelude::*};
use serde::{Deserialize, Serialize};

use super::Util;

//
// Definition
//

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct BoundedInt {
    base: i32,
    range: i32,
    offset: i32,
}

impl BoundedInt {
    pub fn new_with_bounds(lower: i32, upper: i32, value: i32) -> BoundedInt {
        assert!(lower <= value);
        assert!(value <= upper);
        BoundedInt {
            base: lower,
            range: upper - lower,
            offset: value - lower,
        }
    }

    pub fn new_with_base(base: i32, range: i32, offset: i32) -> BoundedInt {
        assert!(range >= 0);
        assert!(offset >= 0);
        BoundedInt {
            base,
            range,
            offset,
        }
    }

    pub fn new_from_i32(offset: i32) -> BoundedInt {
        BoundedInt {
            base: offset,
            range: 0,
            offset: 0,
        }
    }

    pub fn get_value(&self) -> i32 {
        self.base + self.offset.abs()
    }
    pub fn get_upper_bound(&self) -> i32 {
        self.base + self.range
    }
    pub fn get_lower_bound(&self) -> i32 {
        self.base
    }
    pub fn get_range(&self) -> i32 {
        self.range
    }
    pub fn get_offset(&self) -> i32 {
        self.offset.abs()
    }
}

//
// OIDE
//

impl OIDEAdd for BoundedInt {
    fn add(&self, other: &Self) -> Self {
        BoundedInt {
            offset: {
                let sum = self.offset + other.offset; // -2 self.range <= sum <= 2 self.range
                if sum > self.range {
                    -2i32 * self.range + sum // offset <= 0
                } else if sum < -self.range {
                    2i32 * self.range + sum // 0 <= offset
                } else {
                    sum // -self.range <= offset <= self.range
                }
            },
            ..*self
        }
    }
}
impl OIDEDiff for BoundedInt {
    fn difference(&self, other: &Self) -> Self {
        BoundedInt {
            offset: {
                let diff = self.offset - other.offset; // -2 self.range <= diff <= 2 self.range
                if diff > self.range {
                    -2i32 * self.range + diff // offset <= 0
                } else if diff < -self.range {
                    2i32 * self.range + diff // offset >= 0
                } else {
                    diff // -self.range <= 0 <= self.range
                }
            },
            ..*self
        }
    }
}
impl OIDEScale for BoundedInt {
    fn scale(&self, factor: f32) -> Self {
        BoundedInt {
            offset: (self.offset as f32 * factor) as i32, //TODO: Handle factor > 1.0
            ..*self
        }
    }
}
impl OIDEOpposite for BoundedInt {
    fn opposite(&self, midpoint: Option<&Self>) -> Self {
        BoundedInt {
            base: self.base,
            range: self.range,
            offset: 2 * midpoint.map(|m| m.offset).unwrap_or(0) - self.offset,
        }
    }
}
impl OIDERandomize for BoundedInt {
    fn random(&self, rng: &mut impl Rng) -> Self {
        let mut copy = self.clone();

        copy.offset =
            rng.sample(Uniform::new_inclusive(0, self.range)) * [1, -1].choose(rng).unwrap();
        copy
    }
}
impl OIDECrossover for BoundedInt {
    fn crossover(&self, other: &Self, rng: &mut impl Rng, rate: f64) -> Self {
        Util::crossover(self, other, rng, rate)
    }
}
impl OIDEBoundApplication for BoundedInt {
    fn apply_bounds(&self, other: &Self) -> Self {
        BoundedInt {
            offset: (other.get_value() - self.base).clamp(-self.range, self.range),
            ..*self
        }
    }
}
impl OIDEZero for BoundedInt {
    fn zero(&self) -> Self {
        BoundedInt { offset: 0, ..*self }
    }
}
impl Differentiable for BoundedInt {}

#[cfg(test)]
mod test {
    extern crate plotlib;
    use super::BoundedInt;
    use crate::prelude::*;
    use plotlib::{
        repr::{Histogram, HistogramBins},
        view::{ContinuousView, View},
    };
    use rand::prelude::*;

    #[test]
    fn unif_dist() {
        let mut rng = StdRng::seed_from_u64(123123123);
        let base = BoundedInt::new_with_bounds(0, 9, 0);
        let repeats = 10000;
        let (width, height, bins) = (10 + 5 * 20, 30, 20);

        let values: Vec<_> = (0..repeats)
            .into_iter()
            .map(|_| base.random(&mut rng).get_value() as f64)
            .collect();
        let hist = Histogram::from_slice(&values, HistogramBins::Count(bins));
        let view = ContinuousView::new().add(hist).x_range(-11.0, 11.0);
        let res = view.to_text(width, height).unwrap();
        println!("\n### BASE ###\n{}", res);

        let mut rng = StdRng::seed_from_u64(123123123);
        let base = BoundedInt::new_with_bounds(0, 10, 0);
        let values: Vec<_> = (0..repeats)
            .into_iter()
            .map(|_| {
                base.random(&mut rng)
                    .difference(&base.random(&mut rng))
                    .get_value() as f64
            })
            .collect();
        let hist = Histogram::from_slice(&values, HistogramBins::Count(bins));
        let view = ContinuousView::new().add(hist).x_range(-11.0, 11.0);
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
        let view = ContinuousView::new().add(hist).x_range(-11.0, 11.0);
        let res = view.to_text(width, height).unwrap();
        println!("\n### ADD ###\n{}", res);

        let mut rng = StdRng::seed_from_u64(123123123);
        let values: Vec<_> = (0..repeats)
            .into_iter()
            .map(|_| base.random(&mut rng).opposite(None).get_value() as f64)
            .collect();
        let hist = Histogram::from_slice(&values, HistogramBins::Count(bins));
        let view = ContinuousView::new().add(hist).x_range(-11.0, 11.0);
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
        let view = ContinuousView::new().add(hist).x_range(-11.0, 11.0);
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
        let view = ContinuousView::new().add(hist).x_range(-11.0, 11.0);
        let res = view.to_text(width, height).unwrap();
        println!("\n### ADDOPPDIFF ###\n{}", res);

        //assert!(false);
    }
}
