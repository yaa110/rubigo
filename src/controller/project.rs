use inner::logger::Logger;
use std::path::Path;
use std::fs::{File, create_dir, create_dir_all, remove_dir_all, remove_file};
use std::env::current_dir;
use std::fmt::Display;
use git2::Repository;
use json::JsonValue;
use std::io::Write;
use inner::{vendor, json_helper, helpers};
use futures::Future;
use futures_cpupool::CpuPool;

pub fn new(name: &str, is_lib: bool, logger: &Logger) {
    fn delete_new_project<T: Display>(err: T, path: &Path, current_dir: &Path, logger: &Logger) {
        match remove_dir_all(path) {
            Ok(_) => logger.verbose("Delete project", current_dir.to_str().unwrap_or("unknown")),
            Err(e) => logger.error(e),
        }
        logger.fatal(err)
    }

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
    fn delete_init_project<T: Display>(err: T, path: &Path, logger: &Logger) {
        match remove_file(path) {
            Ok(_) => logger.verbose("Delete file", "rubigo.json"),
            _ => (),
        }
        logger.fatal(err)
    }

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
        let mut git_packages = vendor::find_packages(logger);
        match json_helper::write(json_path, "", Some(object!{
            "info" => object!{
                "name" => parent_name.as_str()
            },
            "packages" => object!{
                "git" => git_packages.clone(),
                "local" => array![],
                "global" => array![]
            }
        })) {
            Ok(_) => logger.verbose("Create file", "rubigo.json"),
            Err(e) => delete_init_project(e, json_path, &logger),
        }

        remove_update_key(&mut git_packages);
        match json_helper::write(Path::new("rubigo.lock"), "", Some(object!{
            "git" => git_packages,
            "local" => array![],
            "global" => array![]
        })) {
            Ok(_) => logger.verbose("Create file", "rubigo.lock"),
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

pub fn reset(logger: Logger, no_prompt: bool) {
    if no_prompt {
        inner_reset(logger);
    } else {
        match helpers::confirmation_prompt("This sub command might cause unexpected changes in `rubigo.json` and `rubigo.lock` files.\nDo you want to continue? [Y/n] ") {
            Ok(accepted) => if accepted {
                inner_reset(logger);
            } else {
                logger.error("aborted");
            },
            Err(e) => logger.fatal(e),
        }
    }

    fn inner_reset(logger: Logger) {
        if !Path::new("vendor").is_dir() {
            logger.fatal("vendor directory not found.");
        }

        let pool = CpuPool::new(2);
        let rubigo_json_future = pool.spawn_fn(|| {
            match json_helper::read(Path::new("rubigo.json")) {
                Ok(content_json) => Ok(content_json),
                Err(e) => Err(e),
            }
        });
        let rubigo_lock_future = pool.spawn_fn(|| {
            match json_helper::read(Path::new("rubigo.lock")) {
                Ok(content_json) => Ok(content_json),
                Err(e) => Err(e),
            }
        });

        logger.verbose("Synchronize", "vendor directory");
        let mut git_packages = vendor::find_packages(logger);

        let rubigo_json = rubigo_json_future.wait().unwrap_or(object!{});
        let rubigo_lock = rubigo_lock_future.wait().unwrap_or(object!{});
        let mut global_packages = rubigo_lock["global"].clone();
        if global_packages.is_null() {
            global_packages = array![];
        }
        let local_packages = rubigo_lock["local"].clone();
        let mut local_packages_result = array![];
        if !local_packages.is_null() {
            for i in 0..local_packages.len() {
                let local_pkg = local_packages[i].clone();
                if Path::new("vendor").join(match local_pkg.as_str() {
                    Some(val_str) => val_str,
                    None => continue,
                }).is_dir() {
                    match local_packages_result.push(local_pkg) {
                        _ => (),
                    };
                }
            }
        }

        let mut info_obj = rubigo_json["info"].clone();
        if info_obj.is_null() {
            info_obj = object!{};
        }
        match json_helper::write(Path::new("rubigo.json"), "", Some(object!{
            "info" => info_obj,
            "packages" => object!{
                "git" => git_packages.clone(),
                "local" => local_packages_result.clone(),
                "global" => global_packages.clone()
            }
        })) {
            Ok(_) => logger.verbose("Replace file", "rubigo.json"),
            Err(e) => logger.fatal(e),
        }

        remove_update_key(&mut git_packages);
        match json_helper::write(Path::new("rubigo.lock"), "", Some(object!{
            "git" => git_packages,
            "local" => local_packages_result,
            "global" => global_packages
        })) {
            Ok(_) => logger.verbose("Replace file", "rubigo.lock"),
            Err(e) => logger.fatal(e),
        }

        logger.verbose("Done", "Rubigo project has been reset")
    }
}

fn remove_update_key(packages: &mut JsonValue) {
    for i in 0..packages.len() {
        packages[i].remove("update");
    }
}
