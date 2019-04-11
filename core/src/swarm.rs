use serde::{Deserialize, Serialize};
use swarm::grammar::SwarmGrammar;
use swarm::grammar::SwarmTemplate;
use cgmath::{Vector3, Zero};
use rand::Rng;

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


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StartDistribution {
    Single(SpeciesIndex),
    Singularity(Vec<(SpeciesIndex,usize)>),
    Plane(f32, f32, Vec<(SpeciesIndex,usize)>)
}

impl StartDistribution {
    pub fn apply(&self, template: SwarmTemplate, rnd : &mut impl Rng) -> SwarmGrammar {
        let agents : Vec<agent::Agent> = match self {
            StartDistribution::Single(index) => {
                vec!(agent::Agent::mk_new(Vector3::<Val>::zero(), Vector3::<Val>::zero(), 10f32, *index).unwrap())
            },
            StartDistribution::Singularity(species) => {
              let mut agents = vec!();

              for (spec, count) in species {
                  for _i in 0..*count {
                      let pos = Vector3::<Val>::zero();
                      agents.push(
                          agent::Agent::mk_rnd_vel(pos , 10f32, *spec, rnd).unwrap()
                      );
                  }
              };

              agents
            },
            StartDistribution::Plane(xscale,yscale,species) => {
              let mut agents = vec!();

              for (spec, count) in species {
                  for _i in 0..*count {
                      let xpos  = rnd.gen_range(-xscale,xscale);
                      let ypos  = rnd.gen_range(-yscale,yscale);
                      let rnd_pos = Vector3::<Val>::new(xpos, ypos, 0f32);

                      agents.push(
                          agent::Agent::mk_rnd_vel(rnd_pos, 10f32, *spec, rnd).unwrap()
                      );
                  }
              };

              agents
            },
        };
        SwarmGrammar { agents: agents, template }
    }
}
