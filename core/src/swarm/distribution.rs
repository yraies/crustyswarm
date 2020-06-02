use super::SpeciesIndex;
use super::Val;
use cgmath::Vector3;
use cgmath::Zero;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

use super::actor::{Agent, Buoy};
use super::grammar::{SwarmGrammar, SwarmTemplate};
use super::world::{ChunkedWorld, World};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StartDistribution {
    pub start_agents: StartAgents,
    pub start_buoys: StartBuoys,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StartAgents {
    Multi(Vec<StartAgents>),
    Single(f32, f32, SpeciesIndex),
    Singularity(Vec<(usize, SpeciesIndex)>),
    Plane(f32, f32, usize, SpeciesIndex),
    Grid(usize, f32, SpeciesIndex),
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StartBuoys {
    Multi(Vec<StartBuoys>),
    Single(f32, f32),
    Plane(f32, f32, usize),
    Grid(usize, f32),
    None,
}

impl StartDistribution {
    pub fn apply(&self, template: SwarmTemplate, rnd: &mut impl Rng) -> SwarmGrammar {
        let agents: Vec<Agent> = StartDistribution::gen_agents(&self.start_agents, &template, rnd);
        let buoys: Vec<Buoy> = StartDistribution::gen_buoys(&self.start_buoys, &template, rnd);
        let mut world = ChunkedWorld::new(agents, 10.0);
        world.set_buoys(buoys);
        SwarmGrammar { world, template }
    }

    fn gen_agents(dist: &StartAgents, template: &SwarmTemplate, rnd: &mut impl Rng) -> Vec<Agent> {
        match dist {
            StartAgents::None => Vec::new(),
            StartAgents::Multi(distributions) => {
                let mut agents = vec![];
                for di in distributions {
                    agents.append(&mut StartDistribution::gen_agents(di, template, rnd));
                }
                agents
            }
            StartAgents::Single(x, z, index) => vec![Agent::mk_new(
                Vector3::<Val>::new(*x, 0.0, *z),
                Vector3::<Val>::zero(),
                20f32,
                *index,
            )
            .unwrap()],
            StartAgents::Singularity(species) => {
                let mut agents = vec![];

                for (count, spec) in species {
                    for _i in 0..*count {
                        let pos = Vector3::<Val>::zero();
                        agents.push(Agent::mk_rnd_vel(pos, 20f32, *spec, rnd).unwrap());
                    }
                }

                agents
            }
            StartAgents::Plane(xscale, yscale, count, spec) => {
                let mut agents = vec![];

                for _i in 0..*count {
                    let xpos = rnd.gen_range(-xscale, xscale);
                    let ypos = rnd.gen_range(-yscale, yscale);
                    let rnd_pos = Vector3::<Val>::new(xpos, ypos, 0f32);

                    agents.push(Agent::mk_rnd_vel(rnd_pos, 20f32, *spec, rnd).unwrap());
                }

                agents
            }
            StartAgents::Grid(count, spacing, species) => {
                let gridsize: f32 = (count - 1) as f32 * spacing;
                let halfsize: f32 = gridsize / 2.0;

                let mut agents = vec![];

                for x in 0..*count {
                    for z in 0..*count {
                        let xpos = -halfsize + (x as f32) * spacing;
                        let zpos = -halfsize + (z as f32) * spacing;
                        let pos = Vector3::<Val>::new(xpos, 0f32, zpos);

                        agents.push(Agent::mk_rnd_vel(pos, 20f32, *species, rnd).unwrap());
                    }
                }

                agents
            }
        }
    }

    fn gen_buoys(dist: &StartBuoys, template: &SwarmTemplate, rnd: &mut impl Rng) -> Vec<Buoy> {
        match dist {
            StartBuoys::Multi(distributions) => {
                let mut buoys = vec![];
                for di in distributions {
                    buoys.append(&mut StartDistribution::gen_buoys(di, template, rnd));
                }
                buoys
            }
            StartBuoys::Single(x, z) => vec![Buoy::new(Vector3::<Val>::new(*x, 0.0, *z), 0.0, 0.0)],
            StartBuoys::Plane(xscale, yscale, count) => {
                let mut buoys = vec![];

                for _i in 0..*count {
                    let xpos = rnd.gen_range(-xscale, xscale);
                    let ypos = rnd.gen_range(-yscale, yscale);
                    let rnd_pos = Vector3::<Val>::new(xpos, ypos, 0f32);

                    buoys.push(Buoy::new(rnd_pos, 0.0, 0.0));
                }

                buoys
            }
            StartBuoys::Grid(count, spacing) => {
                let gridsize: f32 = (count - 1) as f32 * spacing;
                let halfsize: f32 = gridsize / 2.0;

                let mut buoys = vec![];

                for x in 0..*count {
                    for z in 0..*count {
                        let xpos = -halfsize + (x as f32) * spacing;
                        let zpos = -halfsize + (z as f32) * spacing;
                        let pos = Vector3::<Val>::new(xpos, 0f32, zpos);

                        buoys.push(Buoy::new(pos, 0.0, 0.0));
                    }
                }

                buoys
            }
            StartBuoys::None => vec![],
        }
    }
}
