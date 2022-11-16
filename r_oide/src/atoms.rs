pub mod bool;
pub mod bool_cell;
pub mod bounded_float;
pub mod bounded_float_vec;
pub mod bounded_int;
pub mod bounded_int_vec;
pub mod fixed;
pub mod multiset;
pub mod tuples;
pub mod vec;

pub use crate::atoms::bool::*;
pub use crate::atoms::bool_cell::*;
pub use crate::atoms::bounded_float::*;
pub use crate::atoms::bounded_float_vec::*;
pub use crate::atoms::bounded_int::*;
pub use crate::atoms::bounded_int_vec::*;
pub use crate::atoms::fixed::*;
pub use crate::atoms::tuples::*;
pub use crate::atoms::vec::*;

struct Util {}
impl Util {
    fn crossover<T: Clone>(this: &T, other: &T, rng: &mut impl rand::Rng, rate: f64) -> T {
        if rng.gen_bool(rate) {
            other.clone()
        } else {
            this.clone()
        }
    }
}

#[cfg(test)]
mod testbounded_factors {
    use crate::prelude::*;
    use rand::{distributions::Uniform, prelude::*, SeedableRng};

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
