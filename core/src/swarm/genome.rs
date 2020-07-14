pub mod dummies;
pub mod energy;
pub mod replacement;

use cgmath::Vector3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use swarm::actor::{Agent, Artifact};

use self::dummies::*;
use self::replacement::*;

use crate::utils::{Uid, UidGen};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
pub enum SurroundingIndex {
    Agent(SpeciesIndex),
    Artifact(ArtifactIndex),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
#[serde(transparent)]
pub struct SpeciesIndex(pub usize);

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
#[serde(transparent)]
pub struct ArtifactIndex(pub usize);

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
    pub terrain_influences: (Vec<f32>, Vec<f32>),
    pub terrain_size: usize,
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
    pub floor: Factor,
    pub bias: Vector3<Factor>,
    pub gradient: Factor,
    pub normal: Factor,
    pub normal_speed: Factor,
    pub max_speed: Factor,
    pub max_acceleration: Factor,
    pub pacekeeping: Factor,
    pub view_distance: Factor,
    pub sep_distance: Factor,
    pub axis_constraint: Vector3<Factor>,
    pub influenced_by: HashMap<SurroundingIndex, InfluenceFactor>,
    pub noclip: bool,
    pub energy: energy::Energy,
    pub hand_down_seed: bool,
    rules: Vec<ContextRule>,
    pub color_index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
enum Distribution {
    Multi(Vec<Distribution>),
    Single(Vector3<f32>, SurroundingIndex),
    Singularity(Vector3<f32>, Vec<(usize, SurroundingIndex)>),
    Grid(usize, f32, SurroundingIndex),
}

impl SwarmGenome {
    pub fn get_rules(&self, species_index: &SpeciesIndex) -> &Vec<ContextRule> {
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
                if !count.eq(&0usize) {
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
                Agent::mk_new(
                    pos,
                    Vector3::new(0.0, 0.0, 0.0),
                    10.0,
                    species_index,
                    pos,
                    uid,
                )
                .unwrap(),
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

        dbg!(&species_names);
        dbg!(&artifact_names);

        let mut species_results: Vec<Result<Species, Self::Error>> =
            vec![Err("No species initialized".to_string()); species_names.len()];

        for (name, id) in &species_names {
            let dummy_spec = dummy.species_map.get(name).unwrap();
            let influences = dummy_spec
                .influenced_by
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

            let zero: Result<energy::ZeroEnergy, String> = match &dummy_spec.energy.on_zero {
                DummyZeroEnergy::Replace(energy, dummy_replacement) => {
                    let replacement_result =
                        convert_replacement(&species_names, &artifact_names, &dummy_replacement);
                    let replacement = replacement_result?;
                    Ok(energy::ZeroEnergy::Replace(*energy, replacement))
                }
                DummyZeroEnergy::Die => Ok(energy::ZeroEnergy::Die),
                DummyZeroEnergy::Live => Ok(energy::ZeroEnergy::Live),
            };

            let energy = energy::Energy {
                on_movement: dummy_spec.energy.on_movement,
                for_offspring: dummy_spec.energy.for_offspring,
                on_replication: dummy_spec.energy.on_replication,
                on_zero: zero?,
            };

            let species = Species {
                alignment: dummy_spec.urges.alignment,
                axis_constraint: Vector3::from(dummy_spec.axis_constraint),
                center: dummy_spec.urges.center,
                cohesion: dummy_spec.urges.cohesion,
                bias: Vector3::from(dummy_spec.urges.bias),
                gradient: dummy_spec.urges.gradient,
                normal: dummy_spec.urges.normal,
                pacekeeping: dummy_spec.urges.pacekeeping,
                normal_speed: dummy_spec.normal_speed,
                hand_down_seed: dummy_spec.hand_down_seed,
                index: SpeciesIndex(*id),
                influenced_by: influences,
                floor: dummy_spec.urges.floor,
                max_speed: dummy_spec.max_speed,
                max_acceleration: dummy_spec.max_acceleration,
                noclip: dummy_spec.noclip,
                randomness: dummy_spec.urges.randomness,
                rules,
                energy,
                sep_distance: dummy_spec.sep_distance,
                separation: dummy_spec.urges.separation,
                view_distance: dummy_spec.view_distance,
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

        let mut terrain_art_results: Vec<Result<f32, Self::Error>> =
            vec![Err("No terrain influence initialized".to_string()); artifact_names.len()];
        let mut terrain_spec_results: Vec<Result<f32, Self::Error>> =
            vec![Err("No terrain influence initialized".to_string()); species_names.len()];

        for (name, id) in &species_names {
            let dummy_inf = dummy.terrain.influenced_by.get(name).unwrap_or(&0.0);
            terrain_spec_results[*id] = Ok(dummy_inf.clone())
        }

        for (name, id) in &artifact_names {
            let dummy_inf = dummy.terrain.influenced_by.get(name).unwrap_or(&0.0);
            terrain_art_results[*id] = Ok(dummy_inf.clone())
        }

        let terrain_spec = terrain_spec_results
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        let terrain_art = terrain_art_results
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        Ok(SwarmGenome {
            species_map: species_results.into_iter().collect::<Result<Vec<_>, _>>()?,
            artifact_map: artifact_results
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?,
            strategy: ApplicationStrategy::from(dummy.strategy),
            start_dist: convert_distribution(&species_names, &artifact_names, &dummy.start_dist)?,
            terrain_size: dummy.terrain.size,
            terrain_influences: (terrain_spec, terrain_art),
        })
    }
}
