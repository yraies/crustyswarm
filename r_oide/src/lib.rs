pub mod atoms;

pub mod prelude {
    pub use crate::traits::{
        Differentiable, Evaluatable, GeneralParams, IODEPopulation, OIDEAdd, OIDEBoundApplication,
        OIDECrossover, OIDEDiff, OIDEOpposite, OIDERandomize, OIDEScale, OIDEZero, TrialType,
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
            + OIDECrossover
            + OIDEBoundApplication
            + OIDEZero
            + Clone,
    {
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
        fn opposite(&self, midpoint: Option<&Self>) -> Self;
    }
    pub trait OIDERandomize {
        fn random(&self, rng: &mut impl Rng) -> Self;
    }
    pub trait OIDECrossover {
        fn crossover(&self, other: &Self, rng: &mut impl Rng, rate: f64) -> Self;
    }
    pub trait OIDEBoundApplication {
        fn apply_bounds(&self, other: &Self) -> Self;
    }
    pub trait OIDEZero {
        fn zero(&self) -> Self;
    }

    pub trait Evaluatable<E>: Differentiable {
        type Params;
        type EvalInfo;
        fn eval(
            &self,
            general_params: &GeneralParams,
            params: &Self::Params,
        ) -> (E, Self::EvalInfo);
    }

    pub trait IODEPopulation<B: Evaluatable<E>, E, P, I> {
        fn get_size(&self) -> usize;
        fn get_population(&self) -> Vec<&B>;
        fn get_midpoints(&self) -> B;
        fn step(
            &self,
            selection: &mut dyn FnMut(&[(B, E, I)]) -> Vec<B>,
            rng: &mut impl Rng,
            params: P,
            midpoint: Option<&B>,
            f: f32,
            crossover_rate: f64,
        ) -> Vec<B>;
    }

    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub enum TrialType {
        Target,
        TargetOpposite,
        Trial,
        TrialOpposite,
    }

    impl TrialType {
        pub fn to_string(&self) -> &str {
            match self {
                TrialType::Target => "Target",
                TrialType::TargetOpposite => "-Target",
                TrialType::Trial => "Target+Trial",
                TrialType::TrialOpposite => "Target-Trial",
            }
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct GeneralParams {
        pub pop_id: usize,
        pub pop_size: usize,
        pub trial_type: TrialType,
    }

    impl GeneralParams {
        fn new(pop_id: usize, pop_size: usize, trial_type: TrialType) -> GeneralParams {
            GeneralParams {
                pop_id,
                pop_size,
                trial_type,
            }
        }
    }

    impl<B, E, P, I> IODEPopulation<B, E, P, I> for Vec<B>
    where
        B: Evaluatable<E, Params = P, EvalInfo = I> + PartialEq + Debug,
        E: Debug,
    {
        fn get_size(&self) -> usize {
            self.len()
        }

        fn get_population(&self) -> Vec<&B> {
            self.iter().collect::<Vec<&B>>()
        }

        fn get_midpoints(&self) -> B {
            let count = self.get_size();
            self.iter()
                .map(|b| b.scale(1.0 / count as f32))
                .fold(self[0].zero(), |acc, next| acc.add(&next))
        }

        fn step(
            &self,
            selection: &mut dyn FnMut(&[(B, E, I)]) -> Vec<B>,
            rng: &mut impl Rng,
            params: P,
            midpoint: Option<&B>,
            f: f32,
            crossover_rate: f64,
        ) -> Vec<B> {
            let variants = self
                .get_population()
                .iter()
                .enumerate()
                .map(|(idx, &target)| {
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
                    let mutant = target.add(&parent1.difference(parent2).scale(f));
                    //println!(
                    //    "BPPM: {:6.2?} {:6.2?} {:6.2?} {:6.2?}",
                    //    &target, &parent1, &parent2, &mutant
                    //);
                    let trial = target.crossover(&mutant, rng, crossover_rate);
                    [
                        (
                            GeneralParams::new(idx, self.get_size(), TrialType::Target),
                            target.clone(),
                        ),
                        (
                            GeneralParams::new(idx, self.get_size(), TrialType::Trial),
                            trial.clone(),
                        ),
                        (
                            GeneralParams::new(idx, self.get_size(), TrialType::TrialOpposite),
                            trial.opposite(midpoint.clone()),
                        ),
                        (
                            GeneralParams::new(idx, self.get_size(), TrialType::TargetOpposite),
                            target.opposite(midpoint.clone()),
                        ),
                    ]
                })
                .collect::<Vec<_>>();

            //println!("Variants: [");
            //for var in variants.iter() {
            //    println!(
            //        "[{}],",
            //        var.iter()
            //            .map(|foo| format!("{:6.2?},", foo.1))
            //            .collect::<String>()
            //    );
            //}
            //println!("]");

            let evaled_pairs: Vec<_> = variants
                .iter()
                .enumerate()
                .map(|(_idx, set)| {
                    //println!(
                    //    "################\nEval #{:3} of {:3}\n################",
                    //    idx + 1,
                    //    self.len()
                    //);
                    //std::thread::sleep(std::time::Duration::from_millis(300));
                    set.into_iter()
                        .map(|(general_params, base)| {
                            let (eval, info) = base.eval(general_params, &params);
                            (base.clone(), eval, info)
                        })
                        .collect::<Vec<_>>()
                })
                .collect();

            //println!("Evaled: [");
            //for var in evaled_pairs.iter() {
            //    println!(
            //        "[{}],",
            //        var.iter()
            //            .map(|foo| format!("({:6.2?}, {:6.2?}),", foo.0, foo.1))
            //            .collect::<String>()
            //    );
            //}
            //println!("]");

            evaled_pairs
                .into_iter()
                .flat_map(|pair| selection(&pair))
                .collect()
        }
    }
}

#[cfg(test)]
pub mod tests {
    #[allow(unused_imports)]
    use crate::prelude::*;
    use crate::traits::OIDEZero;
    use derive_diff::*;
    use rand::{distributions::Uniform, prelude::*};

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
        fn opposite(&self, midpoint: Option<&Self>) -> Self {
            2.0 * midpoint.unwrap_or(&0.0) - self
        }
    }
    impl OIDERandomize for f32 {
        fn random(&self, rng: &mut impl rand::Rng) -> Self {
            rng.gen()
        }
    }
    impl OIDECrossover for f32 {
        fn crossover(&self, other: &Self, rng: &mut impl rand::Rng, rate: f64) -> Self {
            if rng.gen_bool(rate) {
                *other
            } else {
                *self
            }
        }
    }
    impl OIDEBoundApplication for f32 {
        fn apply_bounds(&self, other: &Self) -> Self {
            *other
        }
    }
    impl OIDEZero for f32 {
        fn zero(&self) -> Self {
            0.0
        }
    }

    impl Differentiable for f32 {}

    impl Evaluatable<f32> for f32 {
        type Params = f32;
        type EvalInfo = TrialType;
        fn eval(&self, gen_par: &GeneralParams, target: &Self::Params) -> (f32, TrialType) {
            ((self - target).abs(), gen_par.trial_type)
        }
    }

    #[test]
    fn test_ode_1d() {
        for seed in 234168374800u64..234168374850u64 {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let mut pop: Vec<_> = (30..=45).map(|v| v as f32).collect();
            let mut lastbest = f32::MAX;
            let mut bests = vec![];
            let target = 5.37;

            let newbest = pop.iter().fold(f32::MAX, |acc, elem| {
                let eval = elem
                    .eval(
                        &GeneralParams {
                            pop_id: 0,
                            pop_size: 0,
                            trial_type: TrialType::Trial,
                        },
                        &target,
                    )
                    .0;
                if acc < eval {
                    acc
                } else {
                    eval
                }
            });
            println!("Best Error : {:7.2?}", newbest);
            bests.push(newbest);

            for i in 1..=30 {
                println!("Iteration {i}", i = i);
                println!("Input      : {:7.2?}", pop);
                let midpoints = pop.get_midpoints();
                println!("Midpoint   : {:?}", midpoints);
                pop = pop.step(
                    &mut |t: &[(f32, f32, TrialType)]| {
                        let res = t
                            .iter()
                            .min_by(|a, b| {
                                a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Less)
                            })
                            .unwrap();
                        vec![res.0]
                    },
                    &mut rng,
                    target,
                    None,
                    0.5,
                    1.0,
                );
                println!("Result     : {:7.2?}", pop);
                let newbest = pop.iter().fold(f32::MAX, |acc, elem| {
                    let eval = elem
                        .eval(
                            &GeneralParams {
                                pop_id: 0,
                                pop_size: 0,
                                trial_type: TrialType::Trial,
                            },
                            &target,
                        )
                        .0;
                    if acc < eval {
                        acc
                    } else {
                        eval
                    }
                });
                println!("Best Error : {:?}", newbest);
                println!("Improvement: {:7.2?}", lastbest - newbest);
                lastbest = newbest;
                bests.push(newbest);
                println!();
            }
            println!("Error: {:6.2?}", bests);
            assert!(
                bests
                    .iter()
                    .min_by(|a, b| a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Less))
                    .unwrap()
                    <= &0.1f32
            )
        }
    }

    #[derive(Clone, Debug, PartialEq, AllOIDETraits)]
    struct Vec4(f32, f32, f32, f32);

    impl Vec4 {
        fn dist(&self, other: &Self) -> f32 {
            f64::sqrt(
                f64::powi(other.0 as f64 - self.0 as f64, 2)
                    + f64::powi(other.1 as f64 - self.1 as f64, 2)
                    + f64::powi(other.2 as f64 - self.2 as f64, 2)
                    + f64::powi(other.3 as f64 - self.3 as f64, 2),
            ) as f32
        }
        fn rand(lower: f32, upper: f32, rng: &mut impl Rng) -> Vec4 {
            let distr = Uniform::new_inclusive(lower, upper);
            Vec4(
                rng.sample(distr),
                rng.sample(distr),
                rng.sample(distr),
                rng.sample(distr),
            )
        }
    }

    impl Evaluatable<f32> for Vec4 {
        type Params = Vec4;
        type EvalInfo = TrialType;
        fn eval(&self, gen_par: &GeneralParams, target: &Self::Params) -> (f32, TrialType) {
            (self.dist(target), gen_par.trial_type)
        }
    }

    #[test]
    fn test_ode_4d() {
        for iters in (20..=100).step_by(20) {
            let mut avg_bests = vec![];
            for populationsize in (20..=60).step_by(20) {
                let mut avg_best = None;
                for seed in 234168374000u64..234168374100u64 {
                    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
                    let mut pop: Vec<_> = (0..=populationsize)
                        .map(|_| Vec4::rand(-50.0, 50.0, &mut rng))
                        .collect();
                    let mut lastbest = f32::MAX;
                    let mut bests = vec![];
                    let target = Vec4(7.3, 5.2, -10.0, 0.0);

                    let newbest = pop.iter().fold(f32::MAX, |acc, elem| {
                        let eval = elem
                            .eval(
                                &GeneralParams {
                                    pop_id: 0,
                                    pop_size: 0,
                                    trial_type: TrialType::Trial,
                                },
                                &target,
                            )
                            .0;
                        if acc < eval {
                            acc
                        } else {
                            eval
                        }
                    });
                    //println!("Best Error : {:7.2?}", newbest);
                    bests.push(newbest);

                    for _i in 1..=iters {
                        //println!("Iteration {i}", i = i);
                        //println!("Input      : {:7.2?}", pop);
                        let midpoints = pop.get_midpoints();
                        //println!("Midpoint   : {:?}", midpoints);
                        pop = pop.step(
                            &mut |t: &[(Vec4, f32, TrialType)]| {
                                let res = t
                                    .iter()
                                    .min_by(|a, b| {
                                        a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Less)
                                    })
                                    .unwrap();
                                vec![res.0.clone()]
                            },
                            &mut rng,
                            target.clone(),
                            Some(&midpoints),
                            0.5,
                            0.5,
                        );
                        //println!("Result     : {:7.2?}", pop);
                        let newbest = pop.iter().fold(f32::MAX, |acc, elem| {
                            let eval = elem
                                .eval(
                                    &GeneralParams {
                                        pop_id: 0,
                                        pop_size: 0,
                                        trial_type: TrialType::Trial,
                                    },
                                    &target,
                                )
                                .0;
                            if acc < eval {
                                acc
                            } else {
                                eval
                            }
                        });
                        //println!("Best Error : {:?}", newbest);
                        //println!("Improvement: {:7.2?}", lastbest - newbest);
                        lastbest = newbest;
                        bests.push(newbest);
                        //println!();
                    }
                    //println!("Error: {:6.2?}", bests);
                    //assert!(
                    //    bests
                    //        .iter()
                    //        .min_by(|a, b| a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Less))
                    //        .unwrap()
                    //        <= &0.2f32
                    //);
                    avg_best = avg_best.map_or(
                        Some((1, bests[bests.len() - 1])),
                        |(ctr, acc): (usize, f32)| Some((ctr + 1, acc + bests[bests.len() - 1])),
                    );
                }
                avg_bests.push(avg_best.unwrap().1 / avg_best.unwrap().0 as f32);
            }
            println!("Average Bests at {:3}: {:8.4?}", iters as f32, avg_bests);
        }
        assert!(false);
    }
}
