pub mod atoms;

pub mod prelude {
    pub use crate::traits::{
        Differentiable, Evaluatable, IODEPopulation, OIDEAdd, OIDEBoundApplication, OIDEDiff,
        OIDEOpposite, OIDERandomize, OIDEScale,
    };

    pub use crate::atoms::{
        bool::*, bounded_float::*, bounded_int::*, fixed::*, multiset::*, tuples::*, *,
    };
}

pub mod traits {
    use rand::prelude::*;
    use std::fmt::Debug;

    pub trait Differentiable
    where
        Self: OIDEAdd
            + OIDEDiff
            + OIDEScale
            + OIDEOpposite
            + OIDERandomize
            + OIDEBoundApplication
            + Clone,
    {
        fn trial_plus_from(&self, parent1: &Self, parent2: &Self, factor: f32) -> Self {
            self.add(&parent1.difference(parent2).scale(factor))
        }
        fn trial_minus_from(&self, parent1: &Self, parent2: &Self, factor: f32) -> Self {
            self.add(&parent1.difference(parent2).scale(factor).opposite())
        }
    }

    pub trait OIDEAdd {
        fn add(&self, other: &Self) -> Self;
    }
    pub trait OIDEDiff {
        fn difference(&self, other: &Self) -> Self;
    }
    pub trait OIDEScale {
        fn scale(&self, factor: f32) -> Self;
    }
    pub trait OIDEOpposite {
        fn opposite(&self) -> Self;
    }
    pub trait OIDERandomize {
        fn random(&self, rng: &mut impl Rng) -> Self;
    }
    pub trait OIDEBoundApplication {
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
                .map(|&target| {
                    let parent1 = self
                        .iter()
                        .filter(|&curr| curr.ne(target))
                        .choose(rng)
                        .expect("No other individuals could be found!");
                    let parent2 = self
                        .iter()
                        .filter(|&curr| curr.ne(target) && curr.ne(parent1))
                        .choose(rng)
                        .expect("No other individuals could be found!");
                    let trial = target.trial_plus_from(parent1, parent2, f);
                    let trial_opposite = target.trial_minus_from(parent1, parent2, f);
                    [
                        target.clone(),
                        trial,
                        trial_opposite,
                        target.clone().opposite(),
                    ]
                })
                .collect::<Vec<_>>();

            let evaled_pairs: Vec<_> = variants
                .iter()
                .enumerate()
                .map(|(idx, set)| {
                    println!("#############################\nEvaluating triple #{:3} of {:3}\n#############################", idx + 1, self.len());
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    set.into_iter()
                        .map(|base| {
                            let (eval, info) = base.eval(&params);
                            (base.clone(), eval, info)
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
    use crate::prelude::*;

    impl OIDEAdd for f32 {
        fn add(&self, other: &Self) -> Self {
            self + other
        }
    }
    impl OIDEDiff for f32 {
        fn difference(&self, other: &Self) -> Self {
            other - self
        }
    }
    impl OIDEScale for f32 {
        fn scale(&self, factor: f32) -> Self {
            self * factor
        }
    }
    impl OIDEOpposite for f32 {
        fn opposite(&self) -> Self {
            -self
        }
    }
    impl OIDERandomize for f32 {
        fn random(&self, rng: &mut impl rand::Rng) -> Self {
            rng.gen()
        }
    }
    impl OIDEBoundApplication for f32 {
        fn apply_bounds(&self, other: &Self) -> Self {
            *other
        }
    }

    impl Differentiable for f32 {}

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
        //let mut pop = rand::Rng::sample_iter(&mut rng, &rand::distributions::Standard)
        //    .take(10)
        //    .map(|f: f32| f * 100.0)
        //    .collect::<Vec<f32>>();
        let mut pop: Vec<_> = (0..10).map(|v| v as f32).collect();
        crate::traits::IODEPopulation::get_population(&pop);
        let mut lastbest = f32::MAX;
        for i in 0..15 {
            println!("Iteration {i}", i = i);
            println!("Input : {:?}", pop);
            pop = pop.step(
                &mut |t: &[(f32, f32, ())], _num: usize| {
                    vec![
                        t.iter()
                            .map(|c| (c, (c.1 + 20.1111211).abs()))
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
        assert!(false)
    }
}
