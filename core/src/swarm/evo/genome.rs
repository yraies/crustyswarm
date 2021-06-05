#![allow(dead_code)]

use cgmath::Vector3;
use serde::{Deserialize, Serialize};

use super::super::genome::{
    replacement::ApplicationStrategy, Distribution, SpeciesIndex, SurroundingIndex,
};

use derive_diff::*;
use r_oide::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, AllOIDETraits)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, AllOIDETraits)]
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
    pub noclip: Fixed<bool>,
    pub energy: OIDEEnergy,
    pub hand_down_seed: FloatyBool,
    pub rules: OIDERuleSet,
    pub color_index: Fixed<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, AllOIDETraits)]
pub struct OIDEEnergy {
    pub on_movement: BoundedFactor,
    pub on_zero: (BoundedFactor, BoundedIdxVec),
    pub on_replication: BoundedFactor,
    pub for_offspring: BoundedFactor,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, AllOIDETraits)]
pub struct OIDERuleSet {
    pub rules: Vec<OIDEContextRule>,
    pub upper_weight_bound: Fixed<f32>,
    pub upper_range_bound: Fixed<f32>,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Default,
    PartialEq,
    Differentiable,
    OIDEAdd,
    OIDEDiff,
    OIDEScale,
    OIDERandomize,
    OIDEBoundApplication,
)]
pub struct OIDEContextRule {
    pub context: BoundedIdxVec,
    pub range: BoundedFactor,
    pub weight: BoundedFactor,
    pub persist: FloatyBool,
    pub replacement: BoundedIdxVec,
}

impl OIDEOpposite for OIDEContextRule {
    fn opposite(&self) -> Self {
        OIDEContextRule {
            context: self.replacement.clone(),
            range: self.range.clone(),
            weight: self.weight.clone(),
            persist: self.persist.clone(),
            replacement: self.context.clone(),
        }
    }
}

impl OIDESwarmGenome {
    pub fn new(
        spec_count: usize,
        art_count: usize,
        rule_count: usize,
        rule_size: usize,
    ) -> OIDESwarmGenome {
        OIDESwarmGenome {
            species_count: spec_count.into(),
            artifact_count: art_count.into(),
            rule_count: rule_count.into(),
            species_map: vec![
                OIDESpecies::new_with_size(
                    spec_count, art_count, rule_count, rule_size,
                );
                spec_count
            ],
            artifact_map: BoundedIdxVec::new_by_idx_count(15, art_count), // colors
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
        rule_size: usize,
    ) -> OIDESpecies {
        OIDESpecies {
            index: 0.into(),
            separation: BoundedFactor::new_with_bounds(0.0, 2.0, 0.0),
            alignment: BoundedFactor::new_with_bounds(0.0, 2.0, 0.0),
            cohesion: BoundedFactor::new_with_bounds(0.0, 2.0, 0.0),
            randomness: BoundedFactor::new_with_bounds(0.0, 2.0, 0.0),
            center: BoundedFactor::new_with_bounds(0.0, 2.0, 0.0),
            floor: BoundedFactor::new_with_bounds(0.0, 2.0, 0.0),
            bias: (
                BoundedFactor::new_with_bounds(-1.0, 1.0, 0.0),
                BoundedFactor::new_with_bounds(-1.0, 1.0, 0.0),
                BoundedFactor::new_with_bounds(-1.0, 1.0, 0.0),
            ),
            gradient: BoundedFactor::new_with_bounds(0.0, 2.0, 0.0),
            normal: BoundedFactor::new_with_bounds(0.0, 2.0, 0.0),
            slope: BoundedFactor::new_with_bounds(0.0, 2.0, 0.0),
            normal_speed: BoundedFactor::new_with_bounds(0.0, 2.0, 0.5),
            max_speed: BoundedFactor::new_with_bounds(0.0, 2.0, 1.0),
            max_acceleration: BoundedFactor::new_with_bounds(0.0, 2.0, 1.0),
            pacekeeping: BoundedFactor::new_with_bounds(0.0, 2.0, 0.0),
            view_distance: BoundedFactor::new_with_bounds(0.0, 200.0, 50.0),
            view_angle: BoundedFactor::new_with_bounds(1.0, 359.9, 270.0),
            sep_distance: BoundedFactor::new_with_bounds(0.0, 50.0, 10.0),
            axis_constraint: (
                BoundedFactor::new_with_bounds(0.0, 2.0, 0.0),
                BoundedFactor::new_with_bounds(0.0, 2.0, 0.0),
                BoundedFactor::new_with_bounds(0.0, 2.0, 0.0),
            ),
            influenced_by: (
                BoundedFactorVec::new(-2.0, 2.0, species_count),
                BoundedFactorVec::new(-2.0, 2.0, artifact_count),
            ),
            noclip: false.into(),
            energy: OIDEEnergy::new_with_size(species_count + artifact_count, rule_size),
            hand_down_seed: false.into(),
            rules: OIDERuleSet::new_with_size(
                species_count + artifact_count,
                rule_count,
                rule_size,
            ),
            color_index: 0.into(),
        }
    }
}

impl OIDEEnergy {
    fn new_with_size(index_count: usize, replacement_count: usize) -> OIDEEnergy {
        OIDEEnergy {
            on_movement: BoundedFactor::new_with_bounds(0.0, 10.0, 0.1),
            on_zero: (
                BoundedFactor::new_with_bounds(0.0, 10.0, 0.1),
                BoundedIdxVec::new_by_idx_count(index_count, replacement_count),
            ),
            on_replication: BoundedFactor::new_with_bounds(0.0, 10.0, 0.1),
            for_offspring: BoundedFactor::new_with_bounds(0.0, 10.0, 0.1),
        }
    }
}

impl OIDERuleSet {
    fn new_with_size(index_count: usize, rule_count: usize, rule_size: usize) -> OIDERuleSet {
        OIDERuleSet {
            rules: vec![OIDEContextRule::new_with_size(index_count, rule_size); rule_count],
            upper_range_bound: 50.0.into(),
            upper_weight_bound: 100.0.into(),
        }
    }
}

impl OIDEContextRule {
    fn new_with_size(index_count: usize, rule_size: usize) -> OIDEContextRule {
        OIDEContextRule {
            context: BoundedIdxVec::new_by_idx_count(index_count, rule_size),
            range: BoundedFactor::new_with_bounds(0.0, 50.0, 0.0),
            weight: BoundedFactor::new_with_bounds(0.0, 100.0, 0.0),
            persist: true.into(),
            replacement: BoundedIdxVec::new_by_idx_count(index_count, rule_size),
        }
    }
}
