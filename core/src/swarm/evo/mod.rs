use r_oide::{atoms::BoundedIdxVec, traits::Evaluatable};
use rand::prelude::*;

use super::grammar::SwarmGrammar;

pub mod conversion;
pub mod genome;

impl Evaluatable<SwarmGrammar> for genome::OIDESwarmGenome {
    type Params = (u64, u64);
    fn eval(&self, params: &Self::Params) -> SwarmGrammar {
        let mut rnd = StdRng::seed_from_u64(params.0);
        let genome = super::genome::SwarmGenome::from(self);
        //println!("{:?}", &genome);
        let mut sg = SwarmGrammar::from(genome, &mut rnd);
        //println!("{:?}", &sg);
        for _ in 0..params.1 {
            sg.step(&mut rnd);
        }
        sg
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
