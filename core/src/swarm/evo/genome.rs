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
    pub artifact_map: Vec<Fixed<usize>>,
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
    pub noclip: FloatyBool,
    pub energy: OIDEEnergy,
    pub hand_down_seed: FloatyBool,
    pub rules: OIDERuleSet,
    pub color_index: Fixed<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, AllOIDETraits)]
pub struct OIDEEnergy {
    pub on_movement: (BoundedFactor, BoundedFactor),
    pub on_zero: (BoundedFactor, BoundedFactor, IndexMultiset),
    pub on_replication: (BoundedFactor, BoundedFactor),
    pub for_offspring: (BoundedFactor, BoundedFactor, BoundedFactor),
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
    OIDECrossover,
    OIDEBoundApplication,
    OIDEZero,
)]
pub struct OIDEContextRule {
    pub context: IndexMultiset,
    pub range: BoundedFactor,
    pub weight: BoundedFactor,
    pub persist: FloatyBool,
    pub replacement: IndexMultiset,
}

impl OIDEOpposite for OIDEContextRule {
    fn opposite(&self, midpoint: Option<&Self>) -> Self {
        OIDEContextRule {
            context: self.replacement.clone(),
            replacement: self.context.clone(),
            range: self.range.opposite(match midpoint {
                Some(ref m) => Some(&m.range),
                None => None,
            }),
            weight: self.weight.opposite(match midpoint {
                Some(ref m) => Some(&m.weight),
                None => None,
            }),
            persist: self.persist.opposite(match midpoint {
                Some(ref m) => Some(&m.persist),
                None => None,
            }),
        }
    }
}

impl OIDESwarmGenome {
    pub fn new(spec_count: usize, art_count: usize, rule_count: usize) -> OIDESwarmGenome {
        OIDESwarmGenome {
            species_count: spec_count.into(),
            artifact_count: art_count.into(),
            rule_count: rule_count.into(),
            species_map: (0..spec_count)
                .map(|idx| OIDESpecies::new_with_size(spec_count, art_count, rule_count, idx))
                .collect(),
            artifact_map: (spec_count..(spec_count + art_count))
                .map(|idx| idx.into())
                .collect(), // colors
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
            terrain_size: 40.into(),
            terrain_spacing: 6.0.into(),
        }
    }
}

impl OIDESpecies {
    pub fn new_with_size(
        species_count: usize,
        artifact_count: usize,
        rule_count: usize,
        index: usize,
    ) -> OIDESpecies {
        OIDESpecies {
            index: index.into(),
            separation: BoundedFactor::new_with_bounds(0.0, 2.0, 0.0),
            alignment: BoundedFactor::new_with_bounds(0.0, 2.0, 0.0),
            cohesion: BoundedFactor::new_with_bounds(0.0, 2.0, 0.0),
            randomness: BoundedFactor::new_with_bounds(0.0, 1.0, 0.0),
            center: BoundedFactor::new_with_bounds(0.0, 0.1, 0.0),
            floor: BoundedFactor::new_with_bounds(0.0, 0.1, 0.0),
            bias: (
                BoundedFactor::new_with_bounds(-0.5, 0.5, 0.0),
                BoundedFactor::new_with_bounds(-0.5, 0.5, 0.0),
                BoundedFactor::new_with_bounds(-0.5, 0.5, 0.0),
            ),
            gradient: BoundedFactor::new_with_bounds(0.0, 1.0, 0.0),
            normal: BoundedFactor::new_with_bounds(0.0, 1.0, 0.0),
            slope: BoundedFactor::new_with_bounds(0.0, 0.5, 0.0),
            normal_speed: BoundedFactor::new_with_bounds(0.0, 1.0, 0.5),
            max_speed: BoundedFactor::new_with_bounds(0.0, 3.0, 1.0),
            max_acceleration: BoundedFactor::new_with_bounds(0.0, 3.0, 1.0),
            pacekeeping: BoundedFactor::new_with_bounds(0.0, 1.0, 0.0),
            view_distance: BoundedFactor::new_with_bounds(0.0, 200.0, 50.0),
            view_angle: BoundedFactor::new_with_bounds(1.0, 359.9, 270.0),
            sep_distance: BoundedFactor::new_with_bounds(0.0, 50.0, 10.0),
            axis_constraint: (
                BoundedFactor::new_with_bounds(0.0, 1.0, 1.0),
                BoundedFactor::new_with_bounds(0.0, 1.0, 1.0),
                BoundedFactor::new_with_bounds(0.0, 1.0, 1.0),
            ),
            influenced_by: (
                BoundedFactorVec::new(-1.0, 1.0, species_count),
                BoundedFactorVec::new(-1.0, 1.0, artifact_count),
            ),
            noclip: false.into(),
            energy: OIDEEnergy::new_with_size(species_count + artifact_count),
            hand_down_seed: false.into(),
            rules: OIDERuleSet::new_with_size(species_count + artifact_count, rule_count),
            color_index: index.into(),
        }
    }
}

impl OIDEEnergy {
    pub fn new_with_size(index_count: usize) -> OIDEEnergy {
        OIDEEnergy {
            on_movement: (
                BoundedFactor::new_with_bounds(0.0, 3.0 - (f32::EPSILON * 4.0), 0.0),
                BoundedFactor::new_with_bounds(0.0, 10.0, 0.1),
            ),
            on_zero: (
                BoundedFactor::new_with_bounds(0.0, 3.0 - (f32::EPSILON * 4.0), 0.0),
                BoundedFactor::new_with_bounds(0.0, 10.0, 0.1),
                IndexMultiset::new_with_size(index_count),
            ),
            on_replication: (
                BoundedFactor::new_with_bounds(0.0, 5.0 - (f32::EPSILON * 4.0), 0.0),
                BoundedFactor::new_with_bounds(0.0, 10.0, 0.1),
            ),
            for_offspring: (
                BoundedFactor::new_with_bounds(0.0, 4.0 - (f32::EPSILON * 4.0), 0.0),
                BoundedFactor::new_with_bounds(0.0, 10.0, 0.0),
                BoundedFactor::new_with_bounds(0.0, 10.0, 0.1),
            ),
        }
    }
}

impl OIDERuleSet {
    pub fn new_with_size(index_count: usize, rule_count: usize) -> OIDERuleSet {
        OIDERuleSet {
            rules: vec![OIDEContextRule::new_with_size(index_count); rule_count],
            upper_range_bound: 50.0.into(),
            upper_weight_bound: 100.0.into(),
        }
    }
}

impl OIDEContextRule {
    pub fn new_with_size(index_count: usize) -> OIDEContextRule {
        OIDEContextRule {
            context: IndexMultiset::new_with_size(index_count),
            range: BoundedFactor::new_with_bounds(0.0, 50.0, 0.0),
            weight: BoundedFactor::new_with_bounds(0.0, 100.0, 0.0),
            persist: true.into(),
            replacement: IndexMultiset::new_with_size(index_count),
        }
    }
}
