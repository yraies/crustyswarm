extern crate crustswarm_lib as crustswarm;
use crustswarm::io;
use crustswarm::swarm::distribution::{StartAgents, StartBuoys, StartDistribution};
use crustswarm::swarm::grammar::SwarmTemplate;
use crustswarm::swarm::ruleset::{Replacement, Rule, RuleSet, RuleStrategy};
use crustswarm::swarm::species::*;

fn main() {
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
            Rule(0.1, Replacement::Spread(0, 6, 0)),
            Rule(0.8, Replacement::Simple(vec![0])),
            Rule(
                0.1,
                Replacement::Multi(vec![Replacement::Simple(vec![0]), Replacement::Buoy]),
            ),
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
