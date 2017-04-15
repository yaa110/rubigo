use ansi_term::Color;
use json::JsonValue;
use inner::json_helper;

pub fn print_header(header: &str, length: usize) {
    println!("{} ({}):", Color::Yellow.paint(header), length);
}

pub fn print_git_packages(pkgs: &JsonValue) {
    for i in 0..pkgs.len() {
        let pkg = &pkgs[i];

        print!("[{}]", i + 1);

        match pkg[json_helper::IMPORT_KEY].as_str() {
            Some(text) => println!("\t{}: {}", Color::Fixed(12).paint("Import"), text),
            None => (),
        }

        match pkg[json_helper::VERSION_KEY].as_str() {
            Some(text) => println!("\t{}: {}", Color::Fixed(12).paint("Version"), text),
            None => (),
        }

        match pkg[json_helper::REPO_KEY].as_str() {
            Some(text) => println!("\t{}: {}\n", Color::Fixed(12).paint("Repository"), text),
            None => println!(),
        }
    }
}

pub fn print_str_packages(pkgs: &JsonValue) {
    for i in 0..pkgs.len() {
        print!("[{}]", i + 1);
        match pkgs[i].as_str() {
            Some(text) => println!("\t{}: {}\n", Color::Fixed(12).paint("Import"), text),
            None => (),
        }
    }
}
