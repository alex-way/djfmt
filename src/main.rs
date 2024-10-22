use clap::Parser as ClapParser;
use djfmt::{formatting::Formatable, template_parser::Template};
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
    path: String,
}

fn main() {
    let args = Args::parse();

    let path: PathBuf = args.path.into();

    if !path.exists() {
        println!("File does not exist");
        return;
    }

    if !path.is_file() {
        println!("Path is not a file");
        return;
    }

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    // let parsed = Element::<RandomState>::parse.parse(&contents).unwrap();
    let parsed = Template::parse(&mut contents.as_str()).unwrap();
    let formatted = parsed.formatted(0);

    file.set_len(0).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();

    file.write_all(formatted.as_bytes()).unwrap();
    file.flush().unwrap();
    println!("File formatted!");
}
