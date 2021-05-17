extern crate crustswarm_lib as crustswarm;

use std::env;

use crustswarm::swarm::oide_genome::OIDESwarmGenome;

fn main() {
    let cmd = env::args().skip(1).next();

    match cmd.as_ref().map(String::as_str) {
        Some("convert") => {
            let oide_path = env::args()
                .skip(2)
                .next()
                .expect("Oide config to convert required!");
            println!("converting {} to concrete genome", oide_path);
            crustswarm::io::genome_to_file(
                &crustswarm::swarm::genome::SwarmGenome::from(
                    &crustswarm::io::oide_genome_from_file(oide_path),
                ),
                "converted.genome.json",
            )
            .map(|err| println!("Error occured while converting: {:?}", err));
        }
        Some("generate_zero") => {
            std::fs::write(
                "zero.oide.json",
                serde_json::to_string_pretty(&OIDESwarmGenome::new(2, 3, 3, 2, 4)).unwrap(),
            )
            .unwrap();
        }
        Some(a) => print!("Command {} unknown", a),
        None => {
            println!("Please provide some command")
        }
    };

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

    let oideresult = crustswarm::swarm::genome::SwarmGenome::from(&oidegnome);

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
