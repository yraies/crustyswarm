#![allow(dead_code)]

use cgmath::Vector3;
use rand::{distributions::Uniform, Rng};
use serde::{Deserialize, Serialize};

use super::super::genome::{
    replacement::ApplicationStrategy, Distribution, SpeciesIndex, SurroundingIndex,
};
use r_oide::{atoms::*, traits::*};

#[derive(Debug, Serialize, Deserialize, PartialEq, ::derive_diff::Differentiable)]
pub struct OIDESwarmGenome {
    pub species_count: Fixed<usize>,
    pub artifact_count: Fixed<usize>,
    pub rule_count: Fixed<usize>,
    pub species_map: Vec<OIDESpecies>,
    pub artifact_map: BoundedIdxVec,
    pub start_dist: Fixed<Distribution>,
    pub strategy: Fixed<ApplicationStrategy>,
    pub terrain_influences: (BoundedFactorVec, BoundedFactorVec),
    pub terrain_size: Fixed<usize>,
    pub terrain_spacing: Fixed<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ::derive_diff::Differentiable)]
pub struct OIDESpecies {
    pub index: Fixed<usize>,
    pub separation: BoundedFactor,
    pub alignment: BoundedFactor,
    pub cohesion: BoundedFactor,
    pub randomness: BoundedFactor,
    pub center: BoundedFactor,
    pub floor: BoundedFactor,
    pub bias: (BoundedFactor, BoundedFactor, BoundedFactor),
    pub gradient: BoundedFactor,
    pub normal: BoundedFactor,
    pub slope: BoundedFactor,
    pub normal_speed: BoundedFactor,
    pub max_speed: BoundedFactor,
    pub max_acceleration: BoundedFactor,
    pub pacekeeping: BoundedFactor,
    pub view_distance: BoundedFactor,
    pub view_angle: BoundedFactor,
    pub sep_distance: BoundedFactor,
    pub axis_constraint: (BoundedFactor, BoundedFactor, BoundedFactor),
    pub influenced_by: (BoundedFactorVec, BoundedFactorVec),
    pub noclip: FloatyBool,
    pub energy: OIDEEnergy,
    pub hand_down_seed: FloatyBool,
    pub rules: OIDERuleSet,
    pub color_index: Fixed<usize>,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, Default, PartialEq, ::derive_diff::Differentiable,
)]
pub struct OIDEEnergy {
    pub on_movement: BoundedFactor,
    pub on_zero: (BoundedFactor, BoundedIdxVec),
    pub on_replication: BoundedFactor,
    pub for_offspring: BoundedFactor,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, Default, PartialEq, ::derive_diff::Differentiable,
)]
pub struct OIDERuleSet {
    pub rules: Vec<OIDEContextRule>,
    pub upper_weight_bound: Fixed<f32>,
    pub upper_range_bound: Fixed<f32>,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, Default, PartialEq, ::derive_diff::Differentiable,
)]
pub struct OIDEContextRule {
    pub context: BoundedIdxVec,
    pub range: Fixed<f32>,
    pub weight: Fixed<f32>,
    pub persist: FloatyBool,
    pub replacement: BoundedIdxVec,
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
            species_count: spec_count.into(),
            artifact_count: art_count.into(),
            rule_count: rule_count.into(),
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
            )
            .into(),
            strategy: ApplicationStrategy {
                every: 2,
                offset: 1,
            }
            .into(),
            terrain_influences: (
                BoundedFactorVec::new(0.0, 5.0, spec_count),
                BoundedFactorVec::new(0.0, 5.0, art_count),
            ),
            terrain_size: 20.into(),
            terrain_spacing: 5.0.into(),
        }
    }

    pub fn random(&self, rng: &mut impl Rng) -> Self {
        OIDESwarmGenome {
            species_count: self.species_count.clone(),
            artifact_count: self.artifact_count.clone(),
            rule_count: self.rule_count.clone(),
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
            terrain_size: self.terrain_size.clone(),
            terrain_spacing: self.terrain_spacing.clone(),
        }
    }

    pub fn get_first_context_count(&self) -> usize {
        self.species_map[0].rules.rules[0].context.vec.len()
    }

    pub fn get_first_replacement_count(&self) -> usize {
        self.species_map[0].rules.rules[0].replacement.vec.len()
    }
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
            index: 0.into(),
            separation: BoundedFactor::new(0.0, 2.0, 0.0),
            alignment: BoundedFactor::new(0.0, 2.0, 0.0),
            cohesion: BoundedFactor::new(0.0, 2.0, 0.0),
            randomness: BoundedFactor::new(0.0, 2.0, 0.0),
            center: BoundedFactor::new(0.0, 2.0, 0.0),
            floor: BoundedFactor::new(0.0, 2.0, 0.0),
            bias: (
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
            axis_constraint: (
                BoundedFactor::new(0.0, 2.0, 0.0),
                BoundedFactor::new(0.0, 2.0, 0.0),
                BoundedFactor::new(0.0, 2.0, 0.0),
            ),
            influenced_by: (
                BoundedFactorVec::new(-2.0, 2.0, species_count),
                BoundedFactorVec::new(-2.0, 2.0, artifact_count),
            ),
            noclip: false.into(),
            energy: OIDEEnergy::new_with_size(species_count + artifact_count, replacement_count),
            hand_down_seed: false.into(),
            rules: OIDERuleSet::new_with_size(
                species_count + artifact_count,
                rule_count,
                context_count,
                replacement_count,
            ),
            color_index: 0.into(),
        }
    }

    fn random(&self, rng: &mut impl Rng) -> Self {
        OIDESpecies {
            index: self.index.clone(),
            separation: self.separation.random(rng),
            alignment: self.alignment.random(rng),
            cohesion: self.cohesion.random(rng),
            randomness: self.randomness.random(rng),
            center: self.center.random(rng),
            floor: self.floor.random(rng),
            bias: (
                self.bias.0.random(rng),
                self.bias.1.random(rng),
                self.bias.2.random(rng),
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
            axis_constraint: (
                self.axis_constraint.0.random(rng),
                self.axis_constraint.1.random(rng),
                self.axis_constraint.2.random(rng),
            ),
            influenced_by: (
                self.influenced_by.0.random(rng),
                self.influenced_by.1.random(rng),
            ),
            noclip: rng.gen::<f32>().into(),
            energy: self.energy.random(rng),
            hand_down_seed: rng.gen::<f32>().into(),
            rules: self.rules.random(rng),
            color_index: rng.gen_range(0, 16).into(),
        }
    }
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
            upper_range_bound: 15.0.into(),
            upper_weight_bound: 100.0.into(),
        }
    }

    fn random(&self, rng: &mut impl Rng) -> Self {
        OIDERuleSet {
            rules: self
                .rules
                .iter()
                .map(|rule| rule.random(rng, *self.upper_range_bound, *self.upper_weight_bound))
                .collect(),
            upper_weight_bound: self.upper_weight_bound.clone(),
            upper_range_bound: self.upper_range_bound.clone(),
        }
    }
}
impl OIDEContextRule {
    fn new_with_size(
        index_count: usize,
        context_count: usize,
        replacement_count: usize,
    ) -> OIDEContextRule {
        OIDEContextRule {
            context: BoundedIdxVec::new(index_count, context_count),
            range: 0.0.into(),
            weight: 1.0.into(),
            persist: true.into(),
            replacement: BoundedIdxVec::new(index_count, replacement_count),
        }
    }
    fn random(&self, rng: &mut impl Rng, upper_range_bound: f32, upper_weight_bound: f32) -> Self {
        OIDEContextRule {
            context: self.context.random(rng),
            range: rng
                .sample(Uniform::new_inclusive(0.0, upper_range_bound))
                .into(),
            weight: rng
                .sample(Uniform::new_inclusive(0.0, upper_weight_bound))
                .into(),
            persist: rng.gen::<f32>().into(),
            replacement: self.replacement.random(rng),
        }
    }
}
