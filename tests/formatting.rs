use djfmt::{formatting::Formatable, template_parser::Template};
use similar::{Algorithm, TextDiff};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[test]
fn test_formatting_comparison() {
    let input_dir = "./tests/formatter/django/input";
    let expected_dir = "./tests/formatter/django/expected";

    for entry in WalkDir::new(input_dir).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let relative_path = entry.path().strip_prefix(input_dir).unwrap();
            let input_path = entry.path();
            let expected_path = Path::new(expected_dir).join(relative_path);

            if expected_path.exists() {
                let input_content =
                    fs::read_to_string(input_path).expect("Failed to read input file");

                let expected_content =
                    fs::read_to_string(expected_path).expect("Failed to read expected file");

                let parsed = Template::parse(&mut input_content.as_str()).unwrap();
                let formatted = parsed.formatted(0);

                if formatted != expected_content {
                    println!("Differences found in file: {:?}", relative_path);
                    let diff = TextDiff::configure()
                        .algorithm(Algorithm::Myers)
                        .diff_lines(&formatted, &expected_content);

                    for change in diff.iter_all_changes() {
                        match change.tag() {
                            similar::ChangeTag::Delete => print!("-{}", change),
                            similar::ChangeTag::Insert => print!("+{}", change),
                            similar::ChangeTag::Equal => print!(" {}", change),
                        }
                    }
                    panic!("Differences found in file: {:?}", relative_path);
                }
            } else {
                panic!("Expected file does not exist: {:?}", expected_path);
            }
        }
    }
}
