use inner::logger::Logger;
use std::path::Path;
use std::fs::{File, create_dir, create_dir_all, remove_dir_all, remove_file};
use std::env::current_dir;
use std::fmt::Display;
use git2::Repository;
use std::ffi::OsStr;
use std::io::Write;
use inner::{vendor, json_helper, helpers};

pub fn new(name: &str, is_lib: bool, logger: &Logger) {
    let path = Path::new(name);
    let current_dir = match current_dir() {
        Ok(path_buf) => path_buf,
        Err(e) => {
            logger.fatal(e);
            return
        },
    };

    if path.exists() {
        logger.fatal(format!("the directory `{}` already exists in {:?}", name, current_dir))
    }

    match create_dir_all(path.join("vendor")) {
        Ok(_) => {
            logger.verbose("Create project", name)
        },
        Err(e) => logger.fatal(e),
    }

    let content;
    let go_file;
    if is_lib {
        content = format!("package {}\n\n", name);
        go_file = format!("{}.go", name);
    } else {
        content = String::from("package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Println(\"Hello, World!\")\n}\n\n");
        go_file = String::from("main.go");
    };

    match File::create(path.join(go_file.as_str())) {
        Ok(mut file) => {
            match file.write_all(content.as_bytes()) {
                Ok(_) => logger.verbose("Create file", go_file),
                Err(e) => delete_new_project(e, path, current_dir.as_path(), logger),
            };
        },
        Err(e) => delete_new_project(e, path, current_dir.as_path(), logger),
    }

    match json_helper::write(path.join("rubigo.json"), name, None) {
        Ok(_) => logger.verbose("Create file", "rubigo.json"),
        Err(e) => delete_new_project(e, path, current_dir.as_path(), logger),
    }

    match Repository::init(path) {
        Ok(repo) => logger.verbose("Initialize git", match repo.workdir() {
            Some(repo_path) => match repo_path.to_str() {
                Some(repo_path_str) => repo_path_str,
                None => "unknown",
            },
            None => "unknown",
        }),
        Err(e) => delete_new_project(e, path, current_dir.as_path(), logger),
    }

    logger.verbose("Done", "Rubigo project has been created")
}

pub fn init(logger: Logger) {
    let json_path = Path::new("rubigo.json");
    if json_path.exists() {
        logger.fatal("Rubigo project has already been initialized")
    }

    let lock_path = Path::new("rubigo.lock");
    if lock_path.exists() {
        match remove_file(lock_path) {
            Ok(_) => logger.verbose("Delete file", "rubigo.lock"),
            Err(e) => delete_init_project(e, json_path, &logger),
        }
    }
    let parent_name = helpers::get_current_dir();
    let vendor_path = Path::new("vendor");
    if !vendor_path.exists() {
        match json_helper::write(json_path, parent_name.as_str(), None) {
            Ok(_) => logger.verbose("Create file", "rubigo.json"),
            Err(e) => delete_init_project(e, json_path, &logger),
        }

        match create_dir(vendor_path) {
            Ok(_) => logger.verbose("Create directory", "vendor"),
            Err(e) => delete_init_project(e, json_path, &logger),
        }
    } else {
        logger.verbose("Synchronize", "vendor directory");
        let packages = vendor::find_packages(logger);
        match json_helper::write(json_path, "", Some(object!{
            "info" => object!{
                "name" => parent_name.as_str()
            },
            "packages" => object!{
                "git" => packages.clone(),
                "local" => array![],
                "global" => array![]
            }
        })) {
            Ok(_) => {
                logger.verbose("Create file", "rubigo.json");
            },
            Err(e) => delete_init_project(e, json_path, &logger),
        }

        match json_helper::write(Path::new("rubigo.lock"), "", Some(object!{
            "packages" => object!{
                "git" => packages,
                "local" => array![],
                "global" => array![]
            }
        })) {
            Ok(_) => {
                logger.verbose("Create file", "rubigo.lock");
            },
            Err(e) => {
                match remove_file("rubigo.lock") {
                    Ok(_) => logger.verbose("Delete file", "rubigo.lock"),
                    _ => (),
                }
                delete_init_project(e, json_path, &logger)
            },
        }
    }

    logger.verbose("Done", "Rubigo project has been initialized")
}

fn delete_init_project<T: Display>(err: T, path: &Path, logger: &Logger) {
    match remove_file(path) {
        Ok(_) => logger.verbose("Delete file", "rubigo.json"),
        _ => (),
    }
    logger.fatal(err)
}

fn delete_new_project<T: Display>(err: T, path: &Path, current_dir: &Path, logger: &Logger) {
    match remove_dir_all(path) {
        Ok(_) => logger.verbose("Delete project", current_dir.to_str().unwrap_or("unknown")),
        Err(e) => logger.error(e),
    }
    logger.fatal(err)
}
