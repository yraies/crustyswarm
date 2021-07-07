extern crate crustswarm_lib as crustswarm;

use crustswarm::swarm::evo::genome::OIDESwarmGenome;
//use linfa::{
//    dataset::{CountedTargets, FromTargetArray, Labels},
//    prelude::*,
//};
//use ndarray::Array2;
use r_oide::{prelude::*, traits::VecCollector};
use std::env;

fn main() {
    let cmd = env::args().skip(1).next();

    match cmd.as_ref().map(String::as_str) {
        Some("help") => {
            println!("[grammar|raw|genome]2oide <some.json> <target.oide.json>\n");
            println!("oide2raw <some.oide.json> <target.genome.json>\n");
            println!("oide2raw <some.oide.json> <target.genome.json>\n");
            println!("rebound_oide <some.oide.json> <target.genome.json>           \nApplies our fixed but arbitrary bounds to some.oide.json\n");
            println!("generate_zero <species_count> <artifact_count> <rule_count>  \nCreates an oide config template with the given sizes\n");
            println!("parametercount <species_count> <artifact_count> <rule_count> \nCounts the number of variable parameters for the given sizes\n");
            println!("convert2csv <source_dir> <target.csv>                        \nCollects a directory of .oide.jsons into a .csv of floats\n");
            println!("hash <some.json>                                             \nReturns the hash of a given configuration (WIP/Faulty...)\n");
        }
        Some("oide2raw") => {
            let oide_path = env::args()
                .skip(2)
                .next()
                .expect("Oide config to convert required!");
            let target_path = env::args()
                .skip(3)
                .next()
                .unwrap_or("converted.genome.json".to_string());
            println!("converting {} to raw genome {}", oide_path, target_path);
            crustswarm::io::raw_genome_to_file(
                &crustswarm::swarm::genome::SwarmGenome::from(
                    &crustswarm::io::oide_genome_from_file(oide_path),
                ),
                target_path,
            )
            .map(|err| println!("Error occured while converting: {:?}", err));
        }
        Some("genome2oide") => {
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
        Some("raw2oide") => {
            let path = env::args()
                .skip(2)
                .next()
                .expect("Oide config to convert required!");
            let target_path = env::args()
                .skip(3)
                .next()
                .unwrap_or("converted.oide.json".to_string());
            println!("converting {} to oide template {}", path, target_path);
            let genome = crustswarm::io::raw_genome_from_file(path);
            let oide_genome = crustswarm::swarm::evo::genome::OIDESwarmGenome::from(&genome);
            crustswarm::io::oide_genome_to_file(&oide_genome, target_path)
                .map(|err| println!("Error occured while converting: {:?}", err));
        }
        Some("grammar2oide") => {
            let path = env::args()
                .skip(2)
                .next()
                .expect("Oide config to convert required!");
            let target_path = env::args()
                .skip(3)
                .next()
                .unwrap_or("converted.oide.json".to_string());
            println!("converting {} to oide template {}", path, target_path);
            let grammar = crustswarm::io::grammar_from_file(path);
            let oide_genome =
                crustswarm::swarm::evo::genome::OIDESwarmGenome::from(&grammar.genome);
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
            let agents = env::args()
                .skip(3)
                .next()
                .unwrap()
                .parse::<usize>()
                .unwrap();
            let artifacts = env::args()
                .skip(4)
                .next()
                .unwrap()
                .parse::<usize>()
                .unwrap();
            let rules = env::args()
                .skip(5)
                .next()
                .unwrap()
                .parse::<usize>()
                .unwrap();
            let target_path = env::args()
                .skip(2)
                .next()
                .unwrap_or(format!("zero_{}_{}_{}.oide.json", agents, artifacts, rules));
            std::fs::write(
                target_path,
                serde_json::to_string_pretty(&OIDESwarmGenome::new(agents, artifacts, rules))
                    .unwrap(),
            )
            .unwrap();
        }
        Some("parametercount") => {
            let agents = env::args()
                .skip(2)
                .next()
                .unwrap()
                .parse::<usize>()
                .unwrap();
            let artifacts = env::args()
                .skip(3)
                .next()
                .unwrap()
                .parse::<usize>()
                .unwrap();
            let rules = env::args()
                .skip(4)
                .next()
                .unwrap()
                .parse::<usize>()
                .unwrap();
            let genome = OIDESwarmGenome::new(agents, artifacts, rules);
            println!(
                "d={:2} g={:2} r={:2} => {:5} parameters",
                agents,
                artifacts,
                rules,
                genome.parameter_count()
            );
        }
        Some("pca_analysis") => {
            let mut genomes = Vec::with_capacity(env::args().skip(2).count());
            let mut genome_generations = Vec::with_capacity(env::args().skip(2).count());
            let dir_path = env::args().skip(2).next().unwrap();
            let dir = std::fs::read_dir(dir_path).unwrap();
            let paths: Vec<_> = dir
                .map(|d| d.unwrap().path())
                .filter(|d| {
                    d.is_file()
                        && d.file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .ends_with(".oide.json")
                })
                .collect();
            paths.iter().for_each(|path| {
                eprintln!("Reading {:?}", path.file_name());
                genomes.push(crustswarm::io::oide_genome_from_file(&path));
                genome_generations.push(
                    path.file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .split("gen")
                        .skip(1)
                        .next()
                        .unwrap()
                        .split("_")
                        .next()
                        .unwrap()
                        .to_string(),
                );
            });

            let features = dbg!(FeatureCollector::collect(&genomes[0]));

            let mut genome_vectors = Vec::with_capacity(genomes.len());
            genomes
                .iter()
                .for_each(|g| genome_vectors.push(VecCollector::collect(g)));

            let (_nsamples, _nfeatures) = (genome_vectors.len(), genome_vectors[0].len());
            //println!(
            //    "{} genomes/{} features á {} genes",
            //    nsamples,
            //    features.len(),
            //    nfeatures
            //);
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            wtr.write_field("generation").unwrap();
            wtr.write_record(&features).unwrap();
            for i in 0..genome_vectors.len() {
                let line: Vec<_> = genome_vectors[i].iter().map(|v| v.to_string()).collect();
                wtr.write_field(genome_generations[i].to_string()).unwrap();
                wtr.write_record(&line).unwrap();
            }
            wtr.flush().unwrap();
        }
        Some("op_analysis_1") => {
            let oide_path = env::args()
                .skip(2)
                .next()
                .expect("Oide config to convert required!");
            let base_genome = crustswarm::io::oide_genome_from_file(&oide_path);
            let filebase = oide_path.strip_suffix(".oide.json").unwrap();

            for i in 0..=10 {
                let target_path =
                    format!("{}_{}_{:0.1}.oide.json", filebase, "scale", i as f32 / 10.0);
                let genome = base_genome.scale(i as f32 / 10.0);
                crustswarm::io::oide_genome_to_file(&genome, target_path)
                    .map(|err| println!("Error occured while converting: {:?}", err));

                let target_path =
                    format!("{}_{}_{:0.1}.oide.json", filebase, "add", i as f32 / 10.0);
                let genome = base_genome.add(&base_genome.scale(i as f32 / 10.0));
                crustswarm::io::oide_genome_to_file(&genome, target_path)
                    .map(|err| println!("Error occured while converting: {:?}", err));

                let target_path =
                    format!("{}_{}_{:0.1}.oide.json", filebase, "diff", i as f32 / 10.0);
                let genome = base_genome.difference(&base_genome.scale(i as f32 / 10.0));
                crustswarm::io::oide_genome_to_file(&genome, target_path)
                    .map(|err| println!("Error occured while converting: {:?}", err));
            }
        }
        Some("op_analysis") => {
            let oide_path = env::args()
                .skip(2)
                .next()
                .expect("Oide config to convert required!");
            let oide_path2 = env::args()
                .skip(3)
                .next()
                .expect("Oide config to convert required!");
            let mut target_genome = crustswarm::io::oide_genome_from_file(&oide_path);
            let base_genome = &OIDESwarmGenome::new(
                *target_genome.species_count,
                *target_genome.artifact_count,
                *target_genome.rule_count,
            )
            .zero();
            target_genome = base_genome.apply_bounds(&target_genome);
            let mut mod_genome = crustswarm::io::oide_genome_from_file(&oide_path2);
            mod_genome = base_genome.apply_bounds(&mod_genome);
            let target_file = oide_path.strip_suffix(".oide.json").unwrap();
            let mod_file = oide_path2.strip_suffix(".oide.json").unwrap();

            for i in 0..=10 {
                let target_path = format!(
                    "{}_{}_{}_{:0.1}.oide.json",
                    target_file,
                    mod_file,
                    "lerp",
                    i as f32 / 10.0
                );

                let scaled_target = target_genome.scale(1.0 - i as f32 / 10.0);
                let scaled_mod = mod_genome.scale(i as f32 / 10.0);
                let genome = target_genome.zero().add(&scaled_target).add(&scaled_mod);
                crustswarm::io::oide_genome_to_file(&genome, target_path)
                    .map(|err| println!("Error occured while converting: {:?}", err));

                let target_path = format!(
                    "{}_{}_{}_{:0.1}.oide.json",
                    target_file,
                    mod_file,
                    "add",
                    i as f32 / 10.0
                );
                let genome = target_genome.add(&mod_genome.scale(i as f32 / 10.0));
                crustswarm::io::oide_genome_to_file(&genome, target_path)
                    .map(|err| println!("Error occured while converting: {:?}", err));
            }

            for i in 0..=10 {
                let target_path = format!(
                    "{}_{}_{:0.1}.oide.json",
                    target_file,
                    "scale",
                    i as f32 / 10.0
                );
                let genome = target_genome.scale(i as f32 / 10.0);
                crustswarm::io::oide_genome_to_file(&genome, target_path)
                    .map(|err| println!("Error occured while converting: {:?}", err));

                let target_path = format!(
                    "{}_{}_{:0.1}.oide.json",
                    target_file,
                    "opposite",
                    i as f32 / 10.0
                );
                let genome = target_genome
                    .zero()
                    .add(&target_genome.scale(1.0 - i as f32 / 10.0))
                    .add(&target_genome.opposite(None).scale(i as f32 / 10.0));
                crustswarm::io::oide_genome_to_file(&genome, target_path)
                    .map(|err| println!("Error occured while converting: {:?}", err));

                let target_path = format!(
                    "{}_{}_{:0.1}.oide.json",
                    mod_file,
                    "opposite",
                    i as f32 / 10.0
                );
                let genome = mod_genome
                    .zero()
                    .add(&mod_genome.scale(1.0 - i as f32 / 10.0))
                    .add(&mod_genome.opposite(None).scale(i as f32 / 10.0));
                crustswarm::io::oide_genome_to_file(&genome, target_path)
                    .map(|err| println!("Error occured while converting: {:?}", err));
            }
        }
        Some("hash") => {
            let oide_path = env::args()
                .skip(2)
                .next()
                .expect("Oide config to convert required!");
            let genome = if oide_path.ends_with(".oide.json") {
                crustswarm::io::oide_genome_from_file(&oide_path)
            } else if oide_path.ends_with(".grammar.json") {
                let translated = crustswarm::swarm::evo::genome::OIDESwarmGenome::from(
                    &crustswarm::io::grammar_from_file(&oide_path).genome,
                );
                translated.apply_bounds(&OIDESwarmGenome::new(
                    *translated.species_count,
                    *translated.artifact_count,
                    *translated.rule_count,
                ))
            } else {
                crustswarm::swarm::evo::genome::OIDESwarmGenome::from(
                    &crustswarm::io::genome_from_file(&oide_path),
                )
            };
            println!("{}", genome.my_hash());
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
