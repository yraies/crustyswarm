use derive_diff::Differentiable;

pub trait Differentiable {
    fn add(&self, other0: &Self) -> Self;
    fn difference(&self, other0: &Self) -> Self;
    fn scale(&self, factor: f32) -> Self;
    fn opposite(&self) -> Self;
    fn clon(&self) -> Self;
    fn random(&self, rng: &mut impl rand::Rng) -> Self;
    fn apply_bounds(&self, other0: &Self) -> Self;
}

impl Differentiable for usize {
    fn add(&self, other0: &Self) -> Self {
        self + other0
    }
    fn difference(&self, other0: &Self) -> Self {
        self.max(other0) - self.min(other0)
    }
    fn scale(&self, factor: f32) -> Self {
        ((*self as f32) * factor) as usize
    }
    fn opposite(&self) -> Self {
        usize::MAX - self
    }
    fn clon(&self) -> Self {
        self.clone()
    }

    fn random(&self, rng: &mut impl rand::Rng) -> Self {
        rng.gen()
    }

    fn apply_bounds(&self, other0: &Self) -> Self {
        *other0
    }
}

impl<T: Differentiable> Differentiable for (T, T) {
    fn add(&self, other0: &Self) -> Self {
        (
            Differentiable::add(&self.0, &other0.0),
            Differentiable::add(&self.1, &other0.1),
        )
    }
    fn difference(&self, other0: &Self) -> Self {
        (
            Differentiable::difference(&self.0, &other0.0),
            Differentiable::difference(&self.1, &other0.1),
        )
    }
    fn scale(&self, factor: f32) -> Self {
        (
            Differentiable::scale(&self.0, factor),
            Differentiable::scale(&self.1, factor),
        )
    }
    fn opposite(&self) -> Self {
        (
            Differentiable::opposite(&self.0),
            Differentiable::opposite(&self.1),
        )
    }
    fn clon(&self) -> Self {
        (Differentiable::clon(&self.0), Differentiable::clon(&self.1))
    }

    fn random(&self, rng: &mut impl rand::Rng) -> Self {
        (self.0.random(rng), self.1.random(rng))
    }

    fn apply_bounds(&self, other0: &Self) -> Self {
        (
            Differentiable::apply_bounds(&self.0, &other0.0),
            Differentiable::apply_bounds(&self.1, &other0.1),
        )
    }
}

#[derive(Differentiable)]
struct NamedStruct {
    baz: usize,
    var: usize,
}

#[derive(Differentiable)]
struct UnnamedStruct(usize, usize);

#[derive(Differentiable)]
struct UnitStruct;

#[allow(dead_code)]
#[derive(Differentiable, Debug)]
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
