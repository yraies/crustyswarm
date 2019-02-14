use cgmath::{Vector3, BaseFloat, InnerSpace, Zero};
use swarm::Val;

pub fn safe_normalize<T: BaseFloat>(v: Vector3<T>) -> Vector3<T> {
    if v.is_zero() { v } else { v.normalize() }
}

pub fn safe_devide_mean(v: Vector3<Val>, d: usize) -> Vector3<Val> {
    if d == 0 { v } else { v / (d as Val) }
}

#[allow(dead_code)]
fn svec(v: &Vector3<Val>) -> String {
    let prec = 2;
    format!("({:.p$};{:.p$};{:.p$})", v.x, v.y, v.z, p = prec)
}