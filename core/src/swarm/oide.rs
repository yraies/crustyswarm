use std::fmt::Debug;

use rand::prelude::*;

use super::{grammar::SwarmGrammar, oide_genome::OIDESwarmGenome};

pub trait Differentiable {
    fn add(&self, other: &Self) -> Self;
    fn difference(&self, other: &Self) -> Self;
    fn scale(&self, factor: f32) -> Self;
    fn opposite(&self) -> Self;
    fn cop(&self) -> Self;
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
        selection: &mut dyn FnMut(((B, E), (B, E), (B, E)), usize) -> B,
        rng: &mut impl Rng,
        params: P,
    ) -> Vec<B>;
}

impl<B: Evaluatable<E, Params = P> + PartialEq + Debug, E, P> IODEPopulation<B, E, P> for Vec<B> {
    fn get_size(&self) -> usize {
        self.len()
    }

    fn get_population(&self) -> Vec<&B> {
        self.iter().collect::<Vec<&B>>()
    }

    fn step(
        &self,
        selection: &mut dyn FnMut(((B, E), (B, E), (B, E)), usize) -> B,
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
                (target.cop(), trial, target_opposite)
            })
            .collect::<Vec<_>>();

        let evaled_pairs = variants
            .iter()
            //.inspect(|f| println!("{:?}, {:?}, {:?}", f.0, f.1, f.2))
            .map(|(target, trial, opposite)| {
                (
                    (target.cop(), target.eval(&params)),
                    (trial.cop(), trial.eval(&params)),
                    (opposite.cop(), opposite.eval(&params)),
                )
            })
            .collect::<Vec<_>>();

        let selected_pop = evaled_pairs
            .into_iter()
            .enumerate()
            .map(|(num, pair)| selection(pair, num))
            .collect();

        selected_pop
    }
}

impl Differentiable for f32 {
    fn add(&self, other: &f32) -> f32 {
        self + other
    }

    fn difference(&self, other: &f32) -> f32 {
        self - other
    }

    fn scale(&self, factor: f32) -> f32 {
        self * factor
    }

    fn opposite(&self) -> f32 {
        -self
    }

    fn cop(&self) -> Self {
        *self
    }
}

impl Evaluatable<f32> for f32 {
    type Params = ();
    fn eval(&self, _params: &Self::Params) -> f32 {
        *self
    }
}

impl Evaluatable<SwarmGrammar> for OIDESwarmGenome {
    type Params = (u64, u64);
    fn eval(&self, params: &Self::Params) -> SwarmGrammar {
        let mut rnd = StdRng::seed_from_u64(params.0);
        let genome = super::genome::SwarmGenome::from(self);
        //println!("{:?}", &genome);
        let mut sg = SwarmGrammar::from(genome, &mut rnd);
        //println!("{:?}", &sg);
        for _ in 0..params.1 {
            sg.step(&mut rnd);
        }
        sg
    }
}

#[test]
fn test_ode_20() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(2345678912u64);
    rng.gen_bool(0.5);
    let mut pop = rng
        .sample_iter(&rand::distributions::Standard)
        .take(10)
        .map(|f: f32| f * 100.0)
        .collect::<Vec<f32>>();
    pop.get_population();
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
                    t1.1
                } else if d2 < d3 {
                    t2.1
                } else {
                    t3.1
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
