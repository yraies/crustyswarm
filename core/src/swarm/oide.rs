use std::fmt::Debug;

use rand::prelude::*;

pub trait Differentiable {
    fn add(&self, other: &Self) -> Self;
    fn difference(&self, other: &Self) -> Self;
    fn scale(&self, factor: f32) -> Self;
    fn opposite(&self) -> Self;
    fn cop(&self) -> Self;
}

pub trait Evaluatable<E>: Differentiable {
    fn eval(&self) -> E;
}

trait IODEPopulation<B, E> {
    fn get_size(&self) -> usize;
    fn get_population(&self) -> Vec<&B>;
    fn step(&self, selection: fn(((&B, E), (&B, E), (&B, E))) -> B, rng: &mut StdRng) -> Vec<B>;
}

impl<B: Evaluatable<E> + PartialEq + Debug, E> IODEPopulation<B, E> for Vec<B> {
    fn get_size(&self) -> usize {
        self.len()
    }

    fn get_population(&self) -> Vec<&B> {
        self.iter().collect::<Vec<&B>>()
    }

    fn step(&self, selection: fn(((&B, E), (&B, E), (&B, E))) -> B, rng: &mut StdRng) -> Vec<B> {
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
                let target_opposite = target.opposite();
                (target.cop(), trial, target_opposite)
            })
            .collect::<Vec<_>>();

        let evaled_pairs = variants
            .iter()
            //.inspect(|f| println!("{:?}, {:?}, {:?}", f.0, f.1, f.2))
            .map(|(target, trial, opposite)| {
                (
                    (target, target.eval()),
                    (trial, trial.eval()),
                    (opposite, opposite.eval()),
                )
            })
            .collect::<Vec<_>>();

        let selected_pop = evaled_pairs
            .into_iter()
            .map(|pair| selection(pair))
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
    fn eval(&self) -> f32 {
        *self
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
            |(t1, t2, t3)| {
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
