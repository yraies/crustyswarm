use compression::prelude::{DecodeExt, Deflater, EncodeExt, Inflater};

use crate::swarm::evo::genome::OIDESwarmGenome;
use crate::swarm::genome::dummies::DummySwarmGenome;
use crate::swarm::genome::SwarmGenome;
use crate::swarm::grammar::SwarmGrammar;
use crate::swarm::world::World;

use std::fs::File;
use std::io::BufWriter;
use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::{convert::TryFrom, fs};

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
    let foo: DummySwarmGenome = serde_json::from_str(&json_str).unwrap();
    SwarmGenome::try_from(foo).unwrap()
}

pub fn raw_genome_from_file(path: impl AsRef<Path>) -> SwarmGenome {
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

pub fn raw_genome_to_file(template: &SwarmGenome, path: impl AsRef<Path>) -> Option<Error> {
    fs::write(&path, serde_json::to_string_pretty(&template).unwrap()).err()
}

pub fn oide_genome_from_file(path: impl AsRef<Path>) -> OIDESwarmGenome {
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

pub fn oide_genome_to_file(template: &OIDESwarmGenome, path: impl AsRef<Path>) -> Option<Error> {
    fs::write(&path, serde_json::to_string_pretty(&template).unwrap()).err()
}

pub fn grammar_from_file(path: impl AsRef<Path>) -> SwarmGrammar {
    let mut file = File::open(&path)
        .map_err(|e| {
            format!(
                "Error while opening json from {}! \nError: {}",
                path.as_ref().display(),
                e
            )
        })
        .unwrap();
    let mut byte_vec = vec![];
    file.read_to_end(&mut byte_vec).unwrap();
    let decompressed = byte_vec
        .into_iter()
        .decode(&mut Deflater::new())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let decompressed = String::from_utf8(decompressed).unwrap();
    //fs::write("decompressed.grammar.json", &decompressed).unwrap_or(());
    serde_json::from_str(&decompressed).unwrap()
}

pub fn grammar_to_file(template: &SwarmGrammar, path: impl AsRef<Path>) -> Option<Error> {
    let json = serde_json::to_string_pretty(&template).unwrap();
    fs::write(&path.as_ref().with_extension("plain.json"), &json).err();
    let compressed = json
        .into_bytes()
        .encode(&mut Inflater::new(), compression::prelude::Action::Finish)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    fs::write(&path, compressed).err()
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
