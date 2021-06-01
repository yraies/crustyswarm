use std::{fmt::Debug, iter::FromIterator};

use crate::traits::Differentiable;
use rand::{distributions::Uniform, Rng};
use serde::{Deserialize, Serialize};

impl<T: Differentiable> Differentiable for Vec<T> {
    fn add(&self, other: &Self) -> Self {
        assert_eq!(self.len(), other.len());
        self.iter()
            .zip(other.iter())
            .map(|(s, o)| s.add(o))
            .collect()
    }

    fn difference(&self, other: &Self) -> Self {
        assert_eq!(self.len(), other.len());
        self.iter()
            .zip(other.iter())
            .map(|(s, o)| s.difference(o))
            .collect()
    }

    fn scale(&self, factor: f32) -> Self {
        self.iter().map(|s| s.scale(factor)).collect()
    }

    fn opposite(&self) -> Self {
        self.iter().map(|s| s.opposite()).collect()
    }

    fn clon(&self) -> Self {
        self.iter().map(|s| s.clon()).collect()
    }

    fn random(&self, rng: &mut impl Rng) -> Self {
        self.iter().map(|s| s.random(rng)).collect()
    }

    fn apply_bounds(&self, other: &Self) -> Self {
        assert_eq!(self.len(), other.len());
        self.iter()
            .zip(other.iter())
            .map(|(s, o)| s.apply_bounds(o))
            .collect()
    }
}

impl<T: Differentiable, U: Differentiable> Differentiable for (T, U) {
    fn add(&self, other: &Self) -> Self {
        (self.0.add(&other.0), self.1.add(&other.1))
    }

    fn difference(&self, other: &Self) -> Self {
        (self.0.difference(&other.0), self.1.difference(&other.1))
    }

    fn scale(&self, factor: f32) -> Self {
        (self.0.scale(factor), self.1.scale(factor))
    }

    fn opposite(&self) -> Self {
        (self.0.opposite(), self.1.opposite())
    }

    fn clon(&self) -> Self {
        (self.0.clon(), self.1.clon())
    }

    fn random(&self, rng: &mut impl Rng) -> Self {
        (self.0.random(rng), self.1.random(rng))
    }

    fn apply_bounds(&self, other: &Self) -> Self {
        (self.0.apply_bounds(&other.0), self.1.apply_bounds(&other.1))
    }
}

impl<T: Differentiable, U: Differentiable, V: Differentiable> Differentiable for (T, U, V) {
    fn add(&self, other: &Self) -> Self {
        (
            self.0.add(&other.0),
            self.1.add(&other.1),
            self.2.add(&other.2),
        )
    }

    fn difference(&self, other: &Self) -> Self {
        (
            self.0.difference(&other.0),
            self.1.difference(&other.1),
            self.2.difference(&other.2),
        )
    }

    fn scale(&self, factor: f32) -> Self {
        (
            self.0.scale(factor),
            self.1.scale(factor),
            self.2.scale(factor),
        )
    }

    fn opposite(&self) -> Self {
        (self.0.opposite(), self.1.opposite(), self.2.opposite())
    }

    fn clon(&self) -> Self {
        (self.0.clon(), self.1.clon(), self.2.clon())
    }

    fn random(&self, rng: &mut impl Rng) -> Self {
        (self.0.random(rng), self.1.random(rng), self.2.random(rng))
    }

    fn apply_bounds(&self, other: &Self) -> Self {
        (
            self.0.apply_bounds(&other.0),
            self.1.apply_bounds(&other.1),
            self.2.apply_bounds(&other.2),
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct Fixed<T: Clone + PartialEq>(T);

impl<T: Clone + PartialEq> Differentiable for Fixed<T> {
    fn add(&self, _other: &Self) -> Self {
        self.clon()
    }

    fn difference(&self, _other: &Self) -> Self {
        self.clon()
    }

    fn scale(&self, _factor: f32) -> Self {
        self.clon()
    }

    fn opposite(&self) -> Self {
        self.clon()
    }

    fn clon(&self) -> Self {
        self.clone()
    }

    fn random(&self, _rng: &mut impl Rng) -> Self {
        self.clon()
    }

    fn apply_bounds(&self, other: &Self) -> Self {
        other.clon()
    }
}

impl<T: Clone + PartialEq> std::ops::Deref for Fixed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Clone + PartialEq> AsRef<T> for Fixed<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: Clone + PartialEq> From<T> for Fixed<T> {
    fn from(other: T) -> Self {
        Fixed(other)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct FloatyBool(f32);

impl std::ops::Deref for FloatyBool {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<FloatyBool> for bool {
    fn from(other: FloatyBool) -> Self {
        other.0 >= 0.5
    }
}

impl From<f32> for FloatyBool {
    fn from(other: f32) -> Self {
        assert!(other <= 1.0);
        assert!(other >= 0.0);
        FloatyBool(other)
    }
}

impl From<bool> for FloatyBool {
    fn from(other: bool) -> Self {
        FloatyBool(if other { 1.0 } else { 0.0 })
    }
}

impl Differentiable for FloatyBool {
    fn add(&self, other: &Self) -> Self {
        let temp_res = self.0 + other.0;
        FloatyBool(if temp_res > 1.0 {
            2.0 - temp_res
        } else {
            temp_res
        })
    }

    fn difference(&self, other: &Self) -> Self {
        FloatyBool((self.0 - other.0).abs())
    }

    fn scale(&self, factor: f32) -> Self {
        FloatyBool(self.0 * factor)
    }

    fn opposite(&self) -> Self {
        FloatyBool(1.0 - self.0)
    }

    fn clon(&self) -> Self {
        self.clone()
    }

    fn random(&self, rng: &mut impl Rng) -> Self {
        FloatyBool(rng.sample(Uniform::new_inclusive(0.0, 1.0)))
    }

    fn apply_bounds(&self, other: &Self) -> Self {
        other.clon()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct BoundedFactor {
    pub base: f32,
    pub range: f32,
    pub val: f32,
}

impl BoundedFactor {
    pub fn new(lower: f32, upper: f32, value: f32) -> BoundedFactor {
        BoundedFactor {
            base: lower,
            range: upper - lower,
            val: value - lower,
        }
    }

    pub fn new_from_f32(value: f32) -> BoundedFactor {
        BoundedFactor {
            base: value,
            range: 0.0,
            val: 0.0,
        }
    }

    pub fn get_value(&self) -> f32 {
        self.base + self.val
    }
}

impl Differentiable for BoundedFactor {
    fn add(&self, other: &Self) -> Self {
        BoundedFactor {
            base: self.base,
            range: self.range,
            val: {
                let sum = self.val + other.val; // 0 <= sum <= 2 self.range
                if sum > self.range {
                    2f32 * self.range - sum
                } else {
                    sum
                }
            },
        }
    }

    fn difference(&self, other: &Self) -> Self {
        BoundedFactor {
            base: self.base,
            range: self.range,
            val: (self.val - other.val).abs(), // - self.range <= diff <= self.range
        }
    }

    fn opposite(&self) -> Self {
        BoundedFactor {
            base: self.base,
            range: self.range,
            val: self.range - self.val,
        }
    }

    fn scale(&self, factor: f32) -> Self {
        BoundedFactor {
            base: self.base,
            range: self.range,
            val: self.val * factor, //TODO: Handle factor > 1.0
        }
    }

    fn clon(&self) -> Self {
        self.clone()
    }

    fn random(&self, rng: &mut impl Rng) -> Self {
        let mut copy = self.clone();
        copy.val = rng.sample(Uniform::new_inclusive(0.0, self.range));
        copy
    }

    fn apply_bounds(&self, other: &Self) -> Self {
        BoundedFactor {
            base: self.base,
            range: self.range,
            val: (other.get_value() - self.base).clamp(0.0, self.range),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct BoolCell<T> {
    pub active: FloatyBool,
    pub value: T,
}

impl<T> BoolCell<T> {
    pub fn is_active(&self) -> bool {
        self.active.clone().into()
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
    fn opposite(&self, index_count: usize) -> Self {
        Self {
            active: self.active.opposite(),
            value: index_count - self.value,
        }
    }

    pub fn random(&self, rng: &mut impl Rng, lower_bound: usize, upper_bound: usize) -> Self {
        BoolCell {
            active: rng.sample(Uniform::new_inclusive(0.0, 1.0)).into(),
            value: rng.gen_range(lower_bound, upper_bound + 1),
        }
    }
}

impl BoolCell<f32> {
    pub fn new() -> Self {
        BoolCell {
            active: 0.0.into(),
            value: 0f32,
        }
    }

    fn add(&self, other: &Self, lower_bound: f32, upper_bound: f32) -> Self {
        Self {
            active: self.active.add(&other.active),
            value: BoundedFactor::new(lower_bound, upper_bound, self.value)
                .add(&BoundedFactor::new(lower_bound, upper_bound, other.value))
                .get_value(),
        }
    }

    fn diff(&self, other: &Self, lower_bound: f32, upper_bound: f32) -> Self {
        Self {
            active: self.active.difference(&other.active),
            value: BoundedFactor::new(lower_bound, upper_bound, self.value)
                .difference(&BoundedFactor::new(lower_bound, upper_bound, other.value))
                .get_value(),
        }
    }

    fn scale(&self, factor: f32, lower_bound: f32, upper_bound: f32) -> Self {
        Self {
            active: self.active.scale(factor), //TODO: deal with factors > 1.0
            value: BoundedFactor::new(lower_bound, upper_bound, self.value)
                .scale(factor)
                .get_value(),
        }
    }
    fn opposite(&self, lower_bound: f32, upper_bound: f32) -> Self {
        Self {
            active: self.active.opposite(),
            value: BoundedFactor::new(lower_bound, upper_bound, self.value)
                .opposite()
                .get_value(),
        }
    }

    pub fn random(&self, rng: &mut impl Rng, lower_bound: f32, upper_bound: f32) -> Self {
        BoolCell {
            active: rng.sample(Uniform::new_inclusive(0.0, 1.0)).into(),
            value: BoundedFactor::new(lower_bound, upper_bound, 0.0)
                .random(rng)
                .get_value(),
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
                active: (if i.0 { 1.0 } else { 0.0 }).into(),
                value: i.1,
            });
        }

        c
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct BoundedFactorVec {
    pub vec: Vec<BoolCell<f32>>,
    pub upper_bound: f32,
    pub lower_bound: f32,
}

impl BoundedFactorVec {
    pub fn new(lower_bound: f32, upper_bound: f32, size: usize) -> BoundedFactorVec {
        BoundedFactorVec {
            vec: vec![BoolCell::<f32>::new(); size],
            upper_bound,
            lower_bound,
        }
    }

    pub fn into_f32_vec(&self) -> Vec<f32> {
        self.vec
            .iter()
            .map(|bar| if bar.is_active() { bar.value } else { 0.0 })
            .collect()
    }

    pub fn fill_to(&mut self, size: usize) {
        while self.vec.len() < size {
            self.vec.push(BoolCell::<f32>::new())
        }
    }
}

impl FromIterator<(bool, f32)> for BoundedFactorVec {
    fn from_iter<I: IntoIterator<Item = (bool, f32)>>(iter: I) -> Self {
        let mut c = BoundedFactorVec {
            vec: vec![],
            upper_bound: f32::MIN,
            lower_bound: f32::MAX,
        };

        for i in iter {
            c.lower_bound = c.lower_bound.min(i.1);
            c.upper_bound = c.upper_bound.max(i.1);
            c.vec.push(BoolCell {
                active: (if i.0 { 1.0 } else { 0.0 }).into(),
                value: i.1,
            });
        }

        c
    }
}

impl Differentiable for BoundedIdxVec {
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

    fn scale(&self, factor: f32) -> Self {
        BoundedIdxVec {
            vec: self.vec.iter().map(|cell| cell.scale(factor)).collect(),
            upper_bound: self.upper_bound,
        }
    }

    fn opposite(&self) -> Self {
        BoundedIdxVec {
            vec: self
                .vec
                .iter()
                .map(|cell| cell.opposite(self.upper_bound))
                .collect(),
            upper_bound: self.upper_bound,
        }
    }

    fn clon(&self) -> Self {
        self.clone()
    }

    fn random(&self, rng: &mut impl Rng) -> Self {
        let mut copy = self.clone();
        copy.vec = copy
            .vec
            .iter()
            .map(|cell| cell.random(rng, 0, self.upper_bound))
            .collect();
        copy
    }

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

impl Differentiable for BoundedFactorVec {
    fn add(&self, other: &Self) -> Self {
        BoundedFactorVec {
            vec: self
                .vec
                .iter()
                .zip(&other.vec)
                .map(|(first, second)| first.add(second, self.lower_bound, self.upper_bound))
                .collect(),
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }

    fn difference(&self, other: &Self) -> Self {
        BoundedFactorVec {
            vec: self
                .vec
                .iter()
                .zip(&other.vec)
                .map(|(first, second)| first.diff(second, self.lower_bound, self.upper_bound))
                .collect(),
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }

    fn scale(&self, factor: f32) -> Self {
        BoundedFactorVec {
            vec: self
                .vec
                .iter()
                .map(|cell| cell.scale(factor, self.lower_bound, self.upper_bound))
                .collect(),
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }

    fn opposite(&self) -> Self {
        BoundedFactorVec {
            vec: self
                .vec
                .iter()
                .map(|cell| cell.opposite(self.lower_bound, self.upper_bound))
                .collect(),
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }

    fn clon(&self) -> Self {
        self.clone()
    }

    fn random(&self, rng: &mut impl Rng) -> Self {
        let mut copy = self.clone();
        copy.vec = copy
            .vec
            .iter()
            .map(|cell| cell.random(rng, self.lower_bound, self.upper_bound))
            .collect();
        copy
    }

    fn apply_bounds(&self, other: &Self) -> Self {
        BoundedFactorVec {
            vec: other.vec.clone(),
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }
}

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
            let factor = BoundedFactor::new(vals[0], vals[2], vals[1]);
            test(factor);
        }
    }

    fn test_bounded_factors(count: usize, test: fn(BoundedFactor, BoundedFactor)) {
        let mut rng = StdRng::seed_from_u64(1_234_567_890);
        let uni = Uniform::new_inclusive(-10.0, 10.0);

        let factor = BoundedFactor {
            base: -9.0,
            range: 6.0,
            val: 3.75,
        };
        let factor2 = BoundedFactor {
            base: -9.0,
            range: 6.0,
            val: 2.0,
        };
        test(factor, factor2);

        for _i in 0..count {
            let mut vals: Vec<f32> = vec![
                rng.sample(&uni),
                rng.sample(&uni),
                rng.sample(&uni),
                rng.sample(&uni),
            ];
            vals.sort_by(|o1, o2| o1.partial_cmp(o2).unwrap());
            let factor = BoundedFactor::new(vals[0], vals[3], vals[1]);
            let factor2 = BoundedFactor::new(vals[0], vals[3], vals[2]);
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
        let factor = BoundedFactor::new(0.0, 4.0, 2.0);
        let factor2 = BoundedFactor::new(0.0, 4.0, 3.0);
        assert_eq!(BoundedFactor::new(0.0, 4.0, 3.0), factor.add(&factor2));

        let factor = BoundedFactor::new(10.0, 20.0, 19.0);
        let factor2 = BoundedFactor::new(10.0, 20.0, 19.0);
        assert_eq!(BoundedFactor::new(10.0, 20.0, 12.0), factor.add(&factor2));
    }

    #[test]
    fn basic_difference() {
        let factor = BoundedFactor::new(-10.0, 10.0, 5.0);
        let factor2 = BoundedFactor::new(-10.0, 10.0, 5.0);
        assert_eq!(
            BoundedFactor::new(-10.0, 10.0, -10.0),
            factor.difference(&factor2)
        );

        let factor = BoundedFactor::new(-10.0, 10.0, 10.0);
        let factor2 = BoundedFactor::new(-10.0, 10.0, -10.0);
        assert_eq!(
            BoundedFactor::new(-10.0, 10.0, 10.0),
            factor.difference(&factor2)
        );

        let factor = BoundedFactor::new(-10.0, 10.0, 5.0);
        let factor2 = BoundedFactor::new(-10.0, 10.0, -7.0);
        assert_eq!(
            BoundedFactor::new(-10.0, 10.0, 2.0),
            factor.difference(&factor2)
        );
    }

    #[test]
    fn fuzz_opposite_idempotent() {
        test_bounded_factor(1000, |factor| {
            assert!(
                factor.get_value() - factor.opposite().opposite().get_value() <= 0.000001,
                "Difference: {:?}",
                factor.get_value() - factor.opposite().opposite().get_value()
            );
        });
    }

    #[test]
    fn fuzz_add_opposite_eq_upper() {
        test_bounded_factor(1000, |factor| {
            let testfac = factor.add(&factor.opposite());
            assert!(
                testfac.get_value() - testfac.base - testfac.range <= 0.000001,
                "Difference: {:?}",
                testfac.get_value() - testfac.base - testfac.range
            );
        });
    }

    #[test]
    fn fuzz_sum_of_diff_and_diff_opposite_idempotent() {
        test_bounded_factors(1000, |factor1, factor2| {
            let diff = factor1.difference(&factor2);
            let testfac = factor1.add(&diff).add(&diff.opposite()).opposite();
            assert!(
                factor1.val - factor1.range - testfac.val <= 0.00001,
                "\nF1: {:?}\nF2: {:?}\nDiff: {:?}\nDiff.opp: {:?}\nF1.add(Diff): {:?}\nF1.add(Diff).add(Diff.opp): {:?}",
                factor1,
                factor2,
                diff.val,
                diff.opposite().val,
                factor1.add(&diff).val,
                testfac.val
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

                let o1 = v1.opposite();
                println!("o1 : {} {} ", o1, f1(&o1));
                assert!(!f1(&o1));

                let o2 = v2.opposite();
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
