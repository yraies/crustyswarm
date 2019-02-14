extern crate cgmath;
extern crate rand;
extern crate rayon;

use cgmath::prelude::*;
use cgmath::Vector3;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use std::time::Instant;
use std::env;

use swarm::agent::Agent;
use swarm::grammar::SwarmGrammar;
use swarm::ruleset::RuleSet;
use swarm::species::Species;
use std::time::Duration;
use std::io::{Write, BufWriter};
use std::fs::File;
use swarm::Val;

mod swarm;
mod utils;


pub fn main() {
    let agent_count = if let Some(arg1) = env::args().nth(1) {
        let sth = arg1.parse().unwrap_or(10);
        println!("The agent count is {}", sth);
        sth
    } else { 10 };

    let iteration_count = if let Some(arg1) = env::args().nth(2) {
        let sth = arg1.parse().unwrap_or(50);
        println!("The iteration count is {}", sth);
        sth
    } else { 50 };

    let mut rnd = SmallRng::seed_from_u64(323_381_111u64);

    let mut agents = Vec::new();
    let v = Vector3::new(-1.0,-1.0,-1.0);
    for i in 1..=agent_count {
        let agent = Agent::mk_new((rnd.gen::<Vector3<Val>>() * 2.0) - v, (rnd.gen::<Vector3<Val>>() * 2.0) - v, 10.0, 1).unwrap();
        let agent2 = Agent::mk_new((rnd.gen::<Vector3<Val>>() * 2.0) - v, (rnd.gen::<Vector3<Val>>() * 2.0) - v, 10.0, 0).unwrap();
        agents.push(agent);
        agents.push(agent2);
    }


    let species = Species::new(0.5, 1.0, 2.5, 0.4,1.0, 7.0);

    let species2 = Species::new(0.5, 1.0, 2.5, 0.4,1.0, 5.0);

    let rule = RuleSet {
        input: 0,
        rules: vec![(vec!(1), 0.1), (vec!(0), 0.89), (vec!(0, 1), 0.01)],
    };

    let rule2 = RuleSet {
        input: 1,
        rules: vec![(vec!(0), 0.2), (vec!(1), 0.79), (vec!(0, 1), 0.01)],
    };


    let mut grammar = SwarmGrammar {
        agents,
        species: vec![species, species2],
        rule_sets: vec![rule, rule2],
    };

    let mut time_ctr = Duration::new(0,0);

//    println!("Initial State:");
//    println!("{:#.2?}\n", &grammar);
    for i in 1..=iteration_count {
        println!("Calculating iteration {} -- {} Agents", i, &grammar.agents.len());
//        println!("{:#.2?}", &grammar);
        let start = Instant::now();
        grammar.step(&mut rnd);
        println!("{:?}",start.elapsed());
        time_ctr += start.elapsed();

        if i%2 == 0 {
//            let mut w = BufWriter::new(File::create(format!("out{:02}.ply", i)).unwrap());
//            print_swarm(&grammar, &mut w);
        }
    }
//    println!("\nFinal State:");
//    println!("{:#.2?}", &grammar);
    println!("Total: {:?}", time_ctr);


}




fn print_swarm(grammar: &SwarmGrammar, writer : &mut BufWriter<File>) {
    let ags = grammar.get_agents();

    write!(writer,"ply\n").unwrap();
    write!(writer,"format ascii 1.0\n").unwrap();
    write!(writer,"element vertex {}\n", ags.len()).unwrap();
    write!(writer,"property float x\nproperty float y\nproperty float z\n").unwrap();
    write!(writer,"end_header\n").unwrap();

    for ag in ags {
        let (x,y,z) = ag.position.into();
        write!(writer,"{} {} {}\n", x, y, z).unwrap();
    }
/*    for ag in ags {
        let (x,y,z) = ag.velocity.into();
        write!(writer,"{} {} {}\n",x ,y ,z).unwrap();
    }*/
}