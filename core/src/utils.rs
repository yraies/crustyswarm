use cgmath::{BaseFloat, InnerSpace, Vector3, Zero};

use rand::Rng;
use swarm::Val;

const MINUS_ONE: Vector3<Val> = Vector3 {
    x: -1.0,
    y: -1.0,
    z: -1.0,
};

pub fn safe_normalize<T: BaseFloat>(v: Vector3<T>) -> Vector3<T> {
    if v.is_zero() {
        v
    } else {
        v.normalize()
    }
}

pub fn safe_devide_mean(v: Vector3<Val>, d: usize) -> Vector3<Val> {
    if d == 0 {
        v
    } else {
        v / (d as Val)
    }
}

pub fn random_one(rnd: &mut impl Rng) -> Vector3<Val> {
    (rnd.gen::<Vector3<Val>>() * 2.0) - MINUS_ONE
}

#[allow(dead_code)]
fn svec(v: &Vector3<Val>) -> String {
    let prec = 2;
    format!("({:.p$};{:.p$};{:.p$})", v.x, v.y, v.z, p = prec)
}
