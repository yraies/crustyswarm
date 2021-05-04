extern crate crustswarm_lib as crustswarm;
use std::{convert::TryFrom, fs::File, io::Read};

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

#[test]
fn oide_genome() {
    use std::io::Write;
    //spec art rule context replacement
    let oidegnome = crustswarm::swarm::oide_genome::OIDESwarmGenome::new(2, 3, 8, 1, 3);
    let oidegnome = oidegnome.random(&mut rand::thread_rng());

    let mut file = std::fs::File::create("genome.oide.json")
        .map_err(|e| e.to_string())
        .unwrap();
    file.write_all(
        serde_json::to_string_pretty(&oidegnome)
            .map_err(|e| e.to_string())
            .unwrap()
            .as_bytes(),
    )
    .map_err(|e| e.to_string())
    .unwrap();

    let oideresult = crustswarm::swarm::genome::SwarmGenome::from(oidegnome);

    let mut file2 = std::fs::File::create("result.genome.json")
        .map_err(|e| e.to_string())
        .unwrap();
    file2
        .write_all(
            serde_json::to_string_pretty(&oideresult)
                .map_err(|e| e.to_string())
                .unwrap()
                .as_bytes(),
        )
        .map_err(|e| e.to_string())
        .unwrap();

    let json_str = std::fs::read_to_string("result.genome.json").unwrap();
    let genome: crustswarm::swarm::genome::SwarmGenome = serde_json::from_str(&json_str).unwrap();
    dbg!(genome.terrain_size);
}
