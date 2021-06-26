extern crate crustswarm_lib as crustswarm;

use crustswarm::swarm::evo::genome::OIDESwarmGenome;
use r_oide::prelude::*;
use std::env;

fn main() {
    let cmd = env::args().skip(1).next();

    match cmd.as_ref().map(String::as_str) {
        Some("convert_to_genome") => {
            let oide_path = env::args()
                .skip(2)
                .next()
                .expect("Oide config to convert required!");
            let target_path = env::args()
                .skip(3)
                .next()
                .unwrap_or("converted.genome.json".to_string());
            println!(
                "converting {} to concrete genome {}",
                oide_path, target_path
            );
            crustswarm::io::genome_to_file(
                &crustswarm::swarm::genome::SwarmGenome::from(
                    &crustswarm::io::oide_genome_from_file(oide_path),
                ),
                target_path,
            )
            .map(|err| println!("Error occured while converting: {:?}", err));
        }
        Some("convert_to_oide") => {
            let path = env::args()
                .skip(2)
                .next()
                .expect("Oide config to convert required!");
            let target_path = env::args()
                .skip(3)
                .next()
                .unwrap_or("converted.oide.json".to_string());
            println!("converting {} to oide template {}", path, target_path);
            let genome = crustswarm::io::genome_from_file(path);
            let oide_genome = crustswarm::swarm::evo::genome::OIDESwarmGenome::from(&genome);
            crustswarm::io::oide_genome_to_file(&oide_genome, target_path)
                .map(|err| println!("Error occured while converting: {:?}", err));
        }
        Some("rebound_oide") => {
            let path = env::args()
                .skip(2)
                .next()
                .expect("Oide config to rebound required!");
            let target_path = env::args()
                .skip(3)
                .next()
                .unwrap_or("rebound.oide.json".to_string());
            println!("rebounding {} oide template to {}", path, target_path);
            let genome = crustswarm::io::oide_genome_from_file(path);

            let new_bound_genome = OIDESwarmGenome::new(
                *genome.species_count,
                *genome.artifact_count,
                *genome.rule_count,
            );

            crustswarm::io::oide_genome_to_file(
                &new_bound_genome.apply_bounds(&genome),
                target_path,
            )
            .map(|err| println!("Error occured while converting: {:?}", err));
        }
        Some("generate_zero") => {
            std::fs::write(
                "zero.oide.json",
                serde_json::to_string_pretty(&OIDESwarmGenome::new(2, 3, 3)).unwrap(),
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
    let oidegnome = crustswarm::swarm::evo::genome::OIDESwarmGenome::new(2, 3, 8);
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

#[test]
#[cfg(test)]
fn oide_genome2() -> Result<(), std::io::Error> {
    use r_oide::prelude::*;
    println!("Cur. Dir: {:?}", std::env::current_dir());
    let base_tree = crustswarm::io::genome_from_file(r"..\experiments\base_tree.json");
    let base_tree_genome = crustswarm::swarm::evo::genome::OIDESwarmGenome::from(&base_tree);

    let new_bound_genome = OIDESwarmGenome::new(
        *base_tree_genome.species_count,
        *base_tree_genome.artifact_count,
        *base_tree_genome.rule_count,
    );

    let rebound_tree_genome = new_bound_genome.apply_bounds(&base_tree_genome);

    crustswarm::io::oide_genome_to_file(&rebound_tree_genome, "test_0.oide.json")
        .map(|err| println!("Error occured while converting: {:?}", err));

    let randomized_tree_genome = rebound_tree_genome
        .random(&mut <rand::rngs::StdRng as rand::SeedableRng>::seed_from_u64(123123621));

    crustswarm::io::oide_genome_to_file(&randomized_tree_genome, "test_1.oide.json")
        .map(|err| println!("Error occured while converting: {:?}", err));

    let scaled_tree_genome = randomized_tree_genome.scale(0.5);

    crustswarm::io::oide_genome_to_file(&scaled_tree_genome, "test_2.oide.json")
        .map(|err| println!("Error occured while converting: {:?}", err));

    let added_tree_genome = rebound_tree_genome.add(&scaled_tree_genome);

    crustswarm::io::oide_genome_to_file(&added_tree_genome, "test_3.oide.json")
        .map(|err| println!("Error occured while converting: {:?}", err));

    Ok(())
}

#[test]
#[cfg(test)]
fn oide_genome3() -> Result<(), std::io::Error> {
    use r_oide::prelude::*;
    println!("Cur. Dir: {:?}", std::env::current_dir());
    let base_tree = crustswarm::io::genome_from_file(r"..\experiments\base_tree.json");
    let base_tree_genome = crustswarm::swarm::evo::genome::OIDESwarmGenome::from(&base_tree);

    let new_bound_genome = OIDESwarmGenome::new(
        *base_tree_genome.species_count,
        *base_tree_genome.artifact_count,
        *base_tree_genome.rule_count,
    );

    let rebound_tree_genome = new_bound_genome.apply_bounds(&base_tree_genome);

    crustswarm::io::oide_genome_to_file(&rebound_tree_genome, "test_avg_base.oide.json")
        .map(|err| println!("Error occured while converting: {:?}", err));

    let randomized_tree_genome = rebound_tree_genome
        .random(&mut <rand::rngs::StdRng as rand::SeedableRng>::seed_from_u64(123123621));

    crustswarm::io::oide_genome_to_file(&randomized_tree_genome, "test_avg_1.oide.json")
        .map(|err| println!("Error occured while converting: {:?}", err));

    let midpoints = vec![rebound_tree_genome.clone(), randomized_tree_genome].get_midpoints();

    crustswarm::io::oide_genome_to_file(&midpoints, "test_avg_midpoints.oide.json")
        .map(|err| println!("Error occured while converting: {:?}", err));

    let opposite = rebound_tree_genome.opposite(Some(&midpoints));

    crustswarm::io::oide_genome_to_file(&opposite, "test_avg_opposite.oide.json")
        .map(|err| println!("Error occured while converting: {:?}", err));

    Ok(())
}
