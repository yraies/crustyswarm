use cgmath::{BaseFloat, InnerSpace, Vector3, Zero};
use rand::Rng;

const MINUS_ONE: Vector3<f32> = Vector3 {
    x: -1.0,
    y: -1.0,
    z: -1.0,
};

#[allow(dead_code)]
pub fn safe_normalize<T: BaseFloat>(v: Vector3<T>) -> Vector3<T> {
    if v.is_zero() {
        v
    } else {
        v.normalize()
    }
}

pub fn safe_devide_mean(v: Vector3<f32>, d: f32) -> Vector3<f32> {
    if d == 0.0 {
        v
    } else {
        v / d
    }
}

pub fn random_one(rnd: &mut impl Rng) -> Vector3<f32> {
    (rnd.gen::<Vector3<f32>>() * 2.0) + MINUS_ONE
}

#[allow(dead_code)]
fn svec(v: &Vector3<f32>) -> String {
    let prec = 2;
    format!("({:.p$};{:.p$};{:.p$})", v.x, v.y, v.z, p = prec)
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone, PartialOrd, Ord)]
pub struct Uid(u64);

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
pub struct UidGen {
    last: u64,
}

impl Default for Uid {
    fn default() -> Uid {
        Uid(0)
    }
}

impl UidGen {
    pub fn next(&mut self) -> Uid {
        self.last += 1;
        Uid(self.last)
    }
}

impl Default for UidGen {
    fn default() -> UidGen {
        UidGen { last: 0 }
    }
}
