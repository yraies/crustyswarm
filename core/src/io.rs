use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use swarm::genome::SwarmGenome;
use swarm::world::World;

pub fn genome_from_file(path: impl AsRef<Path>) -> SwarmGenome {
    let mut file = File::open(&path)
        .map_err(|e| {
            format!(
                "Error while opening json from {}! \nError: {}",
                path.as_ref().display(),
                e
            )
        })
        .unwrap();
    let mut json_str = String::new();
    file.read_to_string(&mut json_str).unwrap();
    serde_json::from_str(&json_str).unwrap()
}

pub fn genome_to_file(template: &SwarmGenome, path: impl AsRef<Path>) -> Option<Error> {
    fs::write(&path, serde_json::to_string_pretty(&template).unwrap()).err()
}

#[allow(dead_code)]
pub fn print_swarm(world: &impl World, writer: &mut BufWriter<File>) {
    let ags: Vec<_> = world.get_all_agents().collect();

    writeln!(writer, "ply").unwrap();
    writeln!(writer, "format ascii 1.0").unwrap();
    writeln!(writer, "element vertex {}", ags.len()).unwrap();
    writeln!(writer, "property float x").unwrap();
    writeln!(writer, "property float y").unwrap();
    writeln!(writer, "property float z").unwrap();
    writeln!(writer, "end_header").unwrap();

    for ag in ags {
        let (x, y, z) = ag.position.into();
        writeln!(writer, "{} {} {}", x, y, z).unwrap();
    }
    /*    for ag in ags {
        let (x,y,z) = ag.velocity.into();
        write!(writer,"{} {} {}\n",x ,y ,z).unwrap();
    }*/
}
