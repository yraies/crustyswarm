use cgmath::Vector3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;

use super::dummies::*;

use crate::utils::{Uid, UidGen};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
pub enum SurroundingIndex {
    Agent(SpeciesIndex),
    Artifact(ArtifactIndex),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
#[serde(transparent)]
pub struct SpeciesIndex(pub(crate) usize);

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
#[serde(transparent)]
pub struct ArtifactIndex(pub(crate) usize);

impl Into<SurroundingIndex> for SpeciesIndex {
    fn into(self) -> SurroundingIndex {
        SurroundingIndex::Agent(self)
    }
}
impl Into<SurroundingIndex> for ArtifactIndex {
    fn into(self) -> SurroundingIndex {
        SurroundingIndex::Artifact(self)
    }
}

type Factor = f32;
type InfluenceFactor = Factor;

#[derive(Debug, Serialize, Deserialize)]
#[serde(try_from = "DummySwarmGenome")]
pub struct SwarmGenome {
    species_map: Vec<Species>,
    artifact_map: Vec<ArtifactType>,
    start_dist: Distribution,
    pub strategy: ApplicationStrategy,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ArtifactType {
    pub color_index: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Species {
    index: SpeciesIndex,
    pub separation: Factor,
    pub alignment: Factor,
    pub cohesion: Factor,
    pub randomness: Factor,
    pub center: Factor,
    pub max_speed: Factor,
    pub view_distance: Factor,
    pub sep_distance: Factor,
    pub axis_constraint: Vector3<Factor>,
    pub influenced_by: HashMap<SurroundingIndex, InfluenceFactor>,
    pub mass: Factor,
    pub noclip: bool,
    pub offspring_energy: OffspringEnergy,
    pub depletion_energy: DepletionEnergy,
    pub zero_energy: ZeroEnergy,
    pub hand_down_seed: bool,
    rules: Vec<ContextRule>,
    pub color_index: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub(crate) struct ContextRule {
    context: Vec<SurroundingIndex>,
    pub range: f32,
    pub weight: Factor,
    pub persist: bool,
    replacement: Replacement,
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
enum Replacement {
    None,
    Simple(Vec<SurroundingIndex>),
    Multi(Vec<Replacement>),
    Spread(SpeciesIndex, usize, usize),
}

use swarm::actor::{Agent, Artifact};

impl Species {
    fn generate_agent(&self, parent: &Agent, new_index: SpeciesIndex) -> Agent {
        let mut clone = parent.clone();
        clone.species_index = new_index;
        clone.energy = self.offspring_energy.get(parent.energy);
        if self.hand_down_seed {
            clone.seed_center = parent.position;
        }
        clone
    }
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

#[derive(Debug, Serialize, Deserialize)]
enum Distribution {
    Multi(Vec<Distribution>),
    Single(Vector3<f32>, SurroundingIndex),
    Singularity(Vector3<f32>, Vec<(usize, SurroundingIndex)>),
    Grid(usize, f32, SurroundingIndex),
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

impl From<DummyApplicationStrategy> for ApplicationStrategy {
    fn from(dummy: DummyApplicationStrategy) -> ApplicationStrategy {
        ApplicationStrategy {
            every: dummy.every,
            offset: dummy.offset.unwrap_or_else(|| dummy.every),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DepletionEnergy {
    Constant(f32),
    Distance(f32),
    None,
}

impl DepletionEnergy {
    pub fn get(&self, velocity: f32) -> f32 {
        match self {
            DepletionEnergy::Constant(value) => *value,
            DepletionEnergy::Distance(factor) => velocity * factor,
            DepletionEnergy::None => 0.0,
        }
    }
}

impl Default for DepletionEnergy {
    fn default() -> DepletionEnergy {
        DepletionEnergy::Constant(1.0)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OffspringEnergy {
    Constant(f32),
    Inherit(f32),
}

impl OffspringEnergy {
    fn get(&self, current: f32) -> f32 {
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ZeroEnergy {
    Die,
    Alive,
}

impl Default for ZeroEnergy {
    fn default() -> ZeroEnergy {
        ZeroEnergy::Die
    }
}

impl SwarmGenome {
    pub(crate) fn get_rules(&self, species_index: &SpeciesIndex) -> &Vec<ContextRule> {
        &self.species_map[species_index.0].rules
    }

    pub fn get_species(&self, agent: &Agent) -> &Species {
        &self.species_map[agent.species_index.0]
    }

    pub fn get_artifact_type(&self, artifact: &Artifact) -> &ArtifactType {
        &self.artifact_map[artifact.artifact_index.0]
    }

    pub fn get_start(
        &self,
        rnd: &mut impl rand::Rng,
        uid_gen: &mut UidGen,
    ) -> (Vec<Agent>, Vec<Artifact>) {
        self.distribute(&self.start_dist, rnd, uid_gen)
    }

    pub fn tick(&mut self) {
        self.strategy.tick();
    }

    fn distribute(
        &self,
        distribution: &Distribution,
        rnd: &mut impl rand::Rng,
        uid_gen: &mut UidGen,
    ) -> (Vec<Agent>, Vec<Artifact>) {
        let mut agents = vec![];
        let mut artifacts = vec![];

        match distribution {
            Distribution::Single(pos, surrounding) => {
                SwarmGenome::push(
                    *surrounding,
                    *pos,
                    &mut agents,
                    &mut artifacts,
                    uid_gen.next(),
                );
            }
            Distribution::Singularity(pos, surroundings) => {
                surroundings.iter().for_each(|(count, surrounding)| {
                    for _i in 0..*count {
                        SwarmGenome::push(
                            *surrounding,
                            *pos,
                            &mut agents,
                            &mut artifacts,
                            uid_gen.next(),
                        );
                    }
                });
            }
            Distribution::Grid(count, spacing, surrounding) => {
                let gridsize: f32 = (count - 1) as f32 * spacing;
                let halfsize: f32 = gridsize / 2.0;

                for x in 0..*count {
                    for z in 0..*count {
                        let xpos = -halfsize + (x as f32) * spacing;
                        let zpos = -halfsize + (z as f32) * spacing;
                        let pos = Vector3::<f32>::new(xpos, 0f32, zpos);

                        SwarmGenome::push(
                            *surrounding,
                            pos,
                            &mut agents,
                            &mut artifacts,
                            uid_gen.next(),
                        );
                    }
                }
            }
            Distribution::Multi(distributions) => {
                distributions
                    .iter()
                    .map(|dist| self.distribute(dist, rnd, uid_gen))
                    .for_each(|(mut other_agents, mut other_artifacts)| {
                        agents.append(&mut other_agents);
                        artifacts.append(&mut other_artifacts);
                    });
            }
        }

        (agents, artifacts)
    }

    fn push(
        surr: SurroundingIndex,
        pos: Vector3<f32>,
        agents: &mut Vec<Agent>,
        artifacts: &mut Vec<Artifact>,
        uid: Uid,
    ) {
        match surr {
            SurroundingIndex::Agent(species_index) => agents.push(
                Agent::mk_new(pos, Vector3::new(0.0, 0.0, 0.0), 10.0, species_index, pos, uid).unwrap(),
            ),
            SurroundingIndex::Artifact(artifact_index) => artifacts.push(Artifact {
                pre: None,
                id: uid,
                position: pos,
                artifact_index,
            }),
        }
    }
}

impl TryFrom<DummySwarmGenome> for SwarmGenome {
    type Error = String;
    fn try_from(dummy: DummySwarmGenome) -> Result<SwarmGenome, Self::Error> {
        type M = HashMap<String, usize>;
        fn convert_identifier(
            specs: &M,
            arts: &M,
            identifier: String,
        ) -> Result<SurroundingIndex, String> {
            if let Some(id) = specs.get(&identifier).or(None) {
                Ok(SurroundingIndex::Agent(SpeciesIndex(*id)))
            } else if let Some(id) = arts.get(&identifier).or(None) {
                Ok(SurroundingIndex::Artifact(ArtifactIndex(*id)))
            } else {
                Err(format!("Identifier {} not defined!", identifier))
            }
        }

        fn convert_replacement(
            specs: &M,
            arts: &M,
            dummy_replacement: &DummyReplacement,
        ) -> Result<Replacement, String> {
            match dummy_replacement {
                DummyReplacement::None => Ok(Replacement::None),
                DummyReplacement::Simple(idents) => {
                    let replacements: Result<Vec<SurroundingIndex>, String> = idents
                        .iter()
                        .map(|ident| convert_identifier(specs, arts, ident.0.to_owned()))
                        .collect();
                    Ok(Replacement::Simple(replacements?))
                }
                DummyReplacement::Multi(reps) => Ok(Replacement::Multi(
                    reps.iter()
                        .map(|rep| convert_replacement(specs, arts, rep))
                        .collect::<Result<Vec<_>, _>>()?,
                )),
                DummyReplacement::Spread(ident, count, offset) => {
                    match convert_identifier(specs, arts, ident.0.to_owned())? {
                        SurroundingIndex::Artifact(_) => {
                            Err(format!("Artifact {} is not supported in Spead!", ident.0))
                        }
                        SurroundingIndex::Agent(index) => {
                            Ok(Replacement::Spread(index, *count, *offset))
                        }
                    }
                }
            }
        }

        fn convert_distribution(
            specs: &M,
            arts: &M,
            dummy_distribution: &DummyDistribution,
        ) -> Result<Distribution, String> {
            match dummy_distribution {
                DummyDistribution::Grid(count, space, ident) => Ok(Distribution::Grid(
                    *count,
                    *space,
                    convert_identifier(specs, arts, ident.0.to_owned())?,
                )),
                DummyDistribution::Multi(dists) => Ok(Distribution::Multi(
                    dists
                        .iter()
                        .map(|dist| convert_distribution(specs, arts, dist))
                        .collect::<Result<Vec<_>, _>>()?,
                )),
                DummyDistribution::Single(pos, ident) => Ok(Distribution::Single(
                    Vector3::from(*pos),
                    convert_identifier(specs, arts, ident.0.to_owned())?,
                )),
                DummyDistribution::Singularity(pos, dists) => Ok(Distribution::Singularity(
                    Vector3::from(*pos),
                    dists
                        .iter()
                        .map(|(count, ident)| {
                            convert_identifier(specs, arts, ident.0.to_owned()).map(|r| (*count, r))
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                )),
            }
        }

        let species_names: M = dummy
            .species_map
            .keys()
            .enumerate()
            .map(|(a, b)| (b.to_owned(), a))
            .collect();
        let artifact_names: M = dummy
            .artifact_map
            .keys()
            .enumerate()
            .map(|(a, b)| (b.to_owned(), a))
            .collect();

        let mut species_results: Vec<Result<Species, Self::Error>> =
            vec![Err("No species initialized".to_string()); species_names.len()];

        for (name, id) in &species_names {
            let dummy_spec = dummy.species_map.get(name).unwrap();
            let influences = dummy_spec
                .influence
                .iter()
                .map(|(identifier, factor)| {
                    convert_identifier(&species_names, &artifact_names, identifier.0.to_owned())
                        .map(|index| (index, *factor))
                })
                .collect::<Result<HashMap<SurroundingIndex, InfluenceFactor>, Self::Error>>()?;

            let rules = dummy_spec
                .rules
                .iter()
                .map(|dummy_rule| {
                    let context_results: Result<Vec<SurroundingIndex>, Self::Error> = dummy_rule
                        .context
                        .iter()
                        .map(|identifier| {
                            convert_identifier(
                                &species_names,
                                &artifact_names,
                                identifier.0.to_owned(),
                            )
                        })
                        .collect();
                    let context = context_results?;

                    let replacement_result = convert_replacement(
                        &species_names,
                        &artifact_names,
                        &dummy_rule.replacement,
                    );
                    let replacement = replacement_result?;

                    let rule = ContextRule {
                        persist: dummy_rule.persist,
                        range: dummy_rule.range,
                        weight: dummy_rule.weight,
                        context,
                        replacement,
                    };
                    Ok(rule)
                })
                .collect::<Result<Vec<ContextRule>, Self::Error>>()?;

            let species = Species {
                alignment: dummy_spec.alignment,
                axis_constraint: Vector3::from(dummy_spec.axis_constraint),
                center: dummy_spec.center,
                cohesion: dummy_spec.cohesion,
                depletion_energy: dummy_spec.depletion_energy.clone(),
                hand_down_seed: dummy_spec.hand_down_seed,
                index: SpeciesIndex(*id),
                influenced_by: influences,
                offspring_energy: dummy_spec.offspring_energy.clone(),
                mass: dummy_spec.mass,
                max_speed: dummy_spec.max_speed,
                noclip: dummy_spec.noclip,
                randomness: dummy_spec.randomness,
                rules,
                sep_distance: dummy_spec.sep_distance,
                separation: dummy_spec.separation,
                view_distance: dummy_spec.view_distance,
                zero_energy: dummy_spec.zero_energy.clone(),
                color_index: dummy_spec.color_index,
            };

            species_results[*id] = Ok(species);
        }

        let mut artifact_results: Vec<Result<ArtifactType, Self::Error>> =
            vec![Err("No artifact initialized".to_string()); artifact_names.len()];

        for (name, id) in &artifact_names {
            let dummy_art = dummy.artifact_map.get(name).unwrap();
            artifact_results[*id] = Ok(dummy_art.clone())
        }

        Ok(SwarmGenome {
            species_map: species_results.into_iter().collect::<Result<Vec<_>, _>>()?,
            artifact_map: artifact_results
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
            strategy: ApplicationStrategy::from(dummy.strategy),
            start_dist: convert_distribution(&species_names, &artifact_names, &dummy.start_dist)?,
        })
    }
}
