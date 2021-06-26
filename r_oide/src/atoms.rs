use std::{fmt::Debug, iter::FromIterator};

use crate::traits::*;
use rand::{distributions::Uniform, Rng};
use serde::{Deserialize, Serialize};

pub mod bool;
pub mod bounded_float;
pub mod bounded_int;
pub mod fixed;
pub mod multiset;
pub mod tuples;
pub use crate::atoms::bool::*;
pub use crate::atoms::bounded_float::*;
pub use crate::atoms::bounded_int::*;
pub use crate::atoms::fixed::*;
pub use crate::atoms::tuples::*;

struct Util {}
impl Util {
    fn crossover<T: Clone>(this: &T, other: &T, rng: &mut impl Rng, rate: f64) -> T {
        if rng.gen_bool(rate) {
            other.clone()
        } else {
            this.clone()
        }
    }
}

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
                    .map(|m| m.into_iter().map(|v| Some(v)).collect())
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
impl<T: Differentiable + Debug> Differentiable for Vec<T> {}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct BoolCell<T> {
    pub active: bool::FloatyBool,
    pub value: T,
}

impl<T> BoolCell<T> {
    pub fn is_active(&self) -> bool {
        self.active.clone().into()
    }
}
impl<T: OIDEZero> BoolCell<T> {
    fn zero(&self) -> Self {
        BoolCell {
            active: self.active.zero(),
            value: self.value.zero(),
        }
    }
}

impl BoolCell<usize> {
    pub fn new() -> Self {
        BoolCell {
            active: 0.0.into(),
            value: 0usize,
        }
    }

    fn add(&self, other: &Self, index_count: usize) -> Self {
        Self {
            active: self.active.add(&other.active),
            value: (other.value + self.value) % (index_count + 1),
        }
    }

    fn diff(&self, other: &Self) -> Self {
        Self {
            active: self.active.difference(&other.active),
            value: (self.value as i64 - other.value as i64).abs() as usize,
        }
    }

    fn scale(&self, factor: f32) -> Self {
        Self {
            active: self.active.scale(factor), //TODO: deal with factors > 1.0
            value: (self.value as f32 * factor).round() as usize,
        }
    }
    fn opposite(&self, index_count: usize, midpoint: Option<&BoolCell<usize>>) -> Self {
        Self {
            active: self.active.opposite(match midpoint {
                Some(ref m) => Some(&m.active),
                None => None,
            }),
            value: index_count - self.value,
        }
    }
    pub fn random(&self, rng: &mut impl Rng, lower_bound: usize, upper_bound: usize) -> Self {
        BoolCell {
            active: rng.sample(Uniform::new_inclusive(0.0, 1.0)).into(),
            value: rng.gen_range(lower_bound, upper_bound + 1),
        }
    }
    fn zero(&self) -> Self {
        BoolCell {
            active: self.active.zero(),
            value: 0,
        }
    }
}

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
                        .map(|m| m.vec.iter().map(|v| Some(v)).collect())
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
impl Differentiable for BoundedIdxVec {}

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
                        .map(|m| m.vec.iter().map(|v| Some(v)).collect())
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
impl Differentiable for BoundedFactorVec {}

#[cfg(test)]
mod testbounded_factors {
    use super::*;
    use rand::{prelude::*, SeedableRng};

    fn test_bounded_factor(count: usize, test: fn(BoundedFactor)) {
        let mut rng = StdRng::seed_from_u64(1_234_567_890);
        let uni = Uniform::new_inclusive(-10.0, 10.0);

        for _ in 0..count {
            let mut vals: Vec<f32> = vec![rng.sample(&uni), rng.sample(&uni), rng.sample(&uni)];
            vals.sort_by(|o1, o2| o1.partial_cmp(o2).unwrap());
            let factor = BoundedFactor::new_with_bounds(vals[0], vals[2], vals[1]);
            test(factor);
        }
    }

    fn test_bounded_factors(count: usize, test: fn(BoundedFactor, BoundedFactor)) {
        let mut rng = StdRng::seed_from_u64(1_234_567_890);
        let uni = Uniform::new_inclusive(-10.0, 10.0);

        let factor = BoundedFactor::new_with_base(-9.0, 6.0, 3.75);
        let factor2 = BoundedFactor::new_with_base(-9.0, 6.0, 2.0);
        test(factor, factor2);

        for _i in 0..count {
            let mut vals: Vec<f32> = vec![
                rng.sample(&uni),
                rng.sample(&uni),
                rng.sample(&uni),
                rng.sample(&uni),
            ];
            vals.sort_by(|o1, o2| o1.partial_cmp(o2).unwrap());
            let factor = BoundedFactor::new_with_bounds(vals[0], vals[3], vals[1]);
            let factor2 = BoundedFactor::new_with_bounds(vals[0], vals[3], vals[2]);
            if rng.gen() {
                dbg!(_i);
                dbg!(&factor);
                dbg!(&factor2);
                test(factor, factor2);
            } else {
                dbg!(_i);
                dbg!(&factor2);
                dbg!(&factor);
                test(factor2, factor);
            }
        }
    }

    #[test]
    fn basic_addition() {
        let factor = BoundedFactor::new_with_bounds(0.0, 4.0, 2.0);
        let factor2 = BoundedFactor::new_with_bounds(0.0, 4.0, 3.0);
        assert_eq!(
            BoundedFactor::new_with_bounds(0.0, 4.0, 3.0).get_value(),
            factor.add(&factor2).get_value()
        );

        let factor = BoundedFactor::new_with_bounds(10.0, 20.0, 19.0);
        let factor2 = BoundedFactor::new_with_bounds(10.0, 20.0, 19.0);
        assert_eq!(
            BoundedFactor::new_with_bounds(10.0, 20.0, 12.0).get_value(),
            factor.add(&factor2).get_value()
        );
    }

    #[test]
    fn basic_difference() {
        let factor = BoundedFactor::new_with_bounds(-10.0, 10.0, 5.0);
        let factor2 = BoundedFactor::new_with_bounds(-10.0, 10.0, 5.0);
        assert_eq!(
            BoundedFactor::new_with_bounds(-10.0, 10.0, -10.0),
            factor.difference(&factor2)
        );

        let factor = BoundedFactor::new_with_bounds(-10.0, 10.0, 10.0);
        let factor2 = BoundedFactor::new_with_bounds(-10.0, 10.0, -10.0);
        assert_eq!(
            BoundedFactor::new_with_bounds(-10.0, 10.0, 10.0),
            factor.difference(&factor2)
        );

        let factor = BoundedFactor::new_with_bounds(-10.0, 10.0, 5.0);
        let factor2 = BoundedFactor::new_with_bounds(-10.0, 10.0, -7.0);
        assert_eq!(
            BoundedFactor::new_with_bounds(-10.0, 10.0, 2.0),
            factor.difference(&factor2)
        );
    }

    #[test]
    fn fuzz_opposite_idempotent() {
        test_bounded_factor(1000, |factor| {
            assert!(
                factor.get_value() - factor.opposite(None).opposite(None).get_value() <= 0.000001,
                "Difference: {:?}",
                factor.get_value() - factor.opposite(None).opposite(None).get_value()
            );
        });
    }

    #[test]
    fn fuzz_add_opposite_eq_upper() {
        test_bounded_factor(1000, |factor| {
            let testfac = factor.add(&factor.opposite(None));
            assert!(
                testfac.get_value() - testfac.get_lower_bound() - testfac.get_range() <= 0.000001,
                "Difference: {:?}",
                testfac.get_value() - testfac.get_lower_bound() - testfac.get_range()
            );
        });
    }

    #[test]
    fn fuzz_sum_of_diff_and_diff_opposite_idempotent() {
        test_bounded_factors(1000, |factor1, factor2| {
            let diff = factor1.difference(&factor2);
            let testfac = factor1.add(&diff).add(&diff.opposite(None)).opposite(None);
            assert!(
                factor1.get_offset() - factor1.get_range() - testfac.get_offset() <= 0.00001,
                "\nF1: {:?}\nF2: {:?}\nDiff: {:?}\nDiff.opp: {:?}\nF1.add(Diff): {:?}\nF1.add(Diff).add(Diff.opp): {:?}",
                factor1,
                factor2,
                diff.get_offset(),
                diff.opposite(None).get_offset(),
                factor1.add(&diff).get_offset(),
                testfac.get_offset()
            ); // kaputt, weil a+(a-b)-(a-b) != a+((a-b)-(a-b))
        });
    }

    #[test]
    fn fuzz_diff_correctly_uncomutative() {
        test_bounded_factors(1000, |factor1, factor2| {
            let diff = factor1.difference(&factor2);
            let diff2 = factor2.difference(&factor1);
            let testfac = factor1.add(&diff).add(&diff2);
            assert!(
                factor1.get_value() - testfac.get_value() <= 0.00001,
                "Difference: {:?}",
                factor1.get_value() - testfac.get_value()
            );
        });
    }
}

#[cfg(test)]
mod testidxvec {
    use super::*;
    use rand::{prelude::*, SeedableRng};
    use std::fmt::{Display, Write};

    impl Display for BoundedIdxVec {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_char('[')?;
            f.write_str(
                &self
                    .vec
                    .iter()
                    .map(|cell| {
                        return format!(
                            "[{} {:2}]",
                            if cell.is_active() { "O" } else { "X" },
                            cell.value
                        );
                    })
                    .fold(Vec::<String>::new(), |mut acc, elem| {
                        acc.push(elem);
                        acc
                    })
                    .join(", "),
            )?;
            f.write_char(']')?;
            Ok(())
        }
    }

    #[test]
    fn add() {
        let mut rng = StdRng::seed_from_u64(1237919273);
        let total_size = 20;
        for total_count in 1..20 {
            let v1 = BoundedIdxVec::new_by_idx_count(total_count, total_size);
            for _i in 0..200 {
                let v1 = v1.random(&mut rng);
                let v2 = v1.random(&mut rng);
                let f1 = |v: &BoundedIdxVec| v.vec.iter().any(|cell| cell.value >= total_count);

                println!("v1  : {} {}", &v1, f1(&v1));
                assert!(!f1(&v1));

                println!("v2  : {} {}", &v2, f1(&v2));
                assert!(!f1(&v2));

                let o1 = v1.opposite(None);
                println!("o1 : {} {} ", o1, f1(&o1));
                assert!(!f1(&o1));

                let o2 = v2.opposite(None);
                println!("o2 : {} {} ", o2, f1(&o2));
                assert!(!f1(&o2));

                let s1 = v1.scale(0.5);
                println!("s1 : {} {} ", s1, f1(&s1));
                assert!(!f1(&s1));

                let s2 = v2.scale(0.5);
                println!("s2 : {} {} ", s2, f1(&s2));
                assert!(!f1(&s2));

                let v3 = v1.add(&v2);
                println!("add : {} {}", v3, f1(&v3));
                assert!(!f1(&v3));

                let v4 = v1.difference(&v3);
                println!("diff: {} {}", v4, f1(&v4));
                assert!(!f1(&v4));

                println!();
            }
        }
    }
}
