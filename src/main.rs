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
    /// List of files or directories to format
    #[arg()]
    path: PathBuf,
    /// Avoid writing any formatted files back; instead, exit with a non-zero status code if any files would have been modified, and zero otherwise
    #[arg(long)]
    check: bool,
}

fn format_file_both(path: &PathBuf, check: bool) -> bool {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let original_contents = contents.clone();

    let parsed_template = Template::parse(&mut contents.as_str()).unwrap();
    let formatted_template = parsed_template.formatted(0);

    let parsed_html = Node::parse(&mut formatted_template.as_str()).unwrap();
    let formatted_html = parsed_html.formatted(0);

    let formattable = original_contents != formatted_html;
    if check {
        return formattable;
    }

    file.set_len(0).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();

    file.write_all(formatted_html.as_bytes()).unwrap();
    file.flush().unwrap();

    formattable
}

fn main() {
    let args = Args::parse();

    if !args.path.exists() {
        println!("File does not exist");
        std::process::exit(1);
    }

    if args.path.is_file() {
        let modified = format_file_both(&args.path, args.check);
        if modified {
            println!("{}: formatted", args.path.to_string_lossy());
            std::process::exit(1);
        }
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

    let results: Vec<(&PathBuf, bool)> = files
        .par_iter()
        .map(|path| (path, format_file_both(path, args.check)))
        .collect();

    let modified_paths = results
        .iter()
        .filter(|result| result.1)
        .map(|result| {
            println!("{}: formatted", result.0.to_string_lossy());
            result.0
        })
        .collect::<Vec<_>>();

    let any_modified = !modified_paths.is_empty();
    if any_modified {
        std::process::exit(1);
    }
}
