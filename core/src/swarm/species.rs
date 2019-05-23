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
    pub initial_energy: InitialEnergy,
    pub depletion_energy: DepletionEnergy,
    pub zero_energy: ZeroEnergy,
    pub hand_down_seed: bool,
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
        initial_energy: InitialEnergy,
        depletion_energy: DepletionEnergy,
        zero_energy: ZeroEnergy,
        hand_down_seed: bool,
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
            initial_energy,
            depletion_energy,
            zero_energy,
            hand_down_seed,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DepletionEnergy {
    Constant(Val),
    Distance(Val),
    None,
}

impl Species {
    pub fn get_spawn_energy(&self, parent_energy: Val) -> Val {
        match self.initial_energy {
            InitialEnergy::Constant(val) => val,
            InitialEnergy::Inherit(val) => val * parent_energy,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InitialEnergy {
    Constant(Val),
    Inherit(Val),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ZeroEnergy {
    Die,
    Alive,
}
