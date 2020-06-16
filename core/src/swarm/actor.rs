use super::genome::{ArtifactIndex, SpeciesIndex, SurroundingIndex};
use crate::utils;
use crate::utils::Uid;
use cgmath::Vector3;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Actor {
    Agent(Agent),
    Artifact(Artifact),
}

impl Actor {
    pub fn get_id(&self) -> Uid {
        match self {
            Actor::Agent(agent) => agent.id,
            Actor::Artifact(artifact) => artifact.id,
        }
    }
    pub fn get_surrounding_index(&self) -> SurroundingIndex {
        match self {
            Actor::Agent(agent) => agent.species_index.into(),
            Actor::Artifact(artifact) => artifact.artifact_index.into(),
        }
    }
    pub fn get_position(&self) -> Vector3<f32> {
        match self {
            Actor::Agent(agent) => agent.position,
            Actor::Artifact(artifact) => artifact.position,
        }
    }
}

impl Into<SurroundingIndex> for Actor {
    fn into(self) -> SurroundingIndex {
        match self {
            Actor::Agent(agent) => agent.species_index.into(),
            Actor::Artifact(artifact) => artifact.artifact_index.into(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Agent {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub energy: f32,
    pub species_index: SpeciesIndex,
    pub seed_center: Vector3<f32>,
    pub id: Uid,
    pub last: Option<Uid>,
}

impl fmt::Debug for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn fmt_vec(v: &Vector3<f32>, fm: &mut fmt::Formatter) -> String {
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
            "Agent P{} V{} E{} S{:?}",
            pos, vel, self.energy, self.species_index
        )
    }
}

impl Agent {
    #[allow(dead_code)]
    fn new(
        position: Vector3<f32>,
        velocity: Vector3<f32>,
        energy: f32,
        species_index: SpeciesIndex,
        uid: Uid,
    ) -> Agent {
        Agent {
            position,
            velocity,
            energy,
            species_index,
            seed_center: Vector3::new(0.0, 0.0, 0.0),
            last: None,
            id: uid,
        }
    }

    pub fn mk_new(
        position: Vector3<f32>,
        velocity: Vector3<f32>,
        energy: f32,
        species_index: SpeciesIndex,
        seed_center: Vector3<f32>,
        uid: Uid,
    ) -> Result<Agent, &'static str> {
        if energy > 0.0 {
            Result::Ok(Agent {
                position,
                velocity,
                energy,
                species_index,
                seed_center,
                last: None,
                id: uid,
            })
        } else {
            Result::Err("Energy must be greater than zero")
        }
    }

    pub fn mk_rnd_vel(
        position: Vector3<f32>,
        energy: f32,
        species_index: SpeciesIndex,
        rnd: &mut impl Rng,
        seed_center: Vector3<f32>,
        uid: Uid,
    ) -> Result<Agent, &'static str> {
        Agent::mk_new(position, utils::random_one(rnd), energy, species_index,seed_center, uid)
    }
}

impl Into<Actor> for Agent {
    fn into(self) -> Actor {
        Actor::Agent(self)
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Buoy {
    pub position: Vector3<f32>,
    pub y_vel: f32,
    pub base: f32,
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
    pub fn new(position: Vector3<f32>, y_vel: f32, base: f32) -> Buoy {
        Buoy {
            position,
            y_vel,
            base,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    pub position: Vector3<f32>,
    pub artifact_index: ArtifactIndex,
    pub id: Uid,
    pub pre: Option<Uid>,
}

impl fmt::Debug for Artifact {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = &self.position;
        let pos = format!("({:.p$};{:.p$};{:.p$})", v.x, v.y, v.z, p = 2);
        write!(f, "Artifact P{} T{:?}", pos, &self.artifact_index)
    }
}

impl Into<Actor> for Artifact {
    fn into(self) -> Actor {
        Actor::Artifact(self)
    }
}
