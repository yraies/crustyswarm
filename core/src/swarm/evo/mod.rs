use std::time::Duration;

use r_oide::{atoms::BoundedIdxVec, traits::Evaluatable};
use rand::prelude::*;

use super::grammar::SwarmGrammar;

pub mod conversion;
pub mod genome;

pub struct OIDESwarmParams {
    pub seed: u64,
    pub max_iterations: usize,
    pub timeout_hint: Duration,
}

pub struct OIDESwarmEvalInfo {
    pub iterations: usize,
}

impl Evaluatable<SwarmGrammar> for genome::OIDESwarmGenome {
    type Params = OIDESwarmParams;
    type EvalInfo = OIDESwarmEvalInfo;
    fn eval(&self, params: &Self::Params) -> (SwarmGrammar, Self::EvalInfo) {
        let mut rnd = StdRng::seed_from_u64(params.seed);
        let genome = super::genome::SwarmGenome::from(self);
        let mut sg = SwarmGrammar::from(genome, &mut rnd);
        let start_time = std::time::Instant::now();

        let mut iteration = 0;
        for _ in 0..params.max_iterations {
            sg.step(&mut rnd);
            iteration = iteration + 1;
            if start_time.elapsed() > params.timeout_hint {
                break;
            }
        }
        (
            sg,
            OIDESwarmEvalInfo {
                iterations: iteration,
            },
        )
    }
}

use crate::swarm::genome::{ArtifactIndex, SpeciesIndex, SurroundingIndex};

struct FlattenableIntoSurroundingVec;

impl FlattenableIntoSurroundingVec {
    pub fn flatten_into_surrounding_vec(
        vec: &BoundedIdxVec,
        species_count: &dyn AsRef<usize>,
    ) -> Vec<SurroundingIndex> {
        let foobar = vec
            .vec
            .iter()
            .flat_map(|bar| {
                if bar.is_active() {
                    if bar.value >= *species_count.as_ref() {
                        let art_id = bar.value - species_count.as_ref();
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
}
