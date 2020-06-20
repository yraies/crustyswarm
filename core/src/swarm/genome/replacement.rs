use super::{Factor, SpeciesIndex, SurroundingIndex, SwarmGenome, ZeroEnergy};
use crate::utils::UidGen;
use serde::{Deserialize, Serialize};
use swarm::actor::{Agent, Artifact};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub(crate) struct ContextRule {
    pub(super) context: Vec<SurroundingIndex>,
    pub range: f32,
    pub weight: Factor,
    pub persist: bool,
    pub(super) replacement: Replacement,
}

impl ContextRule {
    pub fn is_applicable(&self, context: &Vec<(f32, SurroundingIndex)>) -> bool {
        if self.context.is_empty() {
            return true;
        }

        let mut checkset = self.context.clone();
        for (dist, surrounding) in context {
            if dist < &self.range {
                if let Some(foundthing) = checkset.iter().position(|a| surrounding.eq(a)) {
                    checkset.remove(foundthing);
                }
            }
        }

        checkset.is_empty()
    }

    pub fn replace_agent(
        &self,
        parent: &Agent,
        genome: &SwarmGenome,
        uid_gen: &mut UidGen,
    ) -> (Vec<Agent>, Vec<Artifact>) {
        let (ags, arts) = self.replacement.replace_agent(parent, genome, uid_gen);
        (ags, arts)
    }
}

impl Default for ContextRule {
    fn default() -> ContextRule {
        ContextRule {
            context: Vec::new(),
            weight: 1.0,
            persist: true,
            replacement: Replacement::None,
            range: 5.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) enum Replacement {
    None,
    Simple(Vec<SurroundingIndex>),
    Multi(Vec<Replacement>),
    Spread(SpeciesIndex, usize, usize),
}

impl Replacement {
    pub fn replace_agent(
        &self,
        parent: &Agent,
        genome: &SwarmGenome,
        uid_gen: &mut UidGen,
    ) -> (Vec<Agent>, Vec<Artifact>) {
        let parent_species = &genome.species_map[parent.species_index.0];

        if parent.energy < 0.0 && parent_species.zero_energy == (ZeroEnergy::Die) {
            return (vec![], vec![]);
        }

        let mut new_agents: Vec<Agent> = vec![];
        let mut new_artifacts: Vec<Artifact> = vec![];

        match self {
            Replacement::None => {}
            Replacement::Simple(new_indices) => {
                for index in new_indices.iter() {
                    match index {
                        SurroundingIndex::Agent(new_species_index) => {
                            let new_agent =
                                parent_species.generate_agent(parent, *new_species_index);
                            new_agents.push(new_agent);
                        }
                        SurroundingIndex::Artifact(new_type_index) => {
                            let new_artifact = Artifact {
                                artifact_index: *new_type_index,
                                id: uid_gen.next(),
                                position: parent.position,
                                pre: parent.last,
                            };
                            new_artifacts.push(new_artifact);
                        }
                    }
                }
            }
            Replacement::Multi(repls) => {
                for repl in repls.iter() {
                    let (mut ags, mut arts) = repl.replace_agent(parent, genome, uid_gen);
                    new_agents.append(&mut ags);
                    new_artifacts.append(&mut arts);
                }
            }
            Replacement::Spread(new_species_index, count, offset) => {
                use cgmath::{Deg, Euler, Quaternion};

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
                let mut new_vel = base_rot * parent.velocity;

                for _i in 0..*count {
                    let mut new_agent = parent_species.generate_agent(parent, *new_species_index);
                    new_agent.velocity = new_vel;
                    new_agents.push(new_agent);

                    new_vel = rot * new_vel;
                }
            }
        };
        (new_agents, new_artifacts)
    }
}

impl Default for Replacement {
    fn default() -> Replacement {
        Replacement::None
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct ApplicationStrategy {
    every: usize,
    offset: usize,
}

impl ApplicationStrategy {
    pub fn tick(&mut self) {
        if self.offset >= 1 {
            self.offset -= 1;
        } else {
            self.offset = self.every;
        }
    }
    pub fn should_replace(&self) -> bool {
        self.offset == 0
    }
}

impl From<super::dummies::DummyApplicationStrategy> for ApplicationStrategy {
    fn from(dummy: super::dummies::DummyApplicationStrategy) -> ApplicationStrategy {
        ApplicationStrategy {
            every: dummy.every,
            offset: dummy.offset.unwrap_or_else(|| dummy.every),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OffspringEnergy {
    Constant(f32),
    Inherit(f32),
}

impl OffspringEnergy {
    pub fn get(&self, current: f32) -> f32 {
        match self {
            OffspringEnergy::Constant(value) => *value,
            OffspringEnergy::Inherit(factor) => current * factor,
        }
    }
}

impl Default for OffspringEnergy {
    fn default() -> OffspringEnergy {
        OffspringEnergy::Inherit(1.0)
    }
}
