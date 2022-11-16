use derive_diff::*;
use rand::prelude::*;

trait Differentiable
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
        + Clone,
{
    fn trial_plus_from(&self, parent1: &Self, parent2: &Self, factor: f32) -> Self {
        self.add(&parent1.difference(parent2).scale(factor))
    }
    fn trial_minus_from(&self, parent1: &Self, parent2: &Self, factor: f32) -> Self {
        self.add(&parent1.difference(parent2).opposite(None).scale(factor))
    }
}

trait OIDEAdd {
    fn add(&self, other: &Self) -> Self;
}
trait OIDEDiff {
    fn difference(&self, other: &Self) -> Self;
}
trait OIDEScale {
    fn scale(&self, factor: f32) -> Self;
}
trait OIDEOpposite {
    fn opposite(&self, midpoint: Option<&Self>) -> Self;
}
trait OIDERandomize {
    fn random(&self, rng: &mut impl Rng) -> Self;
}
trait OIDECrossover {
    fn crossover(&self, other: &Self, rng: &mut impl Rng, rate: f64) -> Self;
}
trait OIDEBoundApplication {
    fn apply_bounds(&self, other: &Self) -> Self;
}
trait OIDEZero {
    fn zero(&self) -> Self;
}
trait OIDEParameterCount {
    fn parameter_count(&self) -> usize;
}
trait Visitor<T> {
    type Error;
    fn handle(&mut self, data: T) -> Result<(), Self::Error>;
}
trait Visit<T> {
    fn visit_with<V: Visitor<T>>(&self, f: &mut V) -> Result<(), V::Error>;
}
enum FeatureTraversal {
    Push(String),
    Collect(String),
    Pop,
}

impl OIDEAdd for usize {
    fn add(&self, other0: &Self) -> Self {
        self + other0
    }
}
impl OIDEDiff for usize {
    fn difference(&self, other0: &Self) -> Self {
        self.max(other0) - self.min(other0)
    }
}
impl OIDEScale for usize {
    fn scale(&self, factor: f32) -> Self {
        ((*self as f32) * factor) as usize
    }
}
impl OIDEOpposite for usize {
    fn opposite(&self, midpoint: Option<&Self>) -> Self {
        midpoint.map(|m| m * 2).unwrap_or(usize::MAX) - self
    }
}
impl OIDERandomize for usize {
    fn random(&self, rng: &mut impl rand::Rng) -> Self {
        rng.gen()
    }
}
impl OIDECrossover for usize {
    fn crossover(&self, other: &Self, rng: &mut impl Rng, rate: f64) -> Self {
        if rng.gen_bool(rate) {
            *other
        } else {
            *self
        }
    }
}
impl OIDEBoundApplication for usize {
    fn apply_bounds(&self, other0: &Self) -> Self {
        *other0
    }
}
impl OIDEZero for usize {
    fn zero(&self) -> Self {
        0
    }
}
impl OIDEParameterCount for usize {
    fn parameter_count(&self) -> usize {
        1
    }
}
impl Visit<f32> for usize {
    fn visit_with<V: Visitor<f32>>(&self, f: &mut V) -> Result<(), V::Error> {
        f.handle(*self as f32)
    }
}
impl Visit<FeatureTraversal> for usize {
    fn visit_with<V: Visitor<FeatureTraversal>>(&self, f: &mut V) -> Result<(), V::Error> {
        f.handle(FeatureTraversal::Collect("usize".to_string()))
    }
}
impl Differentiable for usize {}

impl<T: Differentiable> OIDEAdd for (T, T) {
    fn add(&self, other0: &Self) -> Self {
        (
            OIDEAdd::add(&self.0, &other0.0),
            OIDEAdd::add(&self.1, &other0.1),
        )
    }
}
impl<T: Differentiable> OIDEDiff for (T, T) {
    fn difference(&self, other0: &Self) -> Self {
        (
            OIDEDiff::difference(&self.0, &other0.0),
            OIDEDiff::difference(&self.1, &other0.1),
        )
    }
}
impl<T: Differentiable> OIDEScale for (T, T) {
    fn scale(&self, factor: f32) -> Self {
        (
            OIDEScale::scale(&self.0, factor),
            OIDEScale::scale(&self.1, factor),
        )
    }
}
impl<T: Differentiable> OIDEOpposite for (T, T) {
    fn opposite(&self, midpoint: Option<&Self>) -> Self {
        (
            OIDEOpposite::opposite(
                &self.0,
                match midpoint {
                    Some(m) => Some(&m.0),
                    None => None,
                },
            ),
            OIDEOpposite::opposite(
                &self.1,
                match midpoint {
                    Some(m) => Some(&m.1),
                    None => None,
                },
            ),
        )
    }
}
impl<T: Differentiable> OIDERandomize for (T, T) {
    fn random(&self, rng: &mut impl rand::Rng) -> Self {
        (self.0.random(rng), self.1.random(rng))
    }
}
impl<T: Differentiable + Clone> OIDECrossover for (T, T) {
    fn crossover(&self, other: &Self, rng: &mut impl Rng, rate: f64) -> Self {
        (
            if rng.gen_bool(rate) {
                other.0.clone()
            } else {
                self.0.clone()
            },
            if rng.gen_bool(rate) {
                other.1.clone()
            } else {
                self.1.clone()
            },
        )
    }
}
impl<T: Differentiable + Clone> OIDEZero for (T, T) {
    fn zero(&self) -> Self {
        (self.0.zero(), self.1.zero())
    }
}
impl<T: Differentiable + Clone> OIDEParameterCount for (T, T) {
    fn parameter_count(&self) -> usize {
        self.0.parameter_count() + self.1.parameter_count()
    }
}
impl<T: Differentiable> OIDEBoundApplication for (T, T) {
    fn apply_bounds(&self, other0: &Self) -> Self {
        (
            OIDEBoundApplication::apply_bounds(&self.0, &other0.0),
            OIDEBoundApplication::apply_bounds(&self.1, &other0.1),
        )
    }
}
impl<T: Differentiable> Visit<f32> for (T, T) {
    fn visit_with<V: Visitor<f32>>(&self, f: &mut V) -> Result<(), V::Error> {
        self.0.visit_with(f)?;
        self.1.visit_with(f)
    }
}
impl<T: Differentiable> Differentiable for (T, T) {}

#[derive(
    OIDEAdd,
    OIDEDiff,
    OIDEScale,
    OIDEOpposite,
    OIDERandomize,
    OIDECrossover,
    OIDEBoundApplication,
    OIDEZero,
    OIDEParameterCount,
    VisitF32,
    Clone,
    Differentiable,
)]
struct NamedStruct {
    baz: usize,
    var: usize,
}

#[derive(Clone, AllOIDETraits)]
struct UnnamedStruct(usize, usize);

#[derive(
    Debug,
    OIDEAdd,
    OIDEDiff,
    OIDEScale,
    OIDEOpposite,
    OIDERandomize,
    OIDECrossover,
    OIDEBoundApplication,
    OIDEZero,
    OIDEParameterCount,
    VisitF32,
    Clone,
    Differentiable,
)]
struct UnitStruct;

/*
#[allow(dead_code)]
#[derive(
    Debug,
    OIDEAdd,
    OIDEDiff,
    OIDEScale,
    OIDEOpposite,
    OIDERandomize,
    OIDECrossover,
    OIDEBoundApplication,
    Clone,
    Differentiable,
)]
enum Enum {
    Unnamed((usize, usize)),
    Named {
        val: usize,
        var: usize,
        stuff: (usize, usize),
    },
    Unit,
}
*/

fn main() {
    let a = NamedStruct { baz: 7, var: 3 };
    let b = NamedStruct { baz: 1, var: 6 };
    println!("Hello, Named Struct! Result: {}", a.add(&b).var);
    let a = UnnamedStruct(7, 3);
    let b = UnnamedStruct(1, 6);
    println!("Hello, Unnamed Struct! Result: {}", a.add(&b).1);
    let a = UnitStruct;
    let b = UnitStruct;
    let _c = a.add(&b);
    println!("Hello, Unit Struct!");
    /*
    let a = Enum::Unnamed((7, 3));
    let b = Enum::Unnamed((1, 6));
    if let Enum::Unnamed((_, c)) = a.add(&b) {
        println!("Hello, Unnamed Enum! Result: {}", c);
    }
    let a = Enum::Named {
        val: 7,
        var: 3,
        stuff: (7, 3),
    };
    let b = Enum::Named {
        val: 1,
        var: 6,
        stuff: (1, 6),
    };
    if let Enum::Named { stuff, .. } = a.add(&b) {
        println!("Hello, Named Enum! Result: {}", stuff.1);
    }
    let a = Enum::Unit;
    let b = Enum::Unit;
    if let Enum::Unit = a.add(&b) {
        println!("Hello, Unit Enum!");
    }
    let a = 10usize;
    let b = 50usize;
    println!("Hello, usize! Result: {}", a.difference(&b).scale(0.1));
    */
}
