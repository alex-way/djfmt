use clap::Parser as ClapParser;
use djfmt::{formatting::Formatable, html_parser::node::Node, template_parser::Template};
use glob::glob;
use rayon::prelude::*;
use std::{
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

/// Simple program to greet a person
#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    path: PathBuf,
}

fn format_file_both(path: &PathBuf) {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let parsed_template = Template::parse(&mut contents.as_str()).unwrap();
    let formatted_template = parsed_template.formatted(0);

    let parsed_html = Node::parse(&mut formatted_template.as_str()).unwrap();
    let formatted_html = parsed_html.formatted(0);

    file.set_len(0).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();

    file.write_all(formatted_html.as_bytes()).unwrap();
    file.flush().unwrap();
}

fn main() {
    let args = Args::parse();

    if !args.path.exists() {
        println!("File does not exist");
        return;
    }

    if args.path.is_file() {
        format_file_both(&args.path);
        return;
    }

    let glob_path = format!("{}/**/*.html", args.path.to_string_lossy());
    let files = glob(&glob_path)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            if entry.is_file() {
                Some(entry)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    files.par_iter().for_each(|path| {
        format_file_both(path);
    });
}
