use std::fmt;

use cgmath::{Deg, Euler, Quaternion};
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

use super::actor::Agent;
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
    Spread(SpeciesIndex, usize),
}

impl RuleSet {
    pub fn execute(&self, agents: &[Agent], rnd: &mut impl Rng) -> Vec<Agent> {
        let mut new_vec = Vec::new();

        for agent in agents {
            if agent.species_index == self.input {
                let thresh = rnd.gen();
                let mut prob_counter: Val = 0.0;

                for r in &self.rules {
                    prob_counter += r.0;
                    if prob_counter > thresh {
                        r.1.replace_to(agent, &mut new_vec);
                        break;
                    }
                }
            }
        }

        new_vec
    }
}

impl Replacement {
    pub fn replace_to(&self, agent: &Agent, new_vec: &mut Vec<Agent>) {
        match self {
            Replacement::Simple(specs) => {
                for spec in specs.iter() {
                    let mut clone = dbg!(agent.clone());
                    clone.species_index = *spec;
                    new_vec.push(clone);
                }
            }
            Replacement::Multi(repls) => {
                for repl in repls.iter() {
                    repl.replace_to(agent, new_vec);
                }
            }
            Replacement::Spread(index, count) => {
                let rot = Quaternion::from(Euler {
                    x: Deg(0.0),
                    y: Deg((360f32 / (*count as f32)) % 180.0),
                    z: Deg(0.0),
                });
                let base_rot = Quaternion::from(Euler {
                    x: Deg(0.0),
                    y: Deg(90.0),
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
        }
    }
}
