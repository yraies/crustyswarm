use std::collections::HashMap;

use cgmath::Vector3;

use r_oide::atoms::{BoolCell, BoundedFactor, BoundedIdxVec};

use super::genome::*;
use super::{
    super::genome::{
        energy::{Energy, MovementEnergy, OffspringEnergy, ReplicationEnergy, ZeroEnergy},
        replacement::{ContextRule, Replacement},
        ArtifactIndex, ArtifactType, Species, SpeciesIndex, SurroundingIndex, SwarmGenome,
    },
    FlattenableIntoSurroundingVec,
};

impl From<&OIDESwarmGenome> for SwarmGenome {
    fn from(oide_genome: &OIDESwarmGenome) -> Self {
        let species_map = oide_genome
            .species_map
            .iter()
            .map(|oide_species| -> Species {
                let energy = Energy {
                    on_movement: MovementEnergy::Distance(
                        oide_species.energy.on_movement.get_value(),
                    ),
                    on_zero: ZeroEnergy::Replace(
                        oide_species.energy.on_zero.0.get_value() as u16,
                        Replacement::Simple(
                            FlattenableIntoSurroundingVec::flatten_into_surrounding_vec(
                                &oide_species.energy.on_zero.1,
                                &oide_genome.species_count,
                            ),
                        ),
                    ),
                    on_replication: ReplicationEnergy::Constant(
                        oide_species.energy.on_replication.get_value(),
                    ),
                    for_offspring: OffspringEnergy::Inherit(
                        oide_species.energy.for_offspring.get_value(),
                    ),
                };

                let rules = oide_species
                    .rules
                    .rules
                    .iter()
                    .map(|oide_rule| {
                        let foo = ContextRule {
                            context: FlattenableIntoSurroundingVec::flatten_into_surrounding_vec(
                                &oide_rule.context,
                                &oide_genome.species_count,
                            ),
                            range: oide_rule.range.get_value(),
                            weight: oide_rule.weight.get_value(),
                            persist: oide_rule.persist.clone().into(),
                            replacement: Replacement::Simple(
                                FlattenableIntoSurroundingVec::flatten_into_surrounding_vec(
                                    &oide_rule.replacement,
                                    &oide_genome.species_count,
                                ),
                            ),
                        };
                        foo
                    })
                    .collect();

                let mut influences: HashMap<SurroundingIndex, f32> = oide_species
                    .influenced_by
                    .0
                    .into_f32_vec()
                    .iter()
                    .enumerate()
                    .map(|(idx, influence)| {
                        (SurroundingIndex::Agent(SpeciesIndex(idx)), *influence)
                    })
                    .collect();

                influences.extend(
                    oide_species
                        .influenced_by
                        .1
                        .into_f32_vec()
                        .iter()
                        .enumerate()
                        .map(|(idx, influence)| {
                            (SurroundingIndex::Artifact(ArtifactIndex(idx)), *influence)
                        }),
                );

                Species {
                    index: SpeciesIndex(*oide_species.index),
                    separation: oide_species.separation.get_value(),
                    alignment: oide_species.alignment.get_value(),
                    cohesion: oide_species.cohesion.get_value(),
                    randomness: oide_species.randomness.get_value(),
                    center: oide_species.center.get_value(),
                    floor: oide_species.floor.get_value(),
                    bias: Vector3::new(
                        oide_species.bias.0.get_value(),
                        oide_species.bias.1.get_value(),
                        oide_species.bias.2.get_value(),
                    ),
                    gradient: oide_species.gradient.get_value(),
                    normal: oide_species.normal.get_value(),
                    slope: oide_species.slope.get_value(),
                    normal_speed: oide_species.normal_speed.get_value(),
                    max_speed: oide_species.max_speed.get_value(),
                    max_acceleration: oide_species.max_acceleration.get_value(),
                    pacekeeping: oide_species.pacekeeping.get_value(),
                    view_distance: oide_species.view_distance.get_value(),
                    view_angle: oide_species.view_angle.get_value(),
                    sep_distance: oide_species.sep_distance.get_value(),
                    axis_constraint: Vector3::new(
                        oide_species.axis_constraint.0.get_value(),
                        oide_species.axis_constraint.1.get_value(),
                        oide_species.axis_constraint.2.get_value(),
                    ),
                    influenced_by: influences,
                    noclip: *oide_species.noclip,
                    energy,
                    hand_down_seed: oide_species.hand_down_seed.clone().into(),
                    rules,
                    color_index: *oide_species.color_index,
                }
            })
            .collect();

        SwarmGenome {
            species_map,
            artifact_map: oide_genome
                .artifact_map
                .vec
                .iter()
                .map(|entry| ArtifactType {
                    color_index: entry.value,
                })
                .collect(),
            start_dist: (*oide_genome.start_dist).clone(),
            strategy: (*oide_genome.strategy).clone(),
            terrain_influences: (
                oide_genome.terrain_influences.0.into_f32_vec(),
                oide_genome.terrain_influences.1.into_f32_vec(),
            ),
            terrain_size: *oide_genome.terrain_size,
            terrain_spacing: *oide_genome.terrain_spacing,
        }
    }
}

impl From<&SwarmGenome> for OIDESwarmGenome {
    fn from(genome: &SwarmGenome) -> Self {
        let species_count = genome.species_map.iter().count();
        let artifact_count = genome.artifact_map.iter().count();
        //dbg!(&species_count);
        //dbg!(&artifact_count);
        let rule_count = genome
            .species_map
            .iter()
            .map(|spec| spec.rules.iter().count())
            .max()
            .unwrap_or(0);
        let context_count = genome
            .species_map
            .iter()
            .flat_map(|spec| spec.rules.clone())
            .map(|r| r.context.len())
            .max()
            .unwrap_or(0);
        let replacement_count = genome
            .species_map
            .iter()
            .flat_map(|spec| spec.rules.clone())
            .map(|r| r.replacement.count_replacements())
            .max()
            .unwrap_or(0);

        let to_usize = |loc_idx: SurroundingIndex| match loc_idx {
            SurroundingIndex::Agent(SpeciesIndex(idx)) => idx,
            SurroundingIndex::Artifact(ArtifactIndex(idx)) => species_count + idx,
        };

        let oide_species_map = genome
            .species_map
            .iter()
            .map(|species| {
                //dbg!(&species);
                let species_influences = (0..species_count)
                    .into_iter()
                    .map(|idx| {
                        (
                            true,
                            *species
                                .influenced_by
                                .get(&SpeciesIndex(idx).into())
                                .unwrap_or(&0.0),
                        )
                    })
                    .collect();
                //dbg!(&species_influences);
                let artifact_influences = (0..artifact_count)
                    .into_iter()
                    .map(|idx| {
                        (
                            true,
                            *species
                                .influenced_by
                                .get(&ArtifactIndex(idx).into())
                                .unwrap_or(&0.0),
                        )
                    })
                    .collect();
                //dbg!(&artifact_influences);

                fn conv_rep(rep: &Replacement) -> Vec<SurroundingIndex> {
                    match rep {
                        Replacement::None => vec![],
                        Replacement::Simple(a) => {
                            let foo = a.iter().map(|b| b.to_owned()).collect();
                            foo
                        }
                        Replacement::Multi(a) => {
                            let foo = a.iter().flat_map(|b| conv_rep(b)).collect();
                            foo
                        }
                        Replacement::Spread(_, _, _) => vec![],
                    }
                }

                let energy = OIDEEnergy {
                    on_movement: BoundedFactor::new_from_f32(
                        species.energy.on_movement.get_param(),
                    ),
                    on_zero: (
                        BoundedFactor::new_from_f32(species.energy.on_zero.get_param() as f32),
                        match species.energy.on_zero.clone() {
                            ZeroEnergy::Die => BoundedIdxVec {
                                vec: vec![],
                                upper_bound: 0,
                            },
                            ZeroEnergy::Replace(_, reps) => {
                                let bar = conv_rep(&reps);
                                let v = bar.iter().map(|i| to_usize(i.clone()));
                                BoundedIdxVec {
                                    vec: v
                                        .clone()
                                        .map(|i| BoolCell {
                                            active: 1.0.into(),
                                            value: i,
                                        })
                                        .collect(),
                                    upper_bound: species_count + artifact_count - 1,
                                }
                            }
                            ZeroEnergy::Live => BoundedIdxVec {
                                vec: vec![],
                                upper_bound: 0,
                            },
                        },
                    ),
                    on_replication: BoundedFactor::new_from_f32(
                        species.energy.on_replication.get_param(),
                    ),
                    for_offspring: BoundedFactor::new_from_f32(
                        species.energy.for_offspring.get_param(),
                    ),
                };

                let rules = OIDERuleSet {
                    rules: species
                        .rules
                        .iter()
                        .map(|rule| {
                            let mut context = rule
                                .context
                                .iter()
                                .map(|&a| (true, to_usize(a)))
                                .collect::<BoundedIdxVec>();
                            let mut replacement = conv_rep(&rule.replacement)
                                .iter()
                                .map(|&a| (true, to_usize(a)))
                                .collect::<BoundedIdxVec>();

                            context.fill_to(context_count);
                            replacement.fill_to(replacement_count);

                            OIDEContextRule {
                                context,
                                range: BoundedFactor::new_from_f32(rule.range),
                                weight: BoundedFactor::new_from_f32(rule.weight),
                                persist: rule.persist.into(),
                                replacement,
                            }
                        })
                        .collect(),
                    upper_weight_bound: species
                        .rules
                        .iter()
                        .map(|a| a.weight)
                        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less))
                        .unwrap_or(0.0)
                        .into(),
                    upper_range_bound: species
                        .rules
                        .iter()
                        .map(|a| a.range)
                        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less))
                        .unwrap_or(0.0)
                        .into(),
                };

                OIDESpecies {
                    index: to_usize(species.index.into()).into(),
                    separation: BoundedFactor::new_from_f32(species.separation),
                    alignment: BoundedFactor::new_from_f32(species.alignment),
                    cohesion: BoundedFactor::new_from_f32(species.cohesion),
                    randomness: BoundedFactor::new_from_f32(species.randomness),
                    center: BoundedFactor::new_from_f32(species.center),
                    floor: BoundedFactor::new_from_f32(species.floor),
                    bias: (
                        BoundedFactor::new_from_f32(species.bias.x),
                        BoundedFactor::new_from_f32(species.bias.y),
                        BoundedFactor::new_from_f32(species.bias.z),
                    ),
                    gradient: BoundedFactor::new_from_f32(species.gradient),
                    normal: BoundedFactor::new_from_f32(species.normal),
                    slope: BoundedFactor::new_from_f32(species.slope),
                    normal_speed: BoundedFactor::new_from_f32(species.normal_speed),
                    max_speed: BoundedFactor::new_from_f32(species.max_speed),
                    max_acceleration: BoundedFactor::new_from_f32(species.max_acceleration),
                    pacekeeping: BoundedFactor::new_from_f32(species.pacekeeping),
                    view_distance: BoundedFactor::new_from_f32(species.view_distance),
                    view_angle: BoundedFactor::new_from_f32(species.view_angle),
                    sep_distance: BoundedFactor::new_from_f32(species.sep_distance),
                    axis_constraint: (
                        BoundedFactor::new_from_f32(species.axis_constraint.x),
                        BoundedFactor::new_from_f32(species.axis_constraint.y),
                        BoundedFactor::new_from_f32(species.axis_constraint.z),
                    ),
                    influenced_by: (species_influences, artifact_influences),
                    noclip: species.noclip.into(),
                    energy,
                    hand_down_seed: species.hand_down_seed.into(),
                    rules,
                    color_index: species.color_index.into(),
                }
            })
            .collect();

        let terrain_species_influences = genome
            .terrain_influences
            .0
            .iter()
            .map(|v| (true, *v))
            .collect();
        let terrain_artifact_influences = genome
            .terrain_influences
            .1
            .iter()
            .map(|v| (true, *v))
            .collect();

        return OIDESwarmGenome {
            species_count: species_count.into(),
            artifact_count: artifact_count.into(),
            rule_count: rule_count.into(),
            species_map: oide_species_map,
            artifact_map: genome
                .artifact_map
                .iter()
                .map(|foo| (true, foo.color_index))
                .collect(),
            start_dist: genome.start_dist.clone().into(),
            strategy: genome.strategy.clone().into(),
            terrain_influences: (terrain_species_influences, terrain_artifact_influences),
            terrain_size: genome.terrain_size.into(),
            terrain_spacing: genome.terrain_spacing.into(),
        };
    }
}
