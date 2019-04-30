use cgmath::{Vector3, Zero};
use rand::Rng;
use serde::{Deserialize, Serialize};
use swarm::grammar::SwarmGrammar;
use swarm::grammar::SwarmTemplate;

pub mod agent;
pub mod grammar;
pub mod ruleset;
pub mod species;

pub type Val = f32;
type SpeciesIndex = usize;
/*
pub fn new_agent_at_origin(energy: Val, index: SpeciesIndex) -> Result<Agent,&'static str> {
    Agent::mk_new(Vector3::zero(), Vector3::zero(), energy, index)
}*/

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StartDistribution {
    Multi(Vec<StartDistribution>),
    Single(SpeciesIndex),
    Singularity(Vec<(SpeciesIndex, usize)>),
    Plane(f32, f32, SpeciesIndex, usize),
    Grid(usize, SpeciesIndex, f32),
}

impl StartDistribution {
    pub fn apply(&self, template: SwarmTemplate, rnd: &mut impl Rng) -> SwarmGrammar {
        let agents: Vec<agent::Agent> = StartDistribution::genAgents(self, &template, rnd);
        SwarmGrammar {
            agents: agents,
            template,
        }
    }

    fn genAgents(
        startDist: &StartDistribution,
        template: &SwarmTemplate,
        rnd: &mut impl Rng,
    ) -> Vec<agent::Agent> {
        match startDist {
            StartDistribution::Multi(distributions) => {
                let mut agents = vec![];
                for dist in distributions {
                    agents.append(&mut StartDistribution::genAgents(dist, template, rnd));
                }
                agents
            }
            StartDistribution::Single(index) => vec![agent::Agent::mk_new(
                Vector3::<Val>::zero(),
                Vector3::<Val>::zero(),
                10f32,
                *index,
            )
            .unwrap()],
            StartDistribution::Singularity(species) => {
                let mut agents = vec![];

                for (spec, count) in species {
                    for _i in 0..*count {
                        let pos = Vector3::<Val>::zero();
                        agents.push(agent::Agent::mk_rnd_vel(pos, 10f32, *spec, rnd).unwrap());
                    }
                }

                agents
            }
            StartDistribution::Plane(xscale, yscale, spec, count) => {
                let mut agents = vec![];

                for _i in 0..*count {
                    let xpos = rnd.gen_range(-xscale, xscale);
                    let ypos = rnd.gen_range(-yscale, yscale);
                    let rnd_pos = Vector3::<Val>::new(xpos, ypos, 0f32);

                    agents.push(agent::Agent::mk_rnd_vel(rnd_pos, 10f32, *spec, rnd).unwrap());
                }

                agents
            }
            StartDistribution::Grid(count, species, spacing) => {
                let mut gridsize: f32 = (count - 1) as f32 * spacing;
                let mut halfsize: f32 = gridsize / 2.0;

                let mut agents = vec![];

                for x in 0..*count {
                    for z in 0..*count {
                        let xpos = -halfsize + (x as f32) * spacing;
                        let zpos = -halfsize + (z as f32) * spacing;
                        let pos = Vector3::<Val>::new(xpos, 0f32, zpos);

                        agents.push(agent::Agent::mk_rnd_vel(pos, 10f32, *species, rnd).unwrap());
                    }
                }

                agents
            }
        }
    }
}
