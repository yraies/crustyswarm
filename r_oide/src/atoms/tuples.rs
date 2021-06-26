use crate::traits::*;
use rand::prelude::*;

//
// 2-Tuple
//

impl<T: OIDEAdd, U: OIDEAdd> OIDEAdd for (T, U) {
    fn add(&self, other: &Self) -> Self {
        (self.0.add(&other.0), self.1.add(&other.1))
    }
}
impl<T: OIDEDiff, U: OIDEDiff> OIDEDiff for (T, U) {
    fn difference(&self, other: &Self) -> Self {
        (self.0.difference(&other.0), self.1.difference(&other.1))
    }
}
impl<T: OIDEScale, U: OIDEScale> OIDEScale for (T, U) {
    fn scale(&self, factor: f32) -> Self {
        (self.0.scale(factor), self.1.scale(factor))
    }
}
impl<T: OIDEOpposite, U: OIDEOpposite> OIDEOpposite for (T, U) {
    fn opposite(&self, midpoint: Option<&Self>) -> Self {
        (
            self.0.opposite(midpoint.map(|s| &s.0)),
            self.1.opposite(midpoint.map(|s| &s.1)),
        )
    }
}
impl<T: OIDERandomize, U: OIDERandomize> OIDERandomize for (T, U) {
    fn random(&self, rng: &mut impl Rng) -> Self {
        (self.0.random(rng), self.1.random(rng))
    }
}
impl<T: OIDECrossover, U: OIDECrossover> OIDECrossover for (T, U) {
    fn crossover(&self, other: &Self, rng: &mut impl Rng, rate: f64) -> Self {
        (
            self.0.crossover(&other.0, rng, rate),
            self.1.crossover(&other.1, rng, rate),
        )
    }
}
impl<T: OIDEBoundApplication, U: OIDEBoundApplication> OIDEBoundApplication for (T, U) {
    fn apply_bounds(&self, other: &Self) -> Self {
        (self.0.apply_bounds(&other.0), self.1.apply_bounds(&other.1))
    }
}
impl<T: OIDEZero, U: OIDEZero> OIDEZero for (T, U) {
    fn zero(&self) -> Self {
        (self.0.zero(), self.1.zero())
    }
}
impl<T: Differentiable, U: Differentiable> Differentiable for (T, U) {}

//
// 3-Tuple
//

impl<T: OIDEAdd, U: OIDEAdd, V: OIDEAdd> OIDEAdd for (T, U, V) {
    fn add(&self, other: &Self) -> Self {
        (
            self.0.add(&other.0),
            self.1.add(&other.1),
            self.2.add(&other.2),
        )
    }
}
impl<T: OIDEDiff, U: OIDEDiff, V: OIDEDiff> OIDEDiff for (T, U, V) {
    fn difference(&self, other: &Self) -> Self {
        (
            self.0.difference(&other.0),
            self.1.difference(&other.1),
            self.2.difference(&other.2),
        )
    }
}
impl<T: OIDEScale, U: OIDEScale, V: OIDEScale> OIDEScale for (T, U, V) {
    fn scale(&self, factor: f32) -> Self {
        (
            self.0.scale(factor),
            self.1.scale(factor),
            self.2.scale(factor),
        )
    }
}
impl<T: OIDEOpposite, U: OIDEOpposite, V: OIDEOpposite> OIDEOpposite for (T, U, V) {
    fn opposite(&self, midpoint: Option<&Self>) -> Self {
        (
            self.0.opposite(midpoint.map(|s| &s.0)),
            self.1.opposite(midpoint.map(|s| &s.1)),
            self.2.opposite(midpoint.map(|s| &s.2)),
        )
    }
}
impl<T: OIDERandomize, U: OIDERandomize, V: OIDERandomize> OIDERandomize for (T, U, V) {
    fn random(&self, rng: &mut impl Rng) -> Self {
        (self.0.random(rng), self.1.random(rng), self.2.random(rng))
    }
}
impl<T: OIDECrossover, U: OIDECrossover, V: OIDECrossover> OIDECrossover for (T, U, V) {
    fn crossover(&self, other: &Self, rng: &mut impl Rng, rate: f64) -> Self {
        (
            self.0.crossover(&other.0, rng, rate),
            self.1.crossover(&other.1, rng, rate),
            self.2.crossover(&other.2, rng, rate),
        )
    }
}
impl<T: OIDEBoundApplication, U: OIDEBoundApplication, V: OIDEBoundApplication> OIDEBoundApplication
    for (T, U, V)
{
    fn apply_bounds(&self, other: &Self) -> Self {
        (
            self.0.apply_bounds(&other.0),
            self.1.apply_bounds(&other.1),
            self.2.apply_bounds(&other.2),
        )
    }
}
impl<T: OIDEZero, U: OIDEZero, V: OIDEZero> OIDEZero for (T, U, V) {
    fn zero(&self) -> Self {
        (self.0.zero(), self.1.zero(), self.2.zero())
    }
}
impl<T: Differentiable, U: Differentiable, V: Differentiable> Differentiable for (T, U, V) {}
