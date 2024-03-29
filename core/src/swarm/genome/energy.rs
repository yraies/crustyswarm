use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Energy {
    pub on_movement: MovementEnergy,
    pub on_zero: ZeroEnergy,
    pub on_replication: ReplicationEnergy,
    pub for_offspring: OffspringEnergy,
}

//
// ENUMS
//

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum ReplicationEnergy {
    Constant(f32),
    Count(f32),
    PropRel,        // parent.energy = offspring.energy
    PropConst(f32), // remaining energy after offspring with constant offset
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum MovementEnergy {
    Constant(f32),
    Distance(f32),
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ZeroEnergy {
    Die,
    Replace(u16, super::replacement::Replacement),
    Live,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum OffspringEnergy {
    Constant(f32),
    Inherit(f32),
    PropRel(f32),
    PropConst(f32, f32),
}

//
// IMPLS
//

impl ReplicationEnergy {
    pub fn get(&self, current: f32, count: usize, energy_per_offspring: f32) -> f32 {
        match self {
            Self::Constant(value) => current - *value,
            Self::Count(factor) => current - count as f32 * factor,
            Self::PropRel => energy_per_offspring,
            Self::PropConst(offset) => {
                f32::max(0.0, current - offset - count as f32 * energy_per_offspring)
            }
            Self::None => current,
        }
    }
    pub fn get_param(&self) -> f32 {
        match self {
            ReplicationEnergy::Constant(a) => *a,
            ReplicationEnergy::Count(a) => *a,
            ReplicationEnergy::PropRel => 0.0,
            ReplicationEnergy::PropConst(a) => *a,
            ReplicationEnergy::None => 0.0,
        }
    }
}

impl MovementEnergy {
    pub fn get(&self, velocity: f32) -> f32 {
        match self {
            Self::Constant(value) => *value,
            Self::Distance(factor) => velocity * factor,
            Self::None => 0.0,
        }
    }
    pub fn get_param(&self) -> f32 {
        match self {
            MovementEnergy::Constant(a) => *a,
            MovementEnergy::Distance(a) => *a,
            MovementEnergy::None => 0.0,
        }
    }
}

use super::super::actor::{Agent, Artifact};
use super::Species;
impl ZeroEnergy {
    pub fn is_alive(&self, energy: f32) -> bool {
        match self {
            Self::Live => true,
            _ => energy > 0.0,
        }
    }
    pub fn replacement(
        &self,
        parent: &Agent,
        parent_species: &Species,
        uid_gen: &mut crate::utils::UidGen,
    ) -> (Vec<Agent>, Vec<Artifact>) {
        match self {
            Self::Replace(energy, replacement) => {
                let mut reps = replacement.replace_agent_unchecked(
                    parent,
                    parent_species,
                    *energy as f32,
                    uid_gen,
                );
                for ag in &mut reps.0 {
                    ag.last = None;
                }
                reps
            }
            _ => (vec![], vec![]),
        }
    }
    pub fn get_param(&self) -> u16 {
        match self {
            ZeroEnergy::Die => 0,
            ZeroEnergy::Replace(a, _) => *a,
            ZeroEnergy::Live => 0,
        }
    }
}

impl OffspringEnergy {
    pub fn get(&self, current: f32, count: usize, parent_persists: bool) -> f32 {
        assert!(current.is_finite(), "Not finite! Was {:?}", current);
        let ret = match self {
            OffspringEnergy::Constant(value) => *value,
            OffspringEnergy::Inherit(factor) => current * factor,
            OffspringEnergy::PropRel(offset) => {
                if count == 0 {
                    0.0
                } else {
                    let newcount = if parent_persists { count + 1 } else { count };
                    (current - offset) / newcount as f32
                }
            }
            OffspringEnergy::PropConst(offset, ammount) => {
                if count > 0 {
                    f32::min(*ammount, (current - offset) / count as f32)
                } else {
                    0.0
                }
            }
        }
        .clamp(-f32::MAX, f32::MAX);
        assert!(
            ret.is_finite(),
            "Not finite! Was {:?} due to {:?} and {:?}",
            ret,
            current,
            self
        );
        ret
    }
    pub fn get_param(&self) -> f32 {
        match self {
            OffspringEnergy::Constant(a) => *a,
            OffspringEnergy::Inherit(a) => *a,
            OffspringEnergy::PropRel(a) => *a,
            OffspringEnergy::PropConst(a, _) => *a,
        }
    }
}

//
// DEFAULTS
//

impl Default for ReplicationEnergy {
    fn default() -> ReplicationEnergy {
        ReplicationEnergy::Constant(1.0)
    }
}

impl Default for MovementEnergy {
    fn default() -> MovementEnergy {
        MovementEnergy::Constant(1.0)
    }
}

impl Default for ZeroEnergy {
    fn default() -> ZeroEnergy {
        ZeroEnergy::Die
    }
}

impl Default for OffspringEnergy {
    fn default() -> OffspringEnergy {
        OffspringEnergy::Inherit(1.0)
    }
}
