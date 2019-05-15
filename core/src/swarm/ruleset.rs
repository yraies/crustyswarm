use cgmath::{Deg, Euler, Quaternion};
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

use super::actor::{Agent, Artifact, Buoy};
use super::grammar::SwarmTemplate;
use super::species::ZeroEnergy;
use super::SpeciesIndex;
use super::Val;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum RuleStrategy {
    Every(usize, usize),
}

impl RuleStrategy {
    pub fn should_replace(&mut self) -> bool {
        match self {
            RuleStrategy::Every(max, ref mut curr) => {
                if *curr > 1 {
                    *curr -= 1;
                    false
                } else {
                    *curr = *max;
                    true
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleSet {
    pub input: SpeciesIndex,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rule(pub Val, pub Replacement);

#[derive(Debug, Serialize, Deserialize)]
pub enum Replacement {
    Simple(Vec<SpeciesIndex>),
    Multi(Vec<Replacement>),
    Spread(SpeciesIndex, usize, usize),
    Buoy,
}

impl RuleSet {
    pub fn execute(
        &self,
        template: &SwarmTemplate,
        agents: &[Agent],
        rnd: &mut impl Rng,
    ) -> (Vec<Agent>, Vec<Buoy>) {
        let mut new_agents = Vec::new();
        let mut new_buoys = Vec::new();

        for agent in agents {
            if agent.species_index == self.input {
                let thresh = rnd.gen();
                let mut prob_counter: Val = 0.0;

                for r in &self.rules {
                    prob_counter += r.0;
                    if prob_counter > thresh {
                        r.1.replace_to(agent, template, &mut new_agents, &mut new_buoys);
                        break;
                    }
                }
            }
        }

        (new_agents, new_buoys)
    }
}

impl Replacement {
    pub fn replace_to(
        &self,
        agent: &Agent,
        template: &SwarmTemplate,
        new_vec: &mut Vec<Agent>,
        new_buoys: &mut Vec<Buoy>,
    ) {
        if agent.energy < 0.0
            && template.species[agent.species_index].zero_energy == (ZeroEnergy::Die)
        {
            return;
        }

        match self {
            Replacement::Simple(specs) => {
                for spec in specs.iter() {
                    let mut clone = agent.clone();
                    clone.species_index = *spec;
                    clone.energy =
                        template.species[clone.species_index].get_spawn_energy(agent.energy);
                    new_vec.push(clone);
                }
            }
            Replacement::Multi(repls) => {
                for repl in repls.iter() {
                    repl.replace_to(agent, template, new_vec, new_buoys);
                }
            }
            Replacement::Spread(index, count, offset) => {
                let rot = Quaternion::from(Euler {
                    x: Deg(0.0),
                    y: Deg(360f32 / (*count as f32)),
                    z: Deg(0.0),
                });
                let base_rot = Quaternion::from(Euler {
                    x: Deg(0.0),
                    y: Deg(*offset as f32),
                    z: Deg(0.0),
                });
                let mut new_vel = base_rot * agent.velocity;

                for _i in 0..*count {
                    let mut clone = agent.clone();
                    clone.species_index = *index;
                    clone.velocity = new_vel;
                    new_vec.push(clone);

                    new_vel = rot * new_vel;
                }
            }
            Replacement::Buoy => new_buoys.push(Buoy {
                position: agent.position.clone(),
                y_vel: 0.0,
            }),
        }
    }
}
