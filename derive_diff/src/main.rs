use derive_diff::*;
use rand::prelude::*;

trait Differentiable
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
        self.add(&parent1.difference(parent2).opposite().scale(factor))
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
    fn opposite(&self) -> Self;
}
trait OIDERandomize {
    fn random(&self, rng: &mut impl Rng) -> Self;
}
trait OIDEBoundApplication {
    fn apply_bounds(&self, other: &Self) -> Self;
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
    fn opposite(&self) -> Self {
        usize::MAX - self
    }
}
impl OIDERandomize for usize {
    fn random(&self, rng: &mut impl rand::Rng) -> Self {
        rng.gen()
    }
}
impl OIDEBoundApplication for usize {
    fn apply_bounds(&self, other0: &Self) -> Self {
        *other0
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
    fn opposite(&self) -> Self {
        (
            OIDEOpposite::opposite(&self.0),
            OIDEOpposite::opposite(&self.1),
        )
    }
}
impl<T: Differentiable> OIDERandomize for (T, T) {
    fn random(&self, rng: &mut impl rand::Rng) -> Self {
        (self.0.random(rng), self.1.random(rng))
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
impl<T: Differentiable> Differentiable for (T, T) {}

#[derive(
    OIDEAdd,
    OIDEDiff,
    OIDEScale,
    OIDEOpposite,
    OIDERandomize,
    OIDEBoundApplication,
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
    OIDEBoundApplication,
    Clone,
    Differentiable,
)]
struct UnitStruct;

#[allow(dead_code)]
#[derive(
    Debug,
    OIDEAdd,
    OIDEDiff,
    OIDEScale,
    OIDEOpposite,
    OIDERandomize,
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
}
