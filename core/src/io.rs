use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use swarm::grammar::SwarmGrammar;
use swarm::grammar::SwarmTemplate;

pub fn grammar_from_file(path: impl AsRef<Path>) -> SwarmGrammar {
    let mut file = File::open(&path)
        .map_err(|e| {
            format!(
                "Error while opening path {}! Error: {}",
                path.as_ref().display(),
                e
            )
        })
        .unwrap();
    let mut json_str = String::new();
    file.read_to_string(&mut json_str).unwrap();
    serde_json::from_str(&json_str).unwrap()
}

pub fn template_from_file(path: impl AsRef<Path>) -> SwarmTemplate {
    let mut file = File::open(&path)
        .map_err(|e| {
            format!(
                "Error while opening path {}! Error: {}",
                path.as_ref().display(),
                e
            )
        })
        .unwrap();
    let mut json_str = String::new();
    file.read_to_string(&mut json_str).unwrap();
    serde_json::from_str(&json_str).unwrap()
}

pub fn grammar_to_file(grammar: &SwarmGrammar, path: impl AsRef<Path>) -> Option<Error> {
    fs::write(&path, serde_json::to_string_pretty(&grammar).unwrap()).err()
}

pub fn template_to_file(template: &SwarmTemplate, path: impl AsRef<Path>) -> Option<Error> {
    fs::write(&path, serde_json::to_string_pretty(&template).unwrap()).err()
}

#[allow(dead_code)]
pub fn print_swarm(grammar: &SwarmGrammar, writer: &mut BufWriter<File>) {
    let ags = grammar.get_agents();

    write!(writer, "ply\n").unwrap();
    write!(writer, "format ascii 1.0\n").unwrap();
    write!(writer, "element vertex {}\n", ags.len()).unwrap();
    write!(
        writer,
        "property float x\nproperty float y\nproperty float z\n"
    )
    .unwrap();
    write!(writer, "end_header\n").unwrap();

    for ag in ags {
        let (x, y, z) = ag.position.into();
        write!(writer, "{} {} {}\n", x, y, z).unwrap();
    }
    /*    for ag in ags {
        let (x,y,z) = ag.velocity.into();
        write!(writer,"{} {} {}\n",x ,y ,z).unwrap();
    }*/
}
