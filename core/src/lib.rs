extern crate cgmath;
extern crate core;
extern crate rand;
extern crate rayon;
extern crate serde;

use swarm::grammar::SwarmGrammar;
use swarm::world::World;

pub mod io;
pub mod swarm;
mod utils;

#[allow(dead_code)]
pub fn agents_to_arr(grammar: &SwarmGrammar) -> Vec<f32> {
    let ags: Vec<_> = grammar.world.get_all_agents().collect();
    let count = ags.len();
    let mut out_vec = Vec::with_capacity(count * 4);
    for agent in ags {
        out_vec.push(agent.position.x);
        out_vec.push(agent.position.y);
        out_vec.push(agent.position.z);
        out_vec.push(agent.species_index.0 as f32);
    }

    out_vec
}

#[allow(dead_code)]
pub fn buoys_to_arr(grammar: &SwarmGrammar) -> Vec<f32> {
    let buoys: Vec<_> = grammar.world.get_all_buoys().collect();
    let count = buoys.len();
    let mut out_vec = Vec::with_capacity(count * 3);
    for buoy in buoys {
        out_vec.push(buoy.position.x);
        out_vec.push(buoy.position.y);
        out_vec.push(buoy.position.z);
    }

    out_vec
}

#[allow(dead_code)]
pub fn artifacts_to_arr(grammar: &SwarmGrammar) -> Vec<f32> {
    let arts: Vec<_> = grammar.world.get_all_artifacts().collect();
    let count = arts.len();
    let mut out_vec = Vec::with_capacity(count * 4);
    for art in arts {
        out_vec.push(art.position.x);
        out_vec.push(art.position.y);
        out_vec.push(art.position.z);
        out_vec.push(art.artifact_index.0 as f32);
    }

    out_vec
}

#[allow(dead_code)]
pub fn agents_to_arr2(grammar: &SwarmGrammar) -> Vec<([f32; 3], usize)> {
    let ags: Vec<_> = grammar.world.get_all_agents().collect();
    let count = ags.len();
    let mut out_vec = Vec::with_capacity(count * 4);
    for agent in ags {
        out_vec.push((
            [agent.position.x, agent.position.y, agent.position.z],
            grammar.genome.get_species(agent).color_index,
        ));
    }

    out_vec
}

#[allow(dead_code)]
pub fn buoys_to_arr2(grammar: &SwarmGrammar) -> Vec<[f32; 3]> {
    let buoys: Vec<_> = grammar.world.get_all_buoys().collect();
    let count = buoys.len();
    let mut out_vec = Vec::with_capacity(count * 3);
    for buoy in buoys {
        out_vec.push([buoy.position.x, buoy.position.y, buoy.position.z]);
    }

    out_vec
}

#[allow(dead_code)]
pub fn artifacts_to_arr2(grammar: &SwarmGrammar) -> Vec<([f32; 3], usize)> {
    let ags: Vec<_> = grammar.world.get_all_artifacts().collect();
    let count = ags.len();
    let mut out_vec = Vec::with_capacity(count * 4);
    for artifact in ags {
        out_vec.push((
            [
                artifact.position.x,
                artifact.position.y,
                artifact.position.z,
            ],
            grammar.genome.get_artifact_type(artifact).color_index,
        ));
    }

    out_vec
}
