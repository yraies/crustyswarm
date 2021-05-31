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
        fn eval(&self, params: &Self::Params) -> E;
    }

    pub trait IODEPopulation<B: Evaluatable<E, Params = P>, E, P> {
        fn get_size(&self) -> usize;
        fn get_population(&self) -> Vec<&B>;
        fn step(
            &self,
            selection: &mut dyn FnMut(((B, E), (B, E), (B, E)), usize) -> Vec<B>,
            rng: &mut impl Rng,
            params: P,
        ) -> Vec<B>;
    }

    impl<B, E, P> IODEPopulation<B, E, P> for Vec<B>
    where
        B: Evaluatable<E, Params = P> + PartialEq + Debug,
    {
        fn get_size(&self) -> usize {
            self.len()
        }

        fn get_population(&self) -> Vec<&B> {
            self.iter().collect::<Vec<&B>>()
        }

        fn step(
            &self,
            selection: &mut dyn FnMut(((B, E), (B, E), (B, E)), usize) -> Vec<B>,
            rng: &mut impl Rng,
            params: P,
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
                    let trial = target.add(&(other1.difference(other2).scale(0.5)));
                    let target_opposite =
                        target.add(&(other1.difference(other2).scale(0.5).opposite()));
                    (target.clon(), trial, target_opposite)
                })
                .collect::<Vec<_>>();

            let evaled_pairs = variants
                .iter()
                //.inspect(|f| println!("{:?}, {:?}, {:?}", f.0, f.1, f.2))
                .map(|(target, trial, opposite)| {
                    (
                        (target.clon(), target.eval(&params)),
                        (trial.clon(), trial.eval(&params)),
                        (opposite.clon(), opposite.eval(&params)),
                    )
                });

            evaled_pairs
                .into_iter()
                .enumerate()
                .flat_map(|(num, pair)| selection(pair, num))
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
        fn eval(&self, _params: &Self::Params) -> f32 {
            *self
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
                &mut |t: ((f32, f32), (f32, f32), (f32, f32)), _: usize| {
                    let (t1, t2, t3) = t;
                    let d1 = (t1.1 + 20.0).abs();
                    let d2 = (t2.1 + 20.0).abs();
                    let d3 = (t3.1 + 20.0).abs();

                    if d1 < d2 && d1 < d3 {
                        vec![t1.1]
                    } else if d2 < d3 {
                        vec![t2.1]
                    } else {
                        vec![t3.1]
                    }
                },
                &mut rng,
                (),
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
