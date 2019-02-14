/*use swarm::agent::Agent;

use cgmath::prelude::*;
use cgmath::Vector3;*/

pub mod agent;
pub mod ruleset;
pub mod species;
pub mod grammar;

pub type Val = f32;
type SpeciesIndex = usize;
/*
pub fn new_agent_at_origin(energy: Val, index: SpeciesIndex) -> Result<Agent,&'static str> {
    Agent::mk_new(Vector3::zero(), Vector3::zero(), energy, index)
}*/