use super::SpeciesIndex;
use super::Val;
use crate::utils;
use cgmath::Vector3;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Agent {
    pub position: Vector3<Val>,
    pub velocity: Vector3<Val>,
    pub energy: Val,
    pub species_index: SpeciesIndex,
    pub seed_center: Vector3<Val>,
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

impl Agent {
    #[allow(dead_code)]
    fn new(
        position: Vector3<Val>,
        velocity: Vector3<Val>,
        energy: Val,
        species_index: SpeciesIndex,
    ) -> Agent {
        Agent {
            position,
            velocity,
            energy,
            species_index,
            seed_center: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn mk_new(
        position: Vector3<Val>,
        velocity: Vector3<Val>,
        energy: Val,
        species_index: SpeciesIndex,
    ) -> Result<Agent, &'static str> {
        if energy > 0.0 {
            Result::Ok(Agent {
                position,
                velocity,
                energy,
                species_index,
                seed_center: Vector3::new(0.0, 0.0, 0.0),
            })
        } else {
            Result::Err("Energy must be greater than zero")
        }
    }

    pub fn mk_rnd_vel(
        position: Vector3<Val>,
        energy: Val,
        species_index: SpeciesIndex,
        rnd: &mut impl Rng,
    ) -> Result<Agent, &'static str> {
        Agent::mk_new(position, utils::random_one(rnd), energy, species_index)
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Buoy {
    pub position: Vector3<Val>,
    pub y_vel: Val,
    pub base: Val,
}

impl fmt::Debug for Buoy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = &self.position;
        let pos = format!("({:.p$};{:.p$};{:.p$})", v.x, v.y, v.z, p = 2);
        write!(f, "Buoy P{} zV{}", pos, &self.y_vel)
    }
}

impl Buoy {
    #[allow(dead_code)]
    pub fn new(position: Vector3<Val>, y_vel: Val, base: Val) -> Buoy {
        Buoy {
            position,
            y_vel,
            base,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    pub position: Vector3<Val>,
    pub a_type: usize,
}

impl fmt::Debug for Artifact {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = &self.position;
        let pos = format!("({:.p$};{:.p$};{:.p$})", v.x, v.y, v.z, p = 2);
        write!(f, "Artifact P{} T{}", pos, &self.a_type)
    }
}

impl Artifact {
    #[allow(dead_code)]
    fn new(position: Vector3<Val>, a_type: usize) -> Artifact {
        Artifact { position, a_type }
    }
}

pub trait Context {
    fn get_context_id(&self) -> usize;
}

pub trait Position {
    fn get_position(&self) -> Vector3<Val>;
}

impl Position for Agent {
    fn get_position(&self) -> Vector3<Val> {
        self.position
    }
}
impl Position for Artifact {
    fn get_position(&self) -> Vector3<Val> {
        self.position
    }
}
