use super::{Factor, Species, SpeciesIndex, SurroundingIndex, SwarmGenome};
use crate::swarm::actor::{Agent, Artifact};
use crate::utils::UidGen;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ContextRule {
    pub context: Vec<SurroundingIndex>,
    pub range: f32,
    pub weight: Factor,
    pub persist: bool,
    pub replacement: Replacement,
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
        let (ags, arts) = self
            .replacement
            .replace_agent(parent, genome, uid_gen, self.persist);
        (ags, arts)
    }
}

impl Default for ContextRule {
    fn default() -> ContextRule {
        ContextRule {
            context: Vec::new(),
            weight: 1.0,
            persist: false,
            replacement: Replacement::None,
            range: 5.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Replacement {
    None,
    Simple(Vec<SurroundingIndex>),
    Multi(Vec<Replacement>),
    Spread(SpeciesIndex, usize, usize),
}

impl Replacement {
    pub fn count_replacements(&self) -> usize {
        match self {
            Replacement::None => 0,
            Replacement::Simple(new_indices) => new_indices.len(),
            Replacement::Multi(repls) => repls.iter().map(|o| o.count_replacements()).sum(),
            Replacement::Spread(_, count, _) => *count,
        }
    }

    fn generate_agent(
        parent: &Agent,
        new_index: SpeciesIndex,
        new_energy: f32,
        hand_down_seed: bool,
        uid_gen: &mut UidGen,
    ) -> Agent {
        let mut clone = parent.clone();
        clone.species_index = new_index;
        clone.energy = new_energy;
        if hand_down_seed {
            clone.seed_center = parent.position;
        }
        clone.id = uid_gen.next();
        clone.iteration += 1;
        clone
    }

    pub fn replace_agent_unchecked(
        &self,
        parent: &Agent,
        parent_species: &Species,
        energy: f32,
        uid_gen: &mut UidGen,
    ) -> (Vec<Agent>, Vec<Artifact>) {
        let mut new_agents: Vec<Agent> = vec![];
        let mut new_artifacts: Vec<Artifact> = vec![];

        match self {
            Replacement::None => {}
            Replacement::Simple(new_indices) => {
                for index in new_indices.iter() {
                    match index {
                        SurroundingIndex::Agent(new_species_index) => {
                            let new_agent = Self::generate_agent(
                                parent,
                                *new_species_index,
                                energy,
                                parent_species.hand_down_seed,
                                uid_gen,
                            );
                            new_agents.push(new_agent);
                        }
                        SurroundingIndex::Artifact(new_type_index) => {
                            let new_artifact = Artifact {
                                artifact_index: *new_type_index,
                                id: uid_gen.next(),
                                position: parent.position,
                                pre: parent.last,
                                iteration: parent.iteration + 1,
                                energy: energy,
                            };
                            new_artifacts.push(new_artifact);
                        }
                    }
                }
            }
            Replacement::Multi(repls) => {
                for repl in repls.iter() {
                    let (mut ags, mut arts) =
                        repl.replace_agent_unchecked(parent, parent_species, energy, uid_gen);
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
                    let mut new_agent = Self::generate_agent(
                        parent,
                        *new_species_index,
                        energy,
                        parent_species.hand_down_seed,
                        uid_gen,
                    );
                    new_agent.velocity = new_vel;
                    new_agents.push(new_agent);

                    new_vel = rot * new_vel;
                }
            }
        };
        (new_agents, new_artifacts)
    }

    pub fn replace_agent(
        &self,
        parent: &Agent,
        genome: &SwarmGenome,
        uid_gen: &mut UidGen,
        persist: bool,
    ) -> (Vec<Agent>, Vec<Artifact>) {
        let parent_species = &genome.species_map[parent.species_index.0];

        if !parent_species.energy.on_zero.is_alive(parent.energy) {
            return parent_species
                .energy
                .on_zero
                .replacement(parent, parent_species, uid_gen);
        }

        let per_offspring_energy = parent_species.energy.for_offspring.get(
            parent.energy,
            self.count_replacements(),
            persist,
        );
        let new_parent_energy = parent_species.energy.on_replication.get(
            parent.energy,
            self.count_replacements(),
            per_offspring_energy,
        );

        let (mut new_agents, new_artifacts) =
            self.replace_agent_unchecked(parent, parent_species, per_offspring_energy, uid_gen);

        if persist {
            let mut new_parent = parent.clone();
            new_parent.energy = new_parent_energy;
            if new_artifacts.len() > 0 {
                new_parent.last = Some(new_artifacts[0].id);

                new_agents.iter_mut().for_each(|a| a.last = new_parent.last);
            }
            new_parent.iteration += 1;
            new_agents.push(new_parent);
        }

        (new_agents, new_artifacts)
    }
}

impl Default for Replacement {
    fn default() -> Replacement {
        Replacement::None
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct ApplicationStrategy {
    pub every: usize,
    pub offset: usize,
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
