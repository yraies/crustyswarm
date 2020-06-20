extern crate crustswarm_lib as crustswarm;
use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;

fn main() {
    println!(
        "{}",
        serde_json::to_string_pretty(&crustswarm::swarm::genome::dummies::example_dummy_genome())
            .unwrap()
    );
    println!("##########");
    println!(
        "{:?}",
        &serde_json::to_string_pretty(&crustswarm::swarm::genome::SwarmGenome::try_from(
            crustswarm::swarm::genome::dummies::example_dummy_genome()
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
}
