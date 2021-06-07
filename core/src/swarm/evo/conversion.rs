use std::collections::HashMap;

use cgmath::Vector3;

use r_oide::atoms::BoundedFactor;

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
                    on_movement: match oide_species.energy.on_movement.0.get_value().trunc()
                        as usize
                    {
                        0 => {
                            MovementEnergy::Constant(oide_species.energy.on_movement.1.get_value())
                        }
                        1 => {
                            MovementEnergy::Distance(oide_species.energy.on_movement.1.get_value())
                        }
                        2 => MovementEnergy::None,
                        _ => unreachable!(
                            "This should not happen! on_movement: {}",
                            oide_species.energy.on_movement.0.get_value()
                        ),
                    },
                    on_zero: match oide_species.energy.on_zero.0.get_value().trunc() as usize {
                        0 => ZeroEnergy::Die,
                        1 => ZeroEnergy::Replace(
                            oide_species.energy.on_zero.1.get_value() as u16,
                            Replacement::Simple(
                                FlattenableIntoSurroundingVec::flatten_into_surrounding_vec(
                                    &oide_species.energy.on_zero.2.get_indices(),
                                    &oide_genome.species_count,
                                ),
                            ),
                        ),
                        2 => ZeroEnergy::Live,
                        _ => unreachable!(
                            "This should not happen! on_zero: {}",
                            oide_species.energy.on_zero.0.get_value()
                        ),
                    },
                    on_replication: match oide_species.energy.on_replication.0.get_value().trunc()
                        as usize
                    {
                        0 => ReplicationEnergy::Constant(
                            oide_species.energy.on_replication.1.get_value(),
                        ),
                        1 => ReplicationEnergy::Count(
                            oide_species.energy.on_replication.1.get_value(),
                        ),
                        2 => ReplicationEnergy::PropRel,
                        3 => ReplicationEnergy::PropConst(
                            oide_species.energy.on_replication.1.get_value(),
                        ),
                        4 => ReplicationEnergy::None,
                        _ => unreachable!(
                            "This should not happen! on_replication: {}",
                            oide_species.energy.on_replication.0.get_value()
                        ),
                    },
                    for_offspring: match oide_species.energy.for_offspring.0.get_value().trunc()
                        as usize
                    {
                        0 => OffspringEnergy::Constant(
                            oide_species.energy.for_offspring.1.get_value(),
                        ),
                        1 => OffspringEnergy::Inherit(
                            oide_species.energy.for_offspring.1.get_value(),
                        ),
                        2 => OffspringEnergy::PropRel(
                            oide_species.energy.for_offspring.1.get_value(),
                        ),
                        3 => OffspringEnergy::PropConst(
                            oide_species.energy.for_offspring.1.get_value(),
                            oide_species.energy.for_offspring.2.get_value(),
                        ),
                        _ => panic!(
                            "This should not happen! for_offspring: {}, {}, {}",
                            oide_species.energy.for_offspring.0.get_value(),
                            oide_species.energy.for_offspring.0.get_value().trunc(),
                            oide_species.energy.for_offspring.0.get_value().trunc() as usize
                        ),
                    },
                };

                let rules = oide_species
                    .rules
                    .rules
                    .iter()
                    .map(|oide_rule| {
                        let foo = ContextRule {
                            context: FlattenableIntoSurroundingVec::flatten_into_surrounding_vec(
                                &oide_rule.context.get_indices(),
                                &oide_genome.species_count,
                            ),
                            range: oide_rule.range.get_value(),
                            weight: oide_rule.weight.get_value(),
                            persist: oide_rule.persist.clone().into(),
                            replacement: Replacement::Simple(
                                FlattenableIntoSurroundingVec::flatten_into_surrounding_vec(
                                    &oide_rule.replacement.get_indices(),
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
                    noclip: oide_species.noclip.is_active(),
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
                    on_movement: {
                        let (v1, v2): (f32, f32) = match &species.energy.on_movement {
                            MovementEnergy::Constant(v) => (0.0, *v),
                            MovementEnergy::Distance(v) => (0.0, *v),
                            MovementEnergy::None => (0.0, 0.0),
                        };
                        (
                            BoundedFactor::new_with_bounds(0.0, 3.0 - (f32::EPSILON * 4.0), v1),
                            BoundedFactor::new_with_bounds(0.0, 10.0, v2),
                        )
                    },
                    on_zero: {
                        match &species.energy.on_zero {
                            ZeroEnergy::Die => (
                                BoundedFactor::new_with_bounds(
                                    0.0,
                                    3.0 - (f32::EPSILON * 4.0),
                                    0.0,
                                ),
                                BoundedFactor::new_with_bounds(0.0, 10.0, 0.0),
                                vec![0.0; artifact_count + species_count].into(),
                            ),
                            ZeroEnergy::Replace(v1, reps) => {
                                let mut distribution: Vec<f32> =
                                    vec![0.0; artifact_count + species_count];
                                conv_rep(&reps)
                                    .iter()
                                    .map(|i| to_usize(i.clone()))
                                    .for_each(|idx| distribution[idx] = distribution[idx] + 1.0);
                                (
                                    BoundedFactor::new_with_bounds(
                                        0.0,
                                        3.0 - (f32::EPSILON * 4.0),
                                        1.0,
                                    ),
                                    BoundedFactor::new_with_bounds(0.0, 10.0, *v1 as f32),
                                    distribution.into(),
                                )
                            }
                            ZeroEnergy::Live => (
                                BoundedFactor::new_with_bounds(
                                    0.0,
                                    3.0 - (f32::EPSILON * 4.0),
                                    2.0,
                                ),
                                BoundedFactor::new_with_bounds(0.0, 10.0, 0.0),
                                vec![0.0; artifact_count + species_count].into(),
                            ),
                        }
                    },
                    on_replication: {
                        let (v1, v2): (f32, f32) = match &species.energy.on_replication {
                            ReplicationEnergy::Constant(v) => (0.0, *v),
                            ReplicationEnergy::Count(v) => (1.0, *v),
                            ReplicationEnergy::PropRel => (2.0, 0.0),
                            ReplicationEnergy::PropConst(v) => (3.0, *v),
                            ReplicationEnergy::None => (4.0, 0.0),
                        };
                        (
                            BoundedFactor::new_with_bounds(0.0, 5.0 - (f32::EPSILON * 4.0), v1),
                            BoundedFactor::new_with_bounds(0.0, 10.0, v2),
                        )
                    },
                    for_offspring: {
                        let (v1, v2, v3): (f32, f32, f32) = match &species.energy.for_offspring {
                            OffspringEnergy::Constant(v1) => (0.0, *v1, 0.0),
                            OffspringEnergy::Inherit(v1) => (1.0, *v1, 0.0),
                            OffspringEnergy::PropRel(v1) => (2.0, *v1, 0.0),
                            OffspringEnergy::PropConst(v1, v2) => (3.0, *v1, *v2),
                        };
                        (
                            BoundedFactor::new_with_bounds(0.0, 4.0 - (f32::EPSILON * 4.0), v1),
                            BoundedFactor::new_with_bounds(0.0, 10.0, v2),
                            BoundedFactor::new_with_bounds(0.0, 10.0, v3),
                        )
                    },
                };

                let mut rules: Vec<_> = species
                    .rules
                    .iter()
                    .map(|rule| {
                        let mut context = vec![0.0; artifact_count + species_count];
                        let mut replacement = vec![0.0; artifact_count + species_count];

                        rule.context.iter().for_each(|surr_idx| {
                            let idx = to_usize(*surr_idx);
                            context[idx] = context[idx] + 1.0;
                        });
                        conv_rep(&rule.replacement).iter().for_each(|surr_idx| {
                            let idx = to_usize(*surr_idx);
                            replacement[idx] = replacement[idx] + 1.0;
                        });

                        OIDEContextRule {
                            context: context.into(),
                            range: BoundedFactor::new_from_f32(rule.range),
                            weight: BoundedFactor::new_from_f32(rule.weight),
                            persist: rule.persist.into(),
                            replacement: replacement.into(),
                        }
                    })
                    .collect();

                rules.resize(
                    rule_count,
                    OIDEContextRule::new_with_size(artifact_count + species_count),
                );

                let ruleset = OIDERuleSet {
                    rules,
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
                    rules: ruleset,
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
