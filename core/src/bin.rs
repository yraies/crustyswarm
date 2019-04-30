extern crate crustswarm_lib as crustswarm;
use crustswarm::io;
use crustswarm::swarm::grammar::{SwarmGrammar, SwarmTemplate};
use crustswarm::swarm::ruleset::RuleStrategy;
use crustswarm::swarm::ruleset::RuleStrategy::Every;
use crustswarm::swarm::species::Species;
use crustswarm::swarm::StartDistribution;

fn main() {
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
    );
    let temp: SwarmTemplate = SwarmTemplate {
        species: vec![spec],
        rule_sets: vec![],
        start_dist: StartDistribution::Multi(vec![
            StartDistribution::Single(0),
            StartDistribution::Single(1),
        ]),
        strategy: RuleStrategy::Every(1, 1),
    };
    io::template_to_file(&temp, "newtest.json");
}
