use super::Val;
use super::SpeciesIndex;
use cgmath::Vector3;
use std::fmt;
use serde::Deserialize;
use serde::Serialize;
use crate::utils;
use rand::Rng;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Agent {
    pub position: Vector3<Val>,
    pub velocity: Vector3<Val>,
    pub energy: Val,
    pub species_index: SpeciesIndex,
}

impl fmt::Debug for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn fmt_vec(v: &Vector3<Val>, fm: &mut fmt::Formatter) -> String {
            if let Some(prec) = fm.precision() {
                format!("({:.p$};{:.p$};{:.p$})", v.x, v.y, v.z, p = prec)
            } else {
                format!("({};{};{})", v.x, v.y, v.z)
            }
        }

        let pos = fmt_vec(&self.position, f);
        let vel = fmt_vec(&self.velocity, f);

        write!(
            f,
            "Agent P{} V{} E{} S{}",
            pos, vel, self.energy, self.species_index
        )
    }
}



impl Agent{
    #[allow(dead_code)]
    fn new(position: Vector3<Val>, velocity: Vector3<Val>, energy: Val, species_index: SpeciesIndex) -> Agent {
        Agent{position, velocity, energy, species_index}
    }

    pub fn mk_new(position: Vector3<Val>, velocity: Vector3<Val>, energy: Val, species_index: SpeciesIndex) -> Result<Agent,&'static str> {
        if energy > 0.0 {
            Result::Ok(Agent{position, velocity, energy, species_index})
        } else {
            Result::Err("Energy must be greater than zero")
        }
    }

    pub fn mk_rnd_vel(position: Vector3<Val>, energy: Val, species_index: SpeciesIndex, rnd : &mut impl Rng) -> Result<Agent, &'static str> {
        Agent::mk_new(position ,utils::random_one(rnd), energy, species_index)
    }
}
