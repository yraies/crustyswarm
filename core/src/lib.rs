extern crate cgmath;
extern crate core;
extern crate rand;
extern crate rayon;
extern crate serde;

use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::time::Duration;
use std::time::Instant;

use cgmath::Vector3;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;

use swarm::actor::Agent;
use swarm::distribution::{StartAgents, StartBuoys, StartDistribution};
use swarm::grammar::SwarmGrammar;
use swarm::grammar::SwarmTemplate;
use swarm::ruleset::{Rule, RuleSet, RuleStrategy};
use swarm::species::*;
use swarm::Val;

pub mod io;
pub mod swarm;
mod utils;

pub fn main() {
    let agent_count = if let Some(arg1) = env::args().nth(1) {
        let sth = arg1.parse().unwrap_or(10);
        println!("The agent count is {}", sth);
        sth
    } else {
        10
    };

    let iteration_count = if let Some(arg1) = env::args().nth(2) {
        let sth = arg1.parse().unwrap_or(50);
        println!("The iteration count is {}", sth);
        sth
    } else {
        50
    };

    let (mut rnd, mut grammar) = gen_swarm(agent_count);

    let mut time_ctr = Duration::new(0, 0);

    //    println!("Initial State:");
    //    println!("{:#.2?}\n", &grammar);
    for i in 1..=iteration_count {
        //        println!("{:#.2?}", &grammar);
        let start = Instant::now();
        grammar.step(&mut rnd);
        println!("{:?}", start.elapsed());
        time_ctr += start.elapsed();

        if i % 2 == 0 {
            //            let mut w = BufWriter::new(File::create(format!("out{:02}.ply", i)).unwrap());
            //            print_swarm(&grammar, &mut w);
        }
    }
    let mut w = BufWriter::new(File::create(format!("out{:02}.ply", 100)).unwrap());
    io::print_swarm(&grammar, &mut w);
    //    println!("\nFinal State:");
    //    println!("{:#.2?}", &grammar);
    println!("Total: {:?}", time_ctr);
}

#[allow(dead_code)]
pub fn gen_swarm(agent_count: i32) -> (SmallRng, SwarmGrammar) {
    let mut rnd = SmallRng::seed_from_u64(323_381_111u64);
    let mut agents = Vec::new();
    let v = Vector3::new(-1.0, -1.0, -1.0);
    for _i in 1..=agent_count {
        let agent = Agent::mk_new(
            (rnd.gen::<Vector3<Val>>() * 2.0) - v,
            (rnd.gen::<Vector3<Val>>() * 2.0) - v,
            10.0,
            1,
        )
        .unwrap();
        let agent2 = Agent::mk_new(
            (rnd.gen::<Vector3<Val>>() * 2.0) - v,
            (rnd.gen::<Vector3<Val>>() * 2.0) - v,
            10.0,
            0,
        )
        .unwrap();
        agents.push(agent);
        agents.push(agent2);
    }
    let species = Species::new(
        1.0,
        1.5,
        1.2,
        0.3,
        0.1,
        2.0,
        5.0,
        [1.0, 1.0, 1.0],
        vec![(0, 1.0), (1, 1.0)],
        1.0,
        EnergyStrategy::None,
    );
    let species2 = Species::new(
        1.8,
        0.3,
        0.5,
        0.8,
        0.1,
        0.8,
        15.0,
        [1.0, 1.0, 1.0],
        vec![(0, 1.0), (1, 1.0)],
        1.0,
        EnergyStrategy::None,
    );
    let rule = RuleSet {
        input: 0,
        rules: vec![
            // Rule::new(vec![0], 0.98),
            // Rule::new(vec![1], 0.01),
            // Rule::new(vec![0, 1], 0.005),
            // Rule::new(vec![], 0.005),
        ],
    };
    let rule2 = RuleSet {
        input: 1,
        rules: vec![
            // Rule::new(vec![1], 0.945),
            // Rule::new(vec![0], 0.05),
            // Rule::new(vec![0, 1], 0.005),
        ],
    };
    let grammar = SwarmGrammar {
        agents,
        artifacts: vec![],
        buoys: vec![],
        template: SwarmTemplate {
            species: vec![species, species2],
            rule_sets: vec![rule, rule2],
            start_dist: StartDistribution {
                start_agents: StartAgents::Singularity(vec![(2, 0), (2, 1)]),
                start_buoys: StartBuoys::None,
            },
            strategy: RuleStrategy::Every(4, 4),
        },
    };
    (rnd, grammar)
}

#[allow(dead_code)]
pub fn agents_to_arr(grammar: &SwarmGrammar) -> Vec<f32> {
    let ags = grammar.get_agents();
    let count = ags.len();
    let mut out_vec = Vec::with_capacity(count * 4);
    for agent in ags {
        out_vec.push(agent.position.x);
        out_vec.push(agent.position.y);
        out_vec.push(agent.position.z);
        out_vec.push(agent.species_index as f32);
    }

    out_vec
}

#[allow(dead_code)]
pub fn buoys_to_arr(grammar: &SwarmGrammar) -> Vec<f32> {
    let buoys = grammar.get_buoys();
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
    let arts = grammar.get_artifacts();
    let count = arts.len();
    let mut out_vec = Vec::with_capacity(count * 4);
    for art in arts {
        out_vec.push(art.position.x);
        out_vec.push(art.position.y);
        out_vec.push(art.position.z);
        out_vec.push(art.a_type as f32);
    }

    out_vec
}
