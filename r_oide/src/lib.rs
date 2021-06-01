pub mod atoms;

pub mod traits {
    use rand::prelude::*;
    use std::fmt::Debug;

    pub trait Differentiable {
        fn add(&self, other: &Self) -> Self;
        fn difference(&self, other: &Self) -> Self;
        fn scale(&self, factor: f32) -> Self;
        fn opposite(&self) -> Self;
        fn clon(&self) -> Self;
        fn random(&self, rng: &mut impl Rng) -> Self;
        fn apply_bounds(&self, other: &Self) -> Self;
    }

    pub trait Evaluatable<E>: Differentiable {
        type Params;
        type EvalInfo;
        fn eval(&self, params: &Self::Params) -> (E, Self::EvalInfo);
    }

    pub trait IODEPopulation<B: Evaluatable<E, Params = P, EvalInfo = I>, E, P, I> {
        fn get_size(&self) -> usize;
        fn get_population(&self) -> Vec<&B>;
        fn step(
            &self,
            selection: &mut dyn FnMut(&[(B, E, I)], usize) -> Vec<B>,
            rng: &mut impl Rng,
            params: P,
            f: f32,
        ) -> Vec<B>;
    }

    impl<B, E, P, I> IODEPopulation<B, E, P, I> for Vec<B>
    where
        B: Evaluatable<E, Params = P, EvalInfo = I> + PartialEq + Debug,
    {
        fn get_size(&self) -> usize {
            self.len()
        }

        fn get_population(&self) -> Vec<&B> {
            self.iter().collect::<Vec<&B>>()
        }

        fn step(
            &self,
            selection: &mut dyn FnMut(&[(B, E, I)], usize) -> Vec<B>,
            rng: &mut impl Rng,
            params: P,
            f: f32,
        ) -> Vec<B> {
            let variants = self
                .get_population()
                .iter()
                .map(|target| {
                    let other1 = self
                        .iter()
                        .filter(|curr| target.ne(curr))
                        .choose(rng)
                        .expect("No other individuals could be found!");
                    let other2 = self
                        .iter()
                        .filter(|curr| target.ne(curr))
                        .choose(rng)
                        .expect("No other individuals could be found!");
                    let trial = target.add(&(other1.difference(other2).scale(f)));
                    let target_opposite =
                        target.add(&(other1.difference(other2).scale(f).opposite()));
                    [target.clon(), trial, target_opposite]
                })
                .collect::<Vec<_>>();

            let evaled_pairs: Vec<_> = variants
                .iter()
                .map(|set| {
                    set.into_iter()
                        .map(|base| {
                            let (eval, info) = base.eval(&params);
                            (base.clon(), eval, info)
                        })
                        .collect::<Vec<_>>()
                })
                .collect();

            evaled_pairs
                .into_iter()
                .enumerate()
                .flat_map(|(num, pair)| selection(&pair, num))
                .collect()
        }
    }
}

pub mod tests {
    #[allow(unused_imports)]
    use crate::traits::{Differentiable, Evaluatable, IODEPopulation};

    impl Differentiable for f32 {
        fn add(&self, other: &Self) -> Self {
            self + other
        }

        fn difference(&self, other: &Self) -> Self {
            other - self
        }

        fn scale(&self, factor: f32) -> Self {
            self * factor
        }

        fn opposite(&self) -> Self {
            -self
        }

        fn clon(&self) -> Self {
            *self
        }

        fn random(&self, rng: &mut impl rand::Rng) -> Self {
            rng.gen()
        }

        fn apply_bounds(&self, other: &Self) -> Self {
            *other
        }
    }

    impl Evaluatable<f32> for f32 {
        type Params = ();
        type EvalInfo = ();
        fn eval(&self, _params: &Self::Params) -> (f32, ()) {
            (*self, ())
        }
    }

    #[test]
    fn test_ode_20() {
        let mut rng = <rand::rngs::StdRng as rand::SeedableRng>::seed_from_u64(2345678912u64);
        rand::Rng::gen_bool(&mut rng, 0.5);
        let mut pop = rand::Rng::sample_iter(&mut rng, &rand::distributions::Standard)
            .take(10)
            .map(|f: f32| f * 100.0)
            .collect::<Vec<f32>>();
        crate::traits::IODEPopulation::get_population(&pop);
        let mut lastbest = f32::MAX;
        for i in 0..15 {
            println!("Iteration {i}", i = i);
            println!("Input : {:?}", pop);
            pop = pop.step(
                &mut |t: &[(f32, f32, ())], _num: usize| {
                    vec![
                        t.iter()
                            .map(|c| (c, (c.1 + 20.0).abs()))
                            .min_by(|(_, v1), (_, v2)| {
                                v1.partial_cmp(v2).unwrap_or(std::cmp::Ordering::Less)
                            })
                            .unwrap()
                            .0
                             .1,
                    ]
                },
                &mut rng,
                (),
                0.5,
            );
            println!("Result: {:?}", pop);
            let newbest = pop.iter().fold(f32::MAX, |acc, elem| {
                if acc < (elem + 20.0).abs() {
                    acc
                } else {
                    (elem + 20.0).abs()
                }
            });
            println!("Best Error : {:?}", newbest);
            println!("Improvement: {:?}", lastbest - newbest);
            lastbest = newbest;
            println!()
        }
    }
}
