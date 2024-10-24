use djfmt::html_parser::node::Node;
use djfmt::{formatting::Formatable, template_parser::Template};
use rstest::rstest;
use similar::{Algorithm, TextDiff};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn show_diff(input: String, expected: String) {
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .diff_lines(&input, &expected);

    for change in diff.iter_all_changes() {
        match change.tag() {
            similar::ChangeTag::Delete => print!("-{}", change),
            similar::ChangeTag::Insert => print!("+{}", change),
            similar::ChangeTag::Equal => print!(" {}", change),
        }
    }
}

#[rstest]
fn test_django_formatting() {
    let input_dir = "./tests/formatter/django/input";
    let expected_dir = "./tests/formatter/django/expected";

    for entry in WalkDir::new(input_dir).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }

        let relative_path = entry.path().strip_prefix(input_dir).unwrap();
        let input_path = entry.path();
        let expected_path = Path::new(expected_dir).join(relative_path);

        if !expected_path.exists() {
            panic!("Expected file does not exist: {:?}", expected_path);
        }

        let input_content = fs::read_to_string(input_path).expect("Failed to read input file");
        let expected = fs::read_to_string(expected_path).expect("Failed to read expected file");

        let parsed = Template::parse(&mut input_content.as_str()).unwrap();
        let actual = parsed.formatted(0);

        if actual != expected {
            println!("Differences found in file: {:?}", relative_path);
            show_diff(actual, expected);
            panic!("Differences found in file: {:?}", relative_path);
        }
    }
}

#[rstest]
fn test_html_formatting() {
    let input_dir = "./tests/formatter/html/input";
    let expected_dir = "./tests/formatter/html/expected";

    for entry in WalkDir::new(input_dir).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }

        let relative_path = entry.path().strip_prefix(input_dir).unwrap();
        let input_path = entry.path();
        let expected_path = Path::new(expected_dir).join(relative_path);

        if !expected_path.exists() {
            panic!("Expected file does not exist: {:?}", expected_path);
        }

        let input_content = fs::read_to_string(input_path).expect("Failed to read input file");
        let expected = fs::read_to_string(expected_path).expect("Failed to read expected file");

        let parsed = Node::parse(&mut input_content.as_str()).unwrap();
        let actual = parsed.formatted(0);

        if actual != expected {
            println!("Differences found in file: {:?}", relative_path);
            show_diff(actual, expected);
            panic!("Differences found in file: {:?}", relative_path);
        }
    }
}
