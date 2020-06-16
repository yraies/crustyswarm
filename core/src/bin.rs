extern crate crustswarm_lib as crustswarm;
use crustswarm::io;
use crustswarm::swarm::distribution::{StartAgents, StartBuoys, StartDistribution};
use crustswarm::swarm::grammar::SwarmTemplate;
use crustswarm::swarm::ruleset::{ContextRule, Replacement, RuleSet, RuleStrategy};
use crustswarm::swarm::species::*;
use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;

fn main() {
    println!(
        "{}",
        serde_json::to_string_pretty(&crustswarm::swarm::dummies::example_dummy_genome()).unwrap()
    );
    println!("##########");
    println!(
        "{:?}",
        &serde_json::to_string_pretty(&crustswarm::swarm::genome::SwarmGenome::try_from(
            crustswarm::swarm::dummies::example_dummy_genome()
        ))
    );
    println!("##########");

    let mut file = File::open("new_config.json").unwrap();
    let mut json_str = String::new();
    file.read_to_string(&mut json_str).unwrap();
    let genome: crustswarm::swarm::genome::SwarmGenome = serde_json::from_str(&json_str).unwrap();
    println!("\n# From ###\n\n");
    println!("{}", &json_str);
    println!("\n# To #####\n\n");
    println!("{:#?}", &genome);

    return;
    let test = 7;
    let spec = Species::new(
        1.0,
        1.0,
        1.0,
        1.0,
        1.0,
        1.0,
        1.0,
        [0.0, 0.0, 0.0],
        vec![(0, 0.0)],
        1.0,
        true,
        InitialEnergy::Inherit(1.0),
        DepletionEnergy::None,
        ZeroEnergy::Alive,
        false,
    );
    let ruleset0 = RuleSet {
        input: 0,
        rules: vec![
            ContextRule {
                weight: 0.1,
                replacement: Replacement::Spread(0, 6, 0),
                ..Default::default()
            },
            ContextRule {
                weight: 0.8,
                replacement: Replacement::Simple(vec![0]),
                ..Default::default()
            },
            ContextRule {
                weight: 0.1,
                replacement: Replacement::Multi(vec![
                    Replacement::Simple(vec![0]),
                    Replacement::Buoy,
                ]),
                ..Default::default()
            },
        ],
    };
    let temp: SwarmTemplate = SwarmTemplate {
        species: vec![spec],
        rule_sets: vec![ruleset0],
        start_dist: StartDistribution {
            start_agents: StartAgents::Multi(vec![
                StartAgents::Single(0.0, 0.0, 0),
                StartAgents::Single(0.0, 0.0, 1),
            ]),
            start_buoys: StartBuoys::Grid(5, 10.0),
        },
        strategy: RuleStrategy::new(1),
    };
    io::template_to_sout(&temp);
}
