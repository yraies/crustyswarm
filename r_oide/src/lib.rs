pub mod atoms;

pub mod prelude {
    pub use crate::traits::{
        Differentiable, Evaluatable, FeatureCollector, FeatureTraversal, GeneralParams,
        IODEPopulation, OIDEAdd, OIDEBoundApplication, OIDECrossover, OIDEDiff, OIDEOpposite,
        OIDEParameterCount, OIDERandomize, OIDEScale, OIDEZero, TrialType, Visit, Visitor,
    };

    pub use crate::atoms::{
        bool::*, bounded_float::*, bounded_int::*, fixed::*, multiset::*, tuples::*, *,
    };
}

pub mod traits {
    use rand::prelude::*;
    use std::{collections::hash_map::DefaultHasher, fmt::Debug, hash::Hasher};

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
            + OIDEParameterCount
            + Visit<f32>
            + Visit<FeatureTraversal>
            + Clone
            + std::hash::Hash,
    {
        fn my_hash(&self) -> u64 {
            let mut hasher = DefaultHasher::new();
            self.hash(&mut hasher);
            hasher.finish()
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
    pub trait OIDEParameterCount {
        fn parameter_count(&self) -> usize;
    }

    pub trait Visitor<T> {
        type Error;
        fn handle(&mut self, data: T) -> Result<(), Self::Error>;
    }
    pub trait Visit<T> {
        fn visit_with<V: Visitor<T>>(&self, visitor: &mut V) -> Result<(), V::Error>;
    }
    pub struct VecCollector(Vec<f32>);
    impl Visitor<f32> for VecCollector {
        type Error = ();

        fn handle(&mut self, data: f32) -> Result<(), Self::Error> {
            self.0.push(data);
            Ok(())
        }
    }
    impl VecCollector {
        pub fn collect(site: &impl Visit<f32>) -> Vec<f32> {
            let mut collector = VecCollector(vec![]);
            site.visit_with(&mut collector).unwrap();
            collector.0
        }
    }
    pub enum FeatureTraversal {
        Push(String),
        Collect(String),
        Pop,
    }
    pub struct FeatureCollector(Vec<String>, Vec<String>);
    impl Visitor<FeatureTraversal> for FeatureCollector {
        type Error = ();

        fn handle(&mut self, data: FeatureTraversal) -> Result<(), Self::Error> {
            match data {
                FeatureTraversal::Push(name) => self.1.push(name),
                FeatureTraversal::Collect(name) => {
                    self.1.push(name);
                    self.0
                        .push(itertools::Itertools::join(&mut self.1.iter(), "."));
                    self.1.pop();
                }
                FeatureTraversal::Pop => {
                    self.1.pop();
                }
            }
            Ok(())
        }
    }
    impl FeatureCollector {
        pub fn collect(site: &impl Visit<FeatureTraversal>) -> Vec<String> {
            let mut collector = FeatureCollector(vec![], vec![]);
            site.visit_with(&mut collector).unwrap();
            collector.0
        }
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
                TrialType::Target => "+Target",
                TrialType::TargetOpposite => "-Target",
                TrialType::Trial => "+Trial",
                TrialType::TrialOpposite => "-Trial",
            }
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct GeneralParams {
        pub pop_id: usize,
        pub pop_size: usize,
        pub trial_type: TrialType,
        pub parents: (u64, u64),
    }

    impl GeneralParams {
        fn new(
            pop_id: usize,
            pop_size: usize,
            trial_type: TrialType,
            parents: (u64, u64),
        ) -> GeneralParams {
            GeneralParams {
                pop_id,
                pop_size,
                trial_type,
                parents,
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
                    let parents = (parent1.my_hash(), parent2.my_hash());
                    let mutant = target.add(&parent1.difference(parent2).scale(f));
                    //println!(
                    //    "BPPM: {:6.2?} {:6.2?} {:6.2?} {:6.2?}",
                    //    &target, &parent1, &parent2, &mutant
                    //);
                    let trial = target.crossover(&mutant, rng, crossover_rate);
                    [
                        (
                            GeneralParams::new(idx, self.get_size(), TrialType::Target, parents),
                            target.clone(),
                        ),
                        (
                            GeneralParams::new(idx, self.get_size(), TrialType::Trial, parents),
                            trial.clone(),
                        ),
                        (
                            GeneralParams::new(
                                idx,
                                self.get_size(),
                                TrialType::TrialOpposite,
                                parents,
                            ),
                            trial.opposite(midpoint.clone()),
                        ),
                        (
                            GeneralParams::new(
                                idx,
                                self.get_size(),
                                TrialType::TargetOpposite,
                                parents,
                            ),
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
    use crate::traits::{
        FeatureCollector, FeatureTraversal, OIDEZero, VecCollector, Visit, Visitor,
    };
    use derive_diff::*;
    use rand::{distributions::Uniform, prelude::*};

    #[derive(Clone, Debug, PartialEq)]
    struct F32(f32);

    impl From<f32> for F32 {
        fn from(f: f32) -> Self {
            F32(f)
        }
    }
    impl From<F32> for f32 {
        fn from(f: F32) -> Self {
            f.0
        }
    }
    impl std::ops::Deref for F32 {
        type Target = f32;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl OIDEAdd for F32 {
        fn add(&self, other: &Self) -> Self {
            F32(self.0 + other.0)
        }
    }
    impl OIDEDiff for F32 {
        fn difference(&self, other: &Self) -> Self {
            F32(other.0 - self.0)
        }
    }
    impl OIDEScale for F32 {
        fn scale(&self, factor: f32) -> Self {
            F32(self.0 * factor)
        }
    }
    impl OIDEOpposite for F32 {
        fn opposite(&self, midpoint: Option<&Self>) -> Self {
            F32(2.0 * midpoint.unwrap_or(&F32(0.0)).0 - self.0)
        }
    }
    impl OIDERandomize for F32 {
        fn random(&self, rng: &mut impl rand::Rng) -> Self {
            F32(rng.gen())
        }
    }
    impl OIDECrossover for F32 {
        fn crossover(&self, other: &Self, rng: &mut impl rand::Rng, rate: f64) -> Self {
            if rng.gen_bool(rate) {
                other.0.into()
            } else {
                self.0.into()
            }
        }
    }
    impl OIDEBoundApplication for F32 {
        fn apply_bounds(&self, other: &Self) -> Self {
            other.clone()
        }
    }
    impl OIDEZero for F32 {
        fn zero(&self) -> Self {
            F32(0.0)
        }
    }
    impl OIDEParameterCount for F32 {
        fn parameter_count(&self) -> usize {
            1
        }
    }
    impl Visit<f32> for F32 {
        fn visit_with<V: Visitor<f32>>(&self, f: &mut V) -> Result<(), V::Error> {
            f.handle(self.0)
        }
    }
    impl Visit<FeatureTraversal> for F32 {
        fn visit_with<V: Visitor<FeatureTraversal>>(&self, f: &mut V) -> Result<(), V::Error> {
            f.handle(FeatureTraversal::Collect("f32".to_string()))
        }
    }
    impl std::hash::Hash for F32 {
        fn hash<H>(&self, state: &mut H)
        where
            H: std::hash::Hasher,
        {
            self.0.to_string().hash(state);
        }
    }

    impl Differentiable for F32 {}

    impl Evaluatable<f32> for F32 {
        type Params = f32;
        type EvalInfo = TrialType;
        fn eval(&self, gen_par: &GeneralParams, target: &Self::Params) -> (f32, TrialType) {
            ((self.0 - target).abs(), gen_par.trial_type)
        }
    }

    #[test]
    fn test_ode_1d() {
        for seed in 234168374800u64..234168374850u64 {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let mut pop: Vec<F32> = (30..=45).map(|v| (v as f32).into()).collect();
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
                            parents: (0, 0),
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
                    &mut |t: &[(F32, f32, TrialType)]| {
                        let res = t
                            .iter()
                            .min_by(|a, b| {
                                a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Less)
                            })
                            .unwrap();
                        vec![res.0.clone()]
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
                                parents: (0, 0),
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

    #[derive(Clone, Hash, Debug, PartialEq, AllOIDETraits)]
    struct Vec4(F32, F32, F32, F32);

    impl Vec4 {
        fn dist(&self, other: &Self) -> f32 {
            f64::sqrt(
                f64::powi(other.0 .0 as f64 - self.0 .0 as f64, 2)
                    + f64::powi(other.1 .0 as f64 - self.1 .0 as f64, 2)
                    + f64::powi(other.2 .0 as f64 - self.2 .0 as f64, 2)
                    + f64::powi(other.3 .0 as f64 - self.3 .0 as f64, 2),
            ) as f32
        }
        fn rand(lower: f32, upper: f32, rng: &mut impl Rng) -> Vec4 {
            let distr = Uniform::new_inclusive(lower, upper);
            Vec4(
                rng.sample(distr).into(),
                rng.sample(distr).into(),
                rng.sample(distr).into(),
                rng.sample(distr).into(),
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
    fn test_collection() {
        let foo: F32 = 3.6f32.into();
        assert_eq!(vec![3.6f32], VecCollector::collect(&foo));
        assert_eq!(vec!("f32"), FeatureCollector::collect(&foo));
        let bar = Vec4(1f32.into(), 2f32.into(), 3f32.into(), 4f32.into());
        assert_eq!(vec![1f32, 2f32, 3f32, 4f32], VecCollector::collect(&bar));
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
                    //let mut lastbest = f32::MAX;
                    let mut bests = vec![];
                    let target = Vec4(7.3.into(), 5.2.into(), (-10.0).into(), 0.0.into());

                    let newbest = pop.iter().fold(f32::MAX, |acc, elem| {
                        let eval = elem
                            .eval(
                                &GeneralParams {
                                    pop_id: 0,
                                    pop_size: 0,
                                    trial_type: TrialType::Trial,
                                    parents: (0, 0),
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
                                        parents: (0, 0),
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
                        //lastbest = newbest;
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
