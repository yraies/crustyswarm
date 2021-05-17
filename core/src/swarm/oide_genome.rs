#![allow(dead_code)]

use std::collections::HashMap;

use cgmath::Vector3;
use rand::{distributions::Uniform, Rng};
use serde::{Deserialize, Serialize};

use super::{
    genome::{
        energy::{Energy, MovementEnergy, OffspringEnergy, ReplicationEnergy, ZeroEnergy},
        replacement::{ApplicationStrategy, ContextRule, Replacement},
        ArtifactIndex, ArtifactType, Distribution, Species, SpeciesIndex, SurroundingIndex,
        SwarmGenome,
    },
    oide::Differentiable,
};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
struct BoundedFactor {
    //    lower_bound: f32,
    //    upper_bound: f32,
    //    value: f32,
    base: f32,
    range: f32,
    val: f32,
}

impl BoundedFactor {
    fn new(lower: f32, upper: f32, value: f32) -> BoundedFactor {
        BoundedFactor {
            base: lower,
            range: upper - lower,
            val: value - lower,
        }
    }

    fn get_value(&self) -> f32 {
        return self.base + self.val;
    }

    fn random(&self, rng: &mut impl Rng) -> Self {
        let mut copy = self.clone();
        copy.val = rng.sample(Uniform::new_inclusive(0.0, self.range));
        copy
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct BoolCell<T> {
    pub active: bool,
    pub value: T,
}

impl BoolCell<usize> {
    fn new() -> Self {
        BoolCell {
            active: false,
            value: 0usize,
        }
    }

    fn add(&self, other: &Self, index_count: usize) -> Self {
        Self {
            active: self.active ^ other.active,
            value: (index_count + other.value + self.value) % index_count,
        }
    }

    fn diff(&self, other: &Self, index_count: usize) -> Self {
        Self {
            active: self.active ^ other.active,
            value: (index_count + other.value - self.value) % index_count,
        }
    }

    fn scale(&self, factor: f32) -> Self {
        Self {
            active: if factor == 0.0 { false } else { self.active },
            value: (self.value as f32 * factor).ceil() as usize,
        }
    }
    fn opposite(&self, index_count: usize) -> Self {
        Self {
            active: !self.active,
            value: index_count - self.value - 1,
        }
    }

    fn random(&self, rng: &mut impl Rng, index_count: usize) -> Self {
        BoolCell {
            active: rng.gen(),
            value: rng.gen_range(0, index_count),
        }
    }
}

impl BoolCell<f32> {
    fn new() -> Self {
        BoolCell {
            active: false,
            value: 0f32,
        }
    }

    fn add(&self, other: &Self, lower_bound: f32, upper_bound: f32) -> Self {
        Self {
            active: self.active ^ other.active,
            value: BoundedFactor::new(lower_bound, upper_bound, self.value)
                .add(&BoundedFactor::new(lower_bound, upper_bound, other.value))
                .get_value(),
        }
    }

    fn diff(&self, other: &Self, lower_bound: f32, upper_bound: f32) -> Self {
        Self {
            active: self.active ^ other.active,
            value: BoundedFactor::new(lower_bound, upper_bound, self.value)
                .difference(&BoundedFactor::new(lower_bound, upper_bound, other.value))
                .get_value(),
        }
    }

    fn scale(&self, factor: f32, lower_bound: f32, upper_bound: f32) -> Self {
        Self {
            active: if factor == 0.0 { self.active } else { false },
            value: BoundedFactor::new(lower_bound, upper_bound, self.value)
                .scale(factor)
                .get_value(),
        }
    }
    fn opposite(&self, lower_bound: f32, upper_bound: f32) -> Self {
        Self {
            active: !self.active,
            value: BoundedFactor::new(lower_bound, upper_bound, self.value)
                .opposite()
                .get_value(),
        }
    }

    fn random(&self, rng: &mut impl Rng, lower_bound: f32, upper_bound: f32) -> Self {
        BoolCell {
            active: rng.gen(),
            value: BoundedFactor::new(lower_bound, upper_bound, 0.0)
                .random(rng)
                .get_value(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct BoundedIdxVec {
    pub vec: Vec<BoolCell<usize>>,
    pub index_count: usize,
}

impl BoundedIdxVec {
    fn new(index_count: usize, size: usize) -> BoundedIdxVec {
        BoundedIdxVec {
            vec: vec![BoolCell::<usize>::new(); size],
            index_count,
        }
    }

    fn flatten_into_surrounding_vec(&self, species_count: usize) -> Vec<SurroundingIndex> {
        let foobar = self
            .vec
            .iter()
            .flat_map(|bar| {
                if bar.active {
                    if bar.value >= species_count {
                        let art_id = bar.value - species_count;
                        Some(SurroundingIndex::Artifact(ArtifactIndex(art_id)))
                    } else {
                        Some(SurroundingIndex::Agent(SpeciesIndex(bar.value)))
                    }
                } else {
                    None
                }
            })
            .collect();
        foobar
    }

    fn get_activation_vec(&self) -> Vec<bool> {
        self.vec.iter().map(|bar| bar.active).collect()
    }

    fn random(&self, rng: &mut impl Rng) -> Self {
        let mut copy = self.clone();
        copy.vec = copy
            .vec
            .iter()
            .map(|cell| cell.random(rng, self.index_count))
            .collect();
        copy
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct BoundedFactorVec {
    pub vec: Vec<BoolCell<f32>>,
    pub upper_bound: f32,
    pub lower_bound: f32,
}

impl BoundedFactorVec {
    fn new(lower_bound: f32, upper_bound: f32, size: usize) -> BoundedFactorVec {
        BoundedFactorVec {
            vec: vec![BoolCell::<f32>::new(); size],
            upper_bound,
            lower_bound,
        }
    }

    fn into_f32_vec(&self) -> Vec<f32> {
        self.vec
            .iter()
            .map(|bar| if bar.active { bar.value } else { 0.0 })
            .collect()
    }

    fn random(&self, rng: &mut impl Rng) -> Self {
        let mut copy = self.clone();
        copy.vec = copy
            .vec
            .iter()
            .map(|cell| cell.random(rng, self.lower_bound, self.upper_bound))
            .collect();
        copy
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct OIDESwarmGenome {
    species_count: usize,
    artifact_count: usize,
    rule_count: usize,
    species_map: Vec<OIDESpecies>,
    artifact_map: BoundedIdxVec,
    start_dist: Distribution,
    strategy: ApplicationStrategy,
    terrain_influences: (BoundedFactorVec, BoundedFactorVec),
    terrain_size: usize,
    terrain_spacing: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct OIDESpecies {
    index: usize,
    separation: BoundedFactor,
    alignment: BoundedFactor,
    cohesion: BoundedFactor,
    randomness: BoundedFactor,
    center: BoundedFactor,
    floor: BoundedFactor,
    bias: Vector3<BoundedFactor>,
    gradient: BoundedFactor,
    normal: BoundedFactor,
    slope: BoundedFactor,
    normal_speed: BoundedFactor,
    max_speed: BoundedFactor,
    max_acceleration: BoundedFactor,
    pacekeeping: BoundedFactor,
    view_distance: BoundedFactor,
    view_angle: BoundedFactor,
    sep_distance: BoundedFactor,
    axis_constraint: Vector3<BoundedFactor>,
    influenced_by: (BoundedFactorVec, BoundedFactorVec),
    noclip: bool,
    energy: OIDEEnergy,
    hand_down_seed: bool,
    rules: OIDERuleSet,
    color_index: usize,
}

impl OIDESpecies {
    pub fn new_with_size(
        species_count: usize,
        artifact_count: usize,
        rule_count: usize,
        context_count: usize,
        replacement_count: usize,
    ) -> OIDESpecies {
        OIDESpecies {
            index: 0,
            separation: BoundedFactor::new(0.0, 2.0, 0.0),
            alignment: BoundedFactor::new(0.0, 2.0, 0.0),
            cohesion: BoundedFactor::new(0.0, 2.0, 0.0),
            randomness: BoundedFactor::new(0.0, 2.0, 0.0),
            center: BoundedFactor::new(0.0, 2.0, 0.0),
            floor: BoundedFactor::new(0.0, 2.0, 0.0),
            bias: Vector3::new(
                BoundedFactor::new(-1.0, 1.0, 0.0),
                BoundedFactor::new(-1.0, 1.0, 0.0),
                BoundedFactor::new(-1.0, 1.0, 0.0),
            ),
            gradient: BoundedFactor::new(0.0, 2.0, 0.0),
            normal: BoundedFactor::new(0.0, 2.0, 0.0),
            slope: BoundedFactor::new(0.0, 2.0, 0.0),
            normal_speed: BoundedFactor::new(0.0, 2.0, 0.5),
            max_speed: BoundedFactor::new(0.0, 2.0, 1.0),
            max_acceleration: BoundedFactor::new(0.0, 2.0, 1.0),
            pacekeeping: BoundedFactor::new(0.0, 2.0, 0.0),
            view_distance: BoundedFactor::new(0.0, 200.0, 50.0),
            view_angle: BoundedFactor::new(1.0, 359.9, 270.0),
            sep_distance: BoundedFactor::new(0.0, 50.0, 10.0),
            axis_constraint: Vector3::new(
                BoundedFactor::new(0.0, 2.0, 0.0),
                BoundedFactor::new(0.0, 2.0, 0.0),
                BoundedFactor::new(0.0, 2.0, 0.0),
            ),
            influenced_by: (
                BoundedFactorVec::new(-2.0, 2.0, species_count),
                BoundedFactorVec::new(-2.0, 2.0, artifact_count),
            ),
            noclip: false,
            energy: OIDEEnergy::new_with_size(species_count + artifact_count, replacement_count),
            hand_down_seed: false,
            rules: OIDERuleSet::new_with_size(
                species_count + artifact_count,
                rule_count,
                context_count,
                replacement_count,
            ),
            color_index: 0,
        }
    }

    fn random(&self, rng: &mut impl Rng) -> Self {
        OIDESpecies {
            index: self.index,
            separation: self.separation.random(rng),
            alignment: self.alignment.random(rng),
            cohesion: self.cohesion.random(rng),
            randomness: self.randomness.random(rng),
            center: self.center.random(rng),
            floor: self.floor.random(rng),
            bias: Vector3::new(
                self.bias.x.random(rng),
                self.bias.y.random(rng),
                self.bias.z.random(rng),
            ),
            gradient: self.gradient.random(rng),
            normal: self.normal.random(rng),
            slope: self.slope.random(rng),
            normal_speed: self.normal_speed.random(rng),
            max_speed: self.max_speed.random(rng),
            max_acceleration: self.max_acceleration.random(rng),
            pacekeeping: self.pacekeeping.random(rng),
            view_distance: self.view_distance.random(rng),
            view_angle: self.view_angle.random(rng),
            sep_distance: self.sep_distance.random(rng),
            axis_constraint: Vector3::new(
                self.axis_constraint.x.random(rng),
                self.axis_constraint.y.random(rng),
                self.axis_constraint.z.random(rng),
            ),
            influenced_by: (
                self.influenced_by.0.random(rng),
                self.influenced_by.1.random(rng),
            ),
            noclip: rng.gen(),
            energy: self.energy.random(rng),
            hand_down_seed: rng.gen(),
            rules: self.rules.random(rng),
            color_index: rng.gen_range(0, 16),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
struct OIDEEnergy {
    on_movement: BoundedFactor,
    on_zero: (BoundedFactor, BoundedIdxVec),
    on_replication: BoundedFactor,
    for_offspring: BoundedFactor,
}

impl OIDEEnergy {
    fn new_with_size(index_count: usize, replacement_count: usize) -> OIDEEnergy {
        OIDEEnergy {
            on_movement: BoundedFactor::new(0.0, 10.0, 0.1),
            on_zero: (
                BoundedFactor::new(0.0, 10.0, 0.1),
                BoundedIdxVec::new(index_count, replacement_count),
            ),
            on_replication: BoundedFactor::new(0.0, 10.0, 0.1),
            for_offspring: BoundedFactor::new(0.0, 10.0, 0.1),
        }
    }

    fn random(&self, rng: &mut impl Rng) -> Self {
        OIDEEnergy {
            on_movement: self.on_movement.random(rng),
            on_zero: (self.on_zero.0.random(rng), self.on_zero.1.random(rng)),
            on_replication: self.on_replication.random(rng),
            for_offspring: self.for_offspring.random(rng),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
struct OIDERuleSet {
    rules: Vec<OIDEContextRule>,
    upper_weight_bound: f32,
    upper_range_bound: f32,
}

impl OIDERuleSet {
    fn new_with_size(
        index_count: usize,
        rule_count: usize,
        context_count: usize,
        replacement_count: usize,
    ) -> OIDERuleSet {
        OIDERuleSet {
            rules: vec![
                OIDEContextRule::new_with_size(
                    index_count,
                    context_count,
                    replacement_count
                );
                rule_count
            ],
            upper_range_bound: 15.0,
            upper_weight_bound: 100.0,
        }
    }

    fn random(&self, rng: &mut impl Rng) -> Self {
        OIDERuleSet {
            rules: self
                .rules
                .iter()
                .map(|rule| rule.random(rng, self.upper_range_bound, self.upper_weight_bound))
                .collect(),
            upper_weight_bound: self.upper_weight_bound,
            upper_range_bound: self.upper_range_bound,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
struct OIDEContextRule {
    context: BoundedIdxVec,
    range: f32,
    weight: f32,
    persist: bool,
    replacement: BoundedIdxVec,
}

impl OIDEContextRule {
    fn new_with_size(
        index_count: usize,
        context_count: usize,
        replacement_count: usize,
    ) -> OIDEContextRule {
        OIDEContextRule {
            context: BoundedIdxVec::new(index_count, context_count),
            range: 0.0,
            weight: 1.0,
            persist: true,
            replacement: BoundedIdxVec::new(index_count, replacement_count),
        }
    }

    fn random(&self, rng: &mut impl Rng, upper_range_bound: f32, upper_weight_bound: f32) -> Self {
        OIDEContextRule {
            context: self.context.random(rng),
            range: rng.sample(Uniform::new_inclusive(0.0, upper_range_bound)),
            weight: rng.sample(Uniform::new_inclusive(0.0, upper_weight_bound)),
            persist: rng.gen(),
            replacement: self.context.random(rng),
        }
    }
}

impl OIDESwarmGenome {
    pub fn new(
        spec_count: usize,
        art_count: usize,
        rule_count: usize,
        context_count: usize,
        replacement_count: usize,
    ) -> OIDESwarmGenome {
        OIDESwarmGenome {
            species_count: spec_count,
            artifact_count: art_count,
            rule_count,
            species_map: vec![
                OIDESpecies::new_with_size(
                    spec_count,
                    art_count,
                    rule_count,
                    context_count,
                    replacement_count,
                );
                spec_count
            ],
            artifact_map: BoundedIdxVec::new(15, art_count), // colors
            start_dist: Distribution::Single(
                Vector3::new(0.0, 0.0, 0.0),
                SurroundingIndex::Agent(SpeciesIndex(0)),
            ),
            strategy: ApplicationStrategy {
                every: 2,
                offset: 1,
            },
            terrain_influences: (
                BoundedFactorVec::new(0.0, 5.0, spec_count),
                BoundedFactorVec::new(0.0, 5.0, art_count),
            ),
            terrain_size: 20,
            terrain_spacing: 5.0,
        }
    }

    pub fn random(&self, rng: &mut impl Rng) -> Self {
        OIDESwarmGenome {
            species_count: self.species_count,
            artifact_count: self.artifact_count,
            rule_count: self.rule_count,
            species_map: self
                .species_map
                .iter()
                .map(|spec| spec.random(rng))
                .collect(),
            artifact_map: self.artifact_map.random(rng),
            start_dist: self.start_dist.clone(),
            strategy: self.strategy.clone(),
            terrain_influences: (
                self.terrain_influences.0.random(rng),
                self.terrain_influences.1.random(rng),
            ),
            terrain_size: self.terrain_size,
            terrain_spacing: self.terrain_spacing,
        }
    }
}

impl From<&OIDESwarmGenome> for SwarmGenome {
    fn from(oide_genome: &OIDESwarmGenome) -> Self {
        let species_map = oide_genome
            .species_map
            .iter()
            .map(|oide_species| {
                let energy = Energy {
                    on_movement: MovementEnergy::Distance(
                        oide_species.energy.on_movement.get_value(),
                    ),
                    on_zero: ZeroEnergy::Replace(
                        oide_species.energy.on_zero.0.get_value() as u16,
                        Replacement::Simple(
                            oide_species
                                .energy
                                .on_zero
                                .1
                                .flatten_into_surrounding_vec(oide_genome.species_count),
                        ),
                    ),
                    on_replication: ReplicationEnergy::PropConst(
                        oide_species.energy.on_replication.get_value(),
                    ),
                    for_offspring: OffspringEnergy::PropConst(
                        0f32,
                        oide_species.energy.for_offspring.get_value(),
                    ),
                };

                let rules = oide_species
                    .rules
                    .rules
                    .iter()
                    .map(|oide_rule| {
                        let foo = ContextRule {
                            context: oide_rule
                                .context
                                .flatten_into_surrounding_vec(oide_genome.species_count),
                            range: oide_rule.range,
                            weight: oide_rule.weight,
                            persist: oide_rule.persist,
                            replacement: Replacement::Simple(
                                oide_rule
                                    .replacement
                                    .flatten_into_surrounding_vec(oide_genome.species_count),
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
                        .0
                        .into_f32_vec()
                        .iter()
                        .enumerate()
                        .map(|(idx, influence)| {
                            (SurroundingIndex::Artifact(ArtifactIndex(idx)), *influence)
                        }),
                );

                Species {
                    index: SpeciesIndex(oide_species.index),
                    separation: oide_species.separation.get_value(),
                    alignment: oide_species.alignment.get_value(),
                    cohesion: oide_species.cohesion.get_value(),
                    randomness: oide_species.randomness.get_value(),
                    center: oide_species.center.get_value(),
                    floor: oide_species.floor.get_value(),
                    bias: oide_species.bias.clone().map(|f| f.get_value()),
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
                    axis_constraint: oide_species.axis_constraint.clone().map(|f| f.get_value()),
                    influenced_by: influences,
                    noclip: oide_species.noclip,
                    energy,
                    hand_down_seed: oide_species.hand_down_seed,
                    rules,
                    color_index: oide_species.color_index,
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
            start_dist: oide_genome.start_dist.clone(),
            strategy: oide_genome.strategy,
            terrain_influences: (
                oide_genome.terrain_influences.0.into_f32_vec(),
                oide_genome.terrain_influences.1.into_f32_vec(),
            ),
            terrain_size: oide_genome.terrain_size,
            terrain_spacing: oide_genome.terrain_spacing,
        }
    }
}

impl Differentiable for OIDESwarmGenome {
    fn add(&self, other: &OIDESwarmGenome) -> OIDESwarmGenome {
        OIDESwarmGenome {
            species_count: self.species_count,
            artifact_count: self.artifact_count,
            rule_count: self.rule_count,
            species_map: self
                .species_map
                .iter()
                .map(|spec1| spec1.add(&other.species_map[spec1.index]))
                .collect(),
            artifact_map: self.artifact_map.add(&other.artifact_map),
            start_dist: self.start_dist.clone(),
            strategy: self.strategy,
            terrain_influences: (
                self.terrain_influences.0.add(&other.terrain_influences.0),
                self.terrain_influences.1.add(&other.terrain_influences.1),
            ),
            terrain_size: self.terrain_size,
            terrain_spacing: self.terrain_spacing,
        }
    }

    fn difference(&self, other: &OIDESwarmGenome) -> OIDESwarmGenome {
        OIDESwarmGenome {
            species_count: self.species_count,
            artifact_count: self.artifact_count,
            rule_count: self.rule_count,
            species_map: self
                .species_map
                .iter()
                .map(|spec1| spec1.difference(&other.species_map[spec1.index]))
                .collect(),
            artifact_map: self.artifact_map.difference(&other.artifact_map),
            start_dist: self.start_dist.clone(),
            strategy: self.strategy,
            terrain_influences: (
                self.terrain_influences
                    .0
                    .difference(&other.terrain_influences.0),
                self.terrain_influences
                    .1
                    .difference(&other.terrain_influences.1),
            ),
            terrain_size: self.terrain_size,
            terrain_spacing: self.terrain_spacing,
        }
    }

    fn scale(&self, factor: f32) -> Self {
        OIDESwarmGenome {
            species_count: self.species_count,
            artifact_count: self.artifact_count,
            rule_count: self.rule_count,
            species_map: self
                .species_map
                .iter()
                .map(|spec1| spec1.scale(factor))
                .collect(),
            artifact_map: self.artifact_map.scale(factor),
            start_dist: self.start_dist.clone(),
            strategy: self.strategy,
            terrain_influences: (
                self.terrain_influences.0.scale(factor),
                self.terrain_influences.1.scale(factor),
            ),
            terrain_size: self.terrain_size,
            terrain_spacing: self.terrain_spacing,
        }
    }

    fn opposite(&self) -> OIDESwarmGenome {
        OIDESwarmGenome {
            species_count: self.species_count,
            artifact_count: self.artifact_count,
            rule_count: self.rule_count,
            species_map: self
                .species_map
                .iter()
                .map(|spec1| spec1.opposite())
                .collect(),
            artifact_map: self.artifact_map.opposite(),
            start_dist: self.start_dist.clone(),
            strategy: self.strategy,
            terrain_influences: (
                self.terrain_influences.0.opposite(),
                self.terrain_influences.1.opposite(),
            ),
            terrain_size: self.terrain_size,
            terrain_spacing: self.terrain_spacing,
        }
    }

    fn cop(&self) -> Self {
        OIDESwarmGenome {
            species_count: self.species_count,
            artifact_count: self.artifact_count,
            rule_count: self.rule_count,
            species_map: self.species_map.iter().map(|spec1| spec1.cop()).collect(),
            artifact_map: self.artifact_map.clone(),
            start_dist: self.start_dist.clone(),
            strategy: self.strategy,
            terrain_influences: (
                self.terrain_influences.0.clone(),
                self.terrain_influences.1.clone(),
            ),
            terrain_size: self.terrain_size,
            terrain_spacing: self.terrain_spacing,
        }
    }
}

impl Differentiable for OIDESpecies {
    fn add(&self, other: &Self) -> Self {
        OIDESpecies {
            index: self.index,
            separation: self.separation.add(&other.separation),
            alignment: self.alignment.add(&other.alignment),
            cohesion: self.cohesion.add(&other.cohesion),
            randomness: self.randomness.add(&other.randomness),
            center: self.center.add(&other.center),
            floor: self.floor.add(&other.floor),
            bias: Vector3::new(
                self.bias.x.add(&other.bias.x),
                self.bias.y.add(&other.bias.y),
                self.bias.z.add(&other.bias.z),
            ),
            gradient: self.gradient.add(&other.gradient),
            normal: self.normal.add(&other.normal),
            slope: self.slope.add(&other.slope),
            normal_speed: self.normal_speed.add(&other.normal_speed),
            max_speed: self.max_speed.add(&other.max_speed),
            max_acceleration: self.max_acceleration.add(&other.max_acceleration),
            pacekeeping: self.pacekeeping.add(&other.pacekeeping),
            view_distance: self.view_distance.add(&other.view_distance),
            view_angle: self.view_angle.add(&other.view_angle),
            sep_distance: self.sep_distance.add(&other.sep_distance),
            axis_constraint: Vector3::new(
                self.axis_constraint.x.add(&other.axis_constraint.x),
                self.axis_constraint.y.add(&other.axis_constraint.y),
                self.axis_constraint.z.add(&other.axis_constraint.z),
            ),
            influenced_by: (
                self.influenced_by.0.add(&other.influenced_by.0),
                self.influenced_by.1.add(&other.influenced_by.1),
            ),
            noclip: (self.noclip ^ other.noclip),
            energy: OIDEEnergy {
                on_movement: self.energy.on_movement.add(&other.energy.on_movement),
                on_zero: (
                    self.energy.on_zero.0.add(&other.energy.on_zero.0),
                    self.energy.on_zero.1.add(&other.energy.on_zero.1),
                ),
                on_replication: self.energy.on_replication.add(&other.energy.on_replication),
                for_offspring: self.energy.for_offspring.add(&other.energy.for_offspring),
            },
            hand_down_seed: (self.hand_down_seed ^ other.hand_down_seed),
            rules: self.rules.add(&other.rules),
            color_index: (other.color_index + self.color_index) % 16,
        }
    }

    fn difference(&self, other: &Self) -> Self {
        OIDESpecies {
            index: self.index,
            separation: self.separation.difference(&other.separation),
            alignment: self.alignment.difference(&other.alignment),
            cohesion: self.cohesion.difference(&other.cohesion),
            randomness: self.randomness.difference(&other.randomness),
            center: self.center.difference(&other.center),
            floor: self.floor.difference(&other.floor),
            bias: Vector3::new(
                self.bias.x.difference(&other.bias.x),
                self.bias.y.difference(&other.bias.y),
                self.bias.z.difference(&other.bias.z),
            ),
            gradient: self.gradient.difference(&other.gradient),
            normal: self.normal.difference(&other.normal),
            slope: self.slope.difference(&other.slope),
            normal_speed: self.normal_speed.difference(&other.normal_speed),
            max_speed: self.max_speed.difference(&other.max_speed),
            max_acceleration: self.max_acceleration.difference(&other.max_acceleration),
            pacekeeping: self.pacekeeping.difference(&other.pacekeeping),
            view_distance: self.view_distance.difference(&other.view_distance),
            view_angle: self.view_angle.difference(&other.view_angle),
            sep_distance: self.sep_distance.difference(&other.sep_distance),
            axis_constraint: Vector3::new(
                self.axis_constraint.x.difference(&other.axis_constraint.x),
                self.axis_constraint.y.difference(&other.axis_constraint.y),
                self.axis_constraint.z.difference(&other.axis_constraint.z),
            ),
            influenced_by: (
                self.influenced_by.0.difference(&other.influenced_by.0),
                self.influenced_by.1.difference(&other.influenced_by.1),
            ),
            noclip: (self.noclip ^ other.noclip),
            energy: OIDEEnergy {
                on_movement: self
                    .energy
                    .on_movement
                    .difference(&other.energy.on_movement),
                on_zero: (
                    self.energy.on_zero.0.difference(&other.energy.on_zero.0),
                    self.energy.on_zero.1.difference(&other.energy.on_zero.1),
                ),
                on_replication: self
                    .energy
                    .on_replication
                    .difference(&other.energy.on_replication),
                for_offspring: self
                    .energy
                    .for_offspring
                    .difference(&other.energy.for_offspring),
            },
            hand_down_seed: (self.hand_down_seed ^ other.hand_down_seed),
            rules: self.rules.difference(&other.rules),
            color_index: (15 + other.color_index - self.color_index) % 16,
        }
    }

    fn scale(&self, factor: f32) -> Self {
        OIDESpecies {
            index: self.index,
            separation: self.separation.scale(factor),
            alignment: self.alignment.scale(factor),
            cohesion: self.cohesion.scale(factor),
            randomness: self.randomness.scale(factor),
            center: self.center.scale(factor),
            floor: self.floor.scale(factor),
            bias: Vector3::new(
                self.bias.x.scale(factor),
                self.bias.y.scale(factor),
                self.bias.z.scale(factor),
            ),
            gradient: self.gradient.scale(factor),
            normal: self.normal.scale(factor),
            slope: self.slope.scale(factor),
            normal_speed: self.normal_speed.scale(factor),
            max_speed: self.max_speed.scale(factor),
            max_acceleration: self.max_acceleration.scale(factor),
            pacekeeping: self.pacekeeping.scale(factor),
            view_distance: self.view_distance.scale(factor),
            view_angle: self.view_angle.scale(factor),
            sep_distance: self.sep_distance.scale(factor),
            axis_constraint: Vector3::new(
                self.axis_constraint.x.scale(factor),
                self.axis_constraint.y.scale(factor),
                self.axis_constraint.z.scale(factor),
            ),
            influenced_by: (
                self.influenced_by.0.scale(factor),
                self.influenced_by.1.scale(factor),
            ),
            noclip: self.noclip,
            energy: OIDEEnergy {
                on_movement: self.energy.on_movement.scale(factor),
                on_zero: (
                    self.energy.on_zero.0.scale(factor),
                    self.energy.on_zero.1.scale(factor),
                ),
                on_replication: self.energy.on_replication.scale(factor),
                for_offspring: self.energy.for_offspring.scale(factor),
            },
            hand_down_seed: self.hand_down_seed,
            rules: self.rules.scale(factor),
            color_index: (self.color_index as f32 * factor).ceil() as usize,
        }
    }

    fn opposite(&self) -> Self {
        OIDESpecies {
            index: self.index,
            separation: self.separation.opposite(),
            alignment: self.alignment.opposite(),
            cohesion: self.cohesion.opposite(),
            randomness: self.randomness.opposite(),
            center: self.center.opposite(),
            floor: self.floor.opposite(),
            bias: Vector3::new(
                self.bias.x.opposite(),
                self.bias.y.opposite(),
                self.bias.z.opposite(),
            ),
            gradient: self.gradient.opposite(),
            normal: self.normal.opposite(),
            slope: self.slope.opposite(),
            normal_speed: self.normal_speed.opposite(),
            max_speed: self.max_speed.opposite(),
            max_acceleration: self.max_acceleration.opposite(),
            pacekeeping: self.pacekeeping.opposite(),
            view_distance: self.view_distance.opposite(),
            view_angle: self.view_angle.opposite(),
            sep_distance: self.sep_distance.opposite(),
            axis_constraint: Vector3::new(
                self.axis_constraint.x.opposite(),
                self.axis_constraint.y.opposite(),
                self.axis_constraint.z.opposite(),
            ),
            influenced_by: (
                self.influenced_by.0.opposite(),
                self.influenced_by.1.opposite(),
            ),
            noclip: !self.noclip,
            energy: OIDEEnergy {
                on_movement: self.energy.on_movement.opposite(),
                on_zero: (
                    self.energy.on_zero.0.opposite(),
                    self.energy.on_zero.1.opposite(),
                ),
                on_replication: self.energy.on_replication.opposite(),
                for_offspring: self.energy.for_offspring.opposite(),
            },
            hand_down_seed: !self.hand_down_seed,
            rules: self.rules.opposite(),
            color_index: 15 - self.color_index,
        }
    }

    fn cop(&self) -> Self {
        self.clone()
    }
}

impl Differentiable for OIDERuleSet {
    fn add(&self, other: &Self) -> Self {
        OIDERuleSet {
            rules: self
                .rules
                .iter()
                .zip(other.rules.iter())
                .map(|(s, o)| OIDEContextRule {
                    context: s.context.add(&o.context),
                    range: BoundedFactor::new(0.0, self.upper_range_bound, s.range)
                        .add(&BoundedFactor::new(0.0, self.upper_range_bound, o.range))
                        .get_value(),
                    weight: BoundedFactor::new(0.0, self.upper_weight_bound, s.weight)
                        .add(&BoundedFactor::new(0.0, self.upper_weight_bound, o.weight))
                        .get_value(),
                    persist: s.persist ^ o.persist,
                    replacement: s.replacement.add(&o.replacement),
                })
                .collect(),
            upper_weight_bound: self.upper_weight_bound,
            upper_range_bound: self.upper_range_bound,
        }
    }

    fn difference(&self, other: &Self) -> Self {
        OIDERuleSet {
            rules: self
                .rules
                .iter()
                .zip(other.rules.iter())
                .map(|(s, o)| OIDEContextRule {
                    context: s.context.difference(&o.context),
                    range: BoundedFactor::new(0.0, self.upper_range_bound, s.range)
                        .difference(&BoundedFactor::new(0.0, self.upper_range_bound, o.range))
                        .get_value(),
                    weight: BoundedFactor::new(0.0, self.upper_weight_bound, s.weight)
                        .difference(&BoundedFactor::new(0.0, self.upper_weight_bound, o.weight))
                        .get_value(),
                    persist: s.persist ^ o.persist,
                    replacement: s.replacement.difference(&o.replacement),
                })
                .collect(),
            upper_weight_bound: self.upper_weight_bound,
            upper_range_bound: self.upper_range_bound,
        }
    }

    fn scale(&self, factor: f32) -> Self {
        OIDERuleSet {
            rules: self
                .rules
                .iter()
                .map(|rule| OIDEContextRule {
                    context: rule.context.scale(factor),
                    range: BoundedFactor::new(0.0, self.upper_range_bound, rule.range)
                        .scale(factor)
                        .get_value(),
                    weight: BoundedFactor::new(0.0, self.upper_weight_bound, rule.weight)
                        .scale(factor)
                        .get_value(),
                    persist: rule.persist,
                    replacement: rule.replacement.scale(factor),
                })
                .collect(),
            upper_weight_bound: self.upper_weight_bound,
            upper_range_bound: self.upper_range_bound,
        }
    }

    fn opposite(&self) -> Self {
        OIDERuleSet {
            rules: self
                .rules
                .iter()
                .map(|rule| OIDEContextRule {
                    context: rule.context.opposite(),
                    range: BoundedFactor::new(0.0, self.upper_range_bound, rule.range)
                        .opposite()
                        .get_value(),
                    weight: BoundedFactor::new(0.0, self.upper_weight_bound, rule.weight)
                        .opposite()
                        .get_value(),
                    persist: !rule.persist,
                    replacement: rule.replacement.opposite(),
                })
                .collect(),
            upper_weight_bound: self.upper_weight_bound,
            upper_range_bound: self.upper_range_bound,
        }
    }

    fn cop(&self) -> Self {
        self.clone()
    }
}

impl Differentiable for BoundedIdxVec {
    fn add(&self, other: &Self) -> Self {
        BoundedIdxVec {
            vec: self
                .vec
                .iter()
                .zip(&other.vec)
                .map(|(first, second)| first.diff(second, self.index_count))
                .collect(),
            index_count: self.index_count,
        }
    }

    fn difference(&self, other: &Self) -> Self {
        BoundedIdxVec {
            vec: self
                .vec
                .iter()
                .zip(&other.vec)
                .map(|(first, second)| first.add(second, self.index_count))
                .collect(),
            index_count: self.index_count,
        }
    }

    fn scale(&self, factor: f32) -> Self {
        BoundedIdxVec {
            vec: self.vec.iter().map(|cell| cell.scale(factor)).collect(),
            index_count: self.index_count,
        }
    }

    fn opposite(&self) -> Self {
        BoundedIdxVec {
            vec: self
                .vec
                .iter()
                .map(|cell| cell.opposite(self.index_count))
                .collect(),
            index_count: self.index_count,
        }
    }

    fn cop(&self) -> Self {
        self.clone()
    }
}

impl Differentiable for BoundedFactorVec {
    fn add(&self, other: &Self) -> Self {
        BoundedFactorVec {
            vec: self
                .vec
                .iter()
                .zip(&other.vec)
                .map(|(first, second)| first.diff(second, self.lower_bound, self.upper_bound))
                .collect(),
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }

    fn difference(&self, other: &Self) -> Self {
        BoundedFactorVec {
            vec: self
                .vec
                .iter()
                .zip(&other.vec)
                .map(|(first, second)| first.add(second, self.lower_bound, self.upper_bound))
                .collect(),
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }

    fn scale(&self, factor: f32) -> Self {
        BoundedFactorVec {
            vec: self
                .vec
                .iter()
                .map(|cell| cell.scale(factor, self.lower_bound, self.upper_bound))
                .collect(),
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }

    fn opposite(&self) -> Self {
        BoundedFactorVec {
            vec: self
                .vec
                .iter()
                .map(|cell| cell.opposite(self.lower_bound, self.upper_bound))
                .collect(),
            upper_bound: self.upper_bound,
            lower_bound: self.lower_bound,
        }
    }

    fn cop(&self) -> Self {
        self.clone()
    }
}

impl Differentiable for BoundedFactor {
    fn add(&self, other: &Self) -> Self {
        BoundedFactor {
            base: self.base,
            range: self.range,
            val: {
                let sum = self.val + other.val; // 0 <= sum <= 2 self.range
                if sum > self.range {
                    2f32 * self.range - sum
                } else {
                    sum
                }
            },
        }
    }

    fn difference(&self, other: &Self) -> Self {
        BoundedFactor {
            base: self.base,
            range: self.range,
            val: {
                let diff = self.val - other.val; // - self.range <= diff <= self.range
                if diff < 0.0 {
                    -diff
                } else {
                    diff
                }
            },
        }
    }

    fn opposite(&self) -> Self {
        BoundedFactor {
            base: self.base,
            range: self.range,
            val: self.range - self.val,
        }
    }

    fn scale(&self, factor: f32) -> Self {
        BoundedFactor {
            base: self.base,
            range: self.range,
            val: self.val * factor, //TODO: Handle factor > 1.0
        }
    }

    fn cop(&self) -> Self {
        self.clone()
    }
}

#[cfg(test)]
mod testbounded_factors {
    use super::*;
    use rand::{prelude::*, SeedableRng};

    fn test_bounded_factor(count: usize, test: fn(BoundedFactor)) {
        let mut rng = StdRng::seed_from_u64(1_234_567_890);
        let uni = Uniform::new_inclusive(-10.0, 10.0);

        for _ in 0..count {
            let mut vals: Vec<f32> = vec![rng.sample(&uni), rng.sample(&uni), rng.sample(&uni)];
            vals.sort_by(|o1, o2| o1.partial_cmp(o2).unwrap());
            let factor = BoundedFactor::new(vals[0], vals[2], vals[1]);
            test(factor);
        }
    }

    fn test_bounded_factors(count: usize, test: fn(BoundedFactor, BoundedFactor)) {
        let mut rng = StdRng::seed_from_u64(1_234_567_890);
        let uni = Uniform::new_inclusive(-10.0, 10.0);

        let factor = BoundedFactor {
            base: -9.0,
            range: 6.0,
            val: 3.75,
        };
        let factor2 = BoundedFactor {
            base: -9.0,
            range: 6.0,
            val: 2.0,
        };
        test(factor, factor2);

        for _i in 0..count {
            let mut vals: Vec<f32> = vec![
                rng.sample(&uni),
                rng.sample(&uni),
                rng.sample(&uni),
                rng.sample(&uni),
            ];
            vals.sort_by(|o1, o2| o1.partial_cmp(o2).unwrap());
            let factor = BoundedFactor::new(vals[0], vals[3], vals[1]);
            let factor2 = BoundedFactor::new(vals[0], vals[3], vals[2]);
            if rng.gen() {
                dbg!(_i);
                dbg!(&factor);
                dbg!(&factor2);
                test(factor, factor2);
            } else {
                dbg!(_i);
                dbg!(&factor2);
                dbg!(&factor);
                test(factor2, factor);
            }
        }
    }

    #[test]
    fn basic_addition() {
        let factor = BoundedFactor::new(0.0, 4.0, 2.0);
        let factor2 = BoundedFactor::new(0.0, 4.0, 3.0);
        assert_eq!(BoundedFactor::new(0.0, 4.0, 3.0), factor.add(&factor2));

        let factor = BoundedFactor::new(10.0, 20.0, 19.0);
        let factor2 = BoundedFactor::new(10.0, 20.0, 19.0);
        assert_eq!(BoundedFactor::new(10.0, 20.0, 12.0), factor.add(&factor2));
    }

    #[test]
    fn basic_difference() {
        let factor = BoundedFactor::new(-10.0, 10.0, 5.0);
        let factor2 = BoundedFactor::new(-10.0, 10.0, 5.0);
        assert_eq!(
            BoundedFactor::new(-10.0, 10.0, -10.0),
            factor.difference(&factor2)
        );

        let factor = BoundedFactor::new(-10.0, 10.0, 10.0);
        let factor2 = BoundedFactor::new(-10.0, 10.0, -10.0);
        assert_eq!(
            BoundedFactor::new(-10.0, 10.0, 10.0),
            factor.difference(&factor2)
        );

        let factor = BoundedFactor::new(-10.0, 10.0, 5.0);
        let factor2 = BoundedFactor::new(-10.0, 10.0, -7.0);
        assert_eq!(
            BoundedFactor::new(-10.0, 10.0, 2.0),
            factor.difference(&factor2)
        );
    }

    #[test]
    fn fuzz_opposite_idempotent() {
        test_bounded_factor(1000, |factor| {
            assert!(
                factor.get_value() - factor.opposite().opposite().get_value() <= 0.000001,
                "Difference: {:?}",
                factor.get_value() - factor.opposite().opposite().get_value()
            );
        });
    }

    #[test]
    fn fuzz_add_opposite_eq_upper() {
        test_bounded_factor(1000, |factor| {
            let testfac = factor.add(&factor.opposite());
            assert!(
                testfac.get_value() - testfac.base - testfac.range <= 0.000001,
                "Difference: {:?}",
                testfac.get_value() - testfac.base - testfac.range
            );
        });
    }

    #[test]
    fn fuzz_sum_of_diff_and_diff_opposite_idempotent() {
        test_bounded_factors(1000, |factor1, factor2| {
            let diff = factor1.difference(&factor2);
            let testfac = factor1.add(&diff).add(&diff.opposite()).opposite();
            assert!(
                factor1.val - factor1.range - testfac.val <= 0.00001,
                "\nF1: {:?}\nF2: {:?}\nDiff: {:?}\nDiff.opp: {:?}\nF1.add(Diff): {:?}\nF1.add(Diff).add(Diff.opp): {:?}",
                factor1,
                factor2,
                diff.val,
                diff.opposite().val,
                factor1.add(&diff).val,
                testfac.val
            ); // kaputt, weil a+(a-b)-(a-b) != a+((a-b)-(a-b))
        });
    }

    #[test]
    fn fuzz_diff_correctly_uncomutative() {
        test_bounded_factors(1000, |factor1, factor2| {
            let diff = factor1.difference(&factor2);
            let diff2 = factor2.difference(&factor1);
            let testfac = factor1.add(&diff).add(&diff2);
            assert!(
                factor1.get_value() - testfac.get_value() <= 0.00001,
                "Difference: {:?}",
                factor1.get_value() - testfac.get_value()
            );
        });
    }
}

#[cfg(test)]
mod testidxvec {
    use super::*;
    use rand::{prelude::*, SeedableRng};
    use std::fmt::{Display, Write};

    impl Display for BoundedIdxVec {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_char('[')?;
            f.write_str(
                &self
                    .vec
                    .iter()
                    .map(|cell| {
                        return format!(
                            "[{} {:2}]",
                            if cell.active { "O" } else { "X" },
                            cell.value
                        );
                    })
                    .fold(Vec::<String>::new(), |mut acc, elem| {
                        acc.push(elem);
                        acc
                    })
                    .join(", "),
            )?;
            f.write_char(']')?;
            Ok(())
        }
    }

    #[test]
    fn add() {
        let mut rng = StdRng::seed_from_u64(1237919273);
        let total_size = 20;
        for total_count in 1..20 {
            let v1 = BoundedIdxVec::new(total_count, total_size);
            for _i in 0..2000 {
                let v1 = v1.random(&mut rng);
                let v2 = v1.random(&mut rng);
                let f1 = |v: &BoundedIdxVec| v.vec.iter().any(|cell| cell.value >= total_count);
                let spec_count = rng.gen_range(0, total_count);
                let f2 = |v: &BoundedIdxVec| {
                    v.flatten_into_surrounding_vec(spec_count)
                        .iter()
                        .any(|idx| match idx {
                            SurroundingIndex::Agent(SpeciesIndex(i)) => i >= &spec_count,
                            SurroundingIndex::Artifact(ArtifactIndex(i)) => {
                                i >= &(total_count - &spec_count)
                            }
                        })
                };

                println!("v1  : {} {} {}:{}", &v1, f1(&v1), spec_count, f2(&v1));
                assert!(!f1(&v1));
                assert!(!f2(&v1));

                println!("v2  : {} {} {}:{}", &v2, f1(&v2), spec_count, f2(&v2));
                assert!(!f1(&v2));
                assert!(!f2(&v2));

                let o1 = v1.opposite();
                println!("o1 : {} {} {}:{}", o1, f1(&o1), spec_count, f2(&o1));
                assert!(!f1(&o1));
                assert!(!f2(&o1));

                let o2 = v2.opposite();
                println!("o2 : {} {} {}:{}", o2, f1(&o2), spec_count, f2(&o2));
                assert!(!f1(&o2));
                assert!(!f2(&o2));

                let s1 = v1.scale(0.5);
                println!("s1 : {} {} {}:{}", s1, f1(&s1), spec_count, f2(&s1));
                assert!(!f1(&s1));
                assert!(!f2(&s1));

                let s2 = v2.scale(0.5);
                println!("s2 : {} {} {}:{}", s2, f1(&s2), spec_count, f2(&s2));
                assert!(!f1(&s2));
                assert!(!f2(&s2));

                let v3 = v1.add(&v2);
                println!("add : {} {} {}:{}", v3, f1(&v3), spec_count, f2(&v3));
                assert!(!f1(&v3));
                assert!(!f2(&v3));

                let v4 = v1.difference(&v2);
                println!("diff: {} {} {}:{}", v4, f1(&v4), spec_count, f2(&v4));
                assert!(!f1(&v4));
                assert!(!f2(&v4));

                println!();
            }
        }
    }
}
