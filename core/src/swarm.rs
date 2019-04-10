use serde::{Deserialize, Serialize};
use swarm::grammar::SwarmGrammar;
use swarm::grammar::SwarmTemplate;

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


#[derive(Serialize, Deserialize)]
pub enum StartDistribution {
    Single(SpeciesIndex),
    Singularity(Vec<(SpeciesIndex,usize)>),
    Plane(f32, f32, usize)
}

trait Expandable {
    fn apply(template : SwarmTemplate) -> SwarmGrammar;
}

impl Expandable for StartDistribution {
    fn apply(template: SwarmTemplate) -> SwarmGrammar {
        unimplemented!()
    }
}