use json::JsonValue;
use inner::json_helper;

pub fn print_header(header: &str, length: usize) {
    println!("{} ({}):", header, length);
}

pub fn print_git_packages(pkgs: &JsonValue) {
    for i in 0..pkgs.len() {
        let pkg = &pkgs[i];

        print!("[{}]", i + 1);

        match pkg[json_helper::IMPORT_KEY].as_str() {
            Some(text) => println!("\t{}: {}", "Import", text),
            None => (),
        }

        match pkg[json_helper::VERSION_KEY].as_str() {
            Some(text) => println!("\t{}: {}", "Version", text),
            None => (),
        }

        match pkg[json_helper::REPO_KEY].as_str() {
            Some(text) => println!("\t{}: {}\n", "Repository", text),
            None => println!(),
        }
    }
}

pub fn print_str_packages(pkgs: &JsonValue) {
    for i in 0..pkgs.len() {
        print!("[{}]", i + 1);
        match pkgs[i].as_str() {
            Some(text) => println!("\t{}: {}\n", "Import", text),
            None => (),
        }
    }
}
