use super::Val;
use serde::Deserialize;
use serde::Serialize;
use swarm::SpeciesIndex;

#[derive(Debug, Serialize, Deserialize)]
pub struct Species {
    pub separation: Val,
    pub alignment: Val,
    pub cohesion: Val,
    pub randomness: Val,
    pub center: Val,
    pub max_speed: Val,
    pub view_distance: Val,
    pub sep_distance: Val,
    pub axis_constraint: [Val; 3],
    pub influence: Vec<(SpeciesIndex, Val)>,
    pub weight: Val,
    pub energy_strategy: EnergyStrategy,
}

impl Species {
    pub fn new(
        separation: Val,
        alignment: Val,
        cohesion: Val,
        randomness: Val,
        center: Val,
        max_speed: Val,
        sep_distance: Val,
        axis_constraint: [Val; 3],
        influence: Vec<(SpeciesIndex, Val)>,
        weight: Val,
        energy_strategy: EnergyStrategy,
    ) -> Species {
        Species {
            separation,
            alignment,
            cohesion,
            randomness,
            center,
            max_speed,
            view_distance: 80.0,
            sep_distance,
            axis_constraint,
            influence,
            weight,
            energy_strategy,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EnergyStrategy {
    Constant(Val),
    Distance(Val),
    None,
}
