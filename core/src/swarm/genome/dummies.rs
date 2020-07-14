use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Default)]
#[serde(transparent)]
pub struct Identifier(pub String);

type Factor = f32;
type InfluenceFactor = Factor;

#[derive(Debug, Serialize, Deserialize)]
pub struct DummySwarmGenome {
    pub species_map: HashMap<String, DummySpecies>,
    pub artifact_map: HashMap<String, super::ArtifactType>,
    pub start_dist: DummyDistribution,
    pub strategy: DummyApplicationStrategy,
    pub terrain: TerrainConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TerrainConfig {
    pub size: usize,
    pub influenced_by: HashMap<String, InfluenceFactor>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Urges {
    #[serde(default)]
    pub bias: [Factor; 3],
    pub separation: Factor,
    pub alignment: Factor,
    pub cohesion: Factor,
    pub randomness: Factor,
    pub center: Factor,
    pub pacekeeping: Factor,
    pub floor: Factor,
    pub gradient: Factor,
    pub normal: Factor,
    pub slope: Factor,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DummySpecies {
    pub urges: Urges,
    pub normal_speed: Factor,
    pub max_speed: Factor,
    pub max_acceleration: Factor,
    pub view_distance: Factor,
    pub sep_distance: Factor,
    #[serde(default = "crate::utils::unit")]
    pub axis_constraint: [Factor; 3],
    pub influenced_by: HashMap<Identifier, InfluenceFactor>,
    #[serde(default = "crate::utils::no")]
    pub noclip: bool,
    pub energy: DummyEnergy,
    #[serde(default = "crate::utils::no")]
    pub hand_down_seed: bool,
    pub rules: Vec<DummyContextRule>,
    pub color_index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DummyContextRule {
    pub context: Vec<Identifier>,
    pub range: f32,
    pub weight: f32,
    #[serde(default = "crate::utils::no")]
    pub persist: bool,
    pub replacement: DummyReplacement,
}

impl Default for DummyContextRule {
    fn default() -> DummyContextRule {
        DummyContextRule {
            context: Vec::new(),
            weight: 1.0,
            persist: false,
            replacement: DummyReplacement::None,
            range: 5.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum DummyReplacement {
    None,
    Simple(Vec<Identifier>),
    Multi(Vec<DummyReplacement>),
    Spread(Identifier, usize, usize),
}

impl Default for DummyReplacement {
    fn default() -> DummyReplacement {
        DummyReplacement::None
    }
}
impl DummyReplacement {
    pub fn contains(&self, other: &str) -> bool {
        match self {
            Self::None => false,
            Self::Simple(ids) => ids.iter().any(|id| id.0.eq(other)),
            Self::Spread(id, _, _) => id.0.eq(other),
            Self::Multi(reps) => reps.iter().any(|rep| rep.contains(other)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DummyDistribution {
    Multi(Vec<DummyDistribution>),
    Single([f32; 3], Identifier),
    Singularity([f32; 3], Vec<(usize, Identifier)>),
    Grid(usize, f32, Identifier),
}

impl DummyDistribution {
    pub fn contains(&self, other: &str) -> bool {
        match self {
            Self::Single(_, id) => id.0.eq(other),
            Self::Singularity(_, ids) => ids.iter().any(|(_, id)| id.0.eq(other)),
            Self::Grid(_, _, id) => id.0.eq(other),
            Self::Multi(reps) => reps.iter().any(|rep| rep.contains(other)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct DummyApplicationStrategy {
    pub every: usize,
    #[serde(default)]
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DummyEnergy {
    pub on_movement: super::energy::MovementEnergy,
    pub on_zero: DummyZeroEnergy,
    pub on_replication: super::energy::ReplicationEnergy,
    pub for_offspring: super::energy::OffspringEnergy,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum DummyZeroEnergy {
    Die,
    Replace(u16, DummyReplacement),
    Live,
}

impl Default for DummyZeroEnergy {
    fn default() -> DummyZeroEnergy {
        Self::Die
    }
}

impl Identifier {
    fn new(name: &str) -> Identifier {
        Identifier(name.to_string())
    }
}

pub fn example_dummy_genome() -> DummySwarmGenome {
    let mut species_map = HashMap::new();
    let mut artifact_map = HashMap::new();
    let mut terrain_map = HashMap::new();

    let mut species = DummySpecies::default();

    species.rules.push(DummyContextRule {
        context: vec![],
        range: 1.0,
        weight: 2.0,
        persist: true,
        replacement: DummyReplacement::Simple(vec![Identifier::new("a0")]),
    });
    species.influenced_by.insert(Identifier::new("seed"), 2.0);

    species_map.insert("seed".to_string(), species);

    artifact_map.insert("a0".to_string(), super::ArtifactType::default());

    terrain_map.insert("a0".to_string(), 0.0);

    DummySwarmGenome {
        strategy: DummyApplicationStrategy {
            every: 10,
            offset: None,
        },
        start_dist: DummyDistribution::Single([0.0, 0.0, 0.0], Identifier::new("seed")),
        species_map,
        artifact_map,
        terrain: TerrainConfig {
            size: 31,
            influenced_by: terrain_map,
        },
    }
}
