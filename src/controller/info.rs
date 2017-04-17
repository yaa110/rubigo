use inner::logger::Logger;
use inner::json_helper;
use std::path::Path;

pub fn display(logger: &Logger) {
    let content = match json_helper::read(Path::new("rubigo.json")) {
        Ok(content) => content,
        Err(e) => {
            logger.fatal(format!("unable to read `rubigo.json`: {}", e));
            return
        }
    };

    let info = &content[json_helper::INFO_KEY];
    if info.is_null() {
        return
    }

    println!("{}:", "Project info");

    match info[json_helper::NAME_KEY].as_str() {
        Some(text) => println!("\t{}: {}", "Project name", text),
        None => (),
    }

    match info[json_helper::IMPORT_KEY].as_str() {
        Some(text) => println!("\t{}: {}", "Import", text),
        None => (),
    }

    match info[json_helper::DESCRIPTION_KEY].as_str() {
        Some(text) => println!("\t{}: {}", "Description", text),
        None => (),
    }

    match info[json_helper::HOMEPAGE_KEY].as_str() {
        Some(text) => println!("\t{}: {}", "Homepage", text),
        None => (),
    }

    match info[json_helper::LICENSE_KEY].as_str() {
        Some(text) => println!("\t{}: {}", "License", text),
        None => (),
    }

    let authors = &info[json_helper::AUTHORS_KEY];
    if authors.len() == 0 {
        return
    }

    println!("\n{} ({}):", "Authors", authors.len());

    for i in 0..authors.len() {
        let author = &authors[i];
        print!("[{}]", i + 1);

        match author[json_helper::NAME_KEY].as_str() {
            Some(text) => println!("\t{}: {}", "Name", text),
            None => (),
        }

        match author[json_helper::EMAIL_KEY].as_str() {
            Some(text) => println!("\t{}: {}", "Email", text),
            None => (),
        }

        match author[json_helper::WEBSITE_KEY].as_str() {
            Some(text) => println!("\t{}: {}\n", "Website", text),
            None => println!(),
        }
    }
}
