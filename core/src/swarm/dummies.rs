use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
#[serde(transparent)]
pub struct Identifier(pub String);

type Factor = f32;
type InfluenceFactor = Factor;

#[derive(Debug, Serialize, Deserialize)]
pub struct DummySwarmGenome {
    pub species_map: HashMap<String, DummySpecies>,
    pub artifact_map: HashMap<String, super::genome::ArtifactType>,
    pub start_dist: DummyDistribution,
    pub strategy: DummyApplicationStrategy,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DummySpecies {
    pub separation: Factor,
    pub alignment: Factor,
    pub cohesion: Factor,
    pub randomness: Factor,
    pub center: Factor,
    pub max_speed: Factor,
    pub view_distance: Factor,
    pub sep_distance: Factor,
    pub axis_constraint: [Factor; 3],
    pub influence: HashMap<Identifier, InfluenceFactor>,
    pub mass: Factor,
    pub noclip: bool,
    pub offspring_energy: super::genome::OffspringEnergy,
    pub depletion_energy: super::genome::DepletionEnergy,
    pub zero_energy: super::genome::ZeroEnergy,
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
    pub persist: bool,
    pub replacement: DummyReplacement,
}

impl Default for DummyContextRule {
    fn default() -> DummyContextRule {
        DummyContextRule {
            context: Vec::new(),
            weight: 1.0,
            persist: true,
            replacement: DummyReplacement::None,
            range: 5.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub enum DummyDistribution {
    Multi(Vec<DummyDistribution>),
    Single([f32; 3], Identifier),
    Singularity([f32; 3], Vec<(usize, Identifier)>),
    Grid(usize, f32, Identifier),
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct DummyApplicationStrategy {
    pub every: usize,
    #[serde(default)]
    pub offset: Option<usize>,
}

impl Identifier {
    fn new(name: &str) -> Identifier {
        Identifier(name.to_string())
    }
}

pub fn example_dummy_genome() -> DummySwarmGenome {
    let mut species_map = HashMap::new();
    let mut artifact_map = HashMap::new();

    let mut species = DummySpecies::default();

    species.rules.push(DummyContextRule {
        context: vec![],
        range: 1.0,
        weight: 2.0,
        persist: true,
        replacement: DummyReplacement::Simple(vec![Identifier::new("a0")]),
    });
    species.influence.insert(Identifier::new("seed"), 2.0);

    species_map.insert("seed".to_string(), species);

    artifact_map.insert("a0".to_string(), super::genome::ArtifactType::default());

    DummySwarmGenome {
        strategy: DummyApplicationStrategy {
            every: 10,
            offset: None,
        },
        start_dist: DummyDistribution::Single([0.0, 0.0, 0.0], Identifier::new("seed")),
        species_map,
        artifact_map,
    }
}
