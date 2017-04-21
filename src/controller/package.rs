use inner::logger::Logger;
use futures::Future;
use futures_cpupool::CpuPool;
use inner::{json_helper, vendor, go, helpers, git_helper};
use std::path::Path;
use json::JsonValue;
use std::sync::mpsc::channel;
use std::thread;
use std::fs::{create_dir_all, remove_dir_all};
use controller::project;
use git2::{Repository, ResetType};
use std::process;

pub fn get(mut package_url: &str, repo_url: Option<&str>, no_prompt: bool, is_global: bool, is_local: bool, logger: Logger) {
    if package_url.ends_with("/") || package_url.ends_with("\\") {
        package_url = &package_url[..package_url.len() - 1];
    }

    if !Path::new("rubigo.json").exists() {
        if no_prompt {
            project::init(logger);
        } else {
            match helpers::confirmation_prompt("The `rubigo.json` file was not found in this directory, it seems that Rubigo project has not been initialized.\nDo you want to initialize it? [Y/n]") {
                Ok(state) => if state {
                    project::init(logger);
                } else {
                    logger.fatal("Rubigo project has not been initialized");
                    return
                },
                Err(e) => {
                    logger.fatal(e);
                    return
                },
            }
        }
    }

    let rubigo_json = match json_helper::read(Path::new("rubigo.json")) {
        Ok(content_json) => content_json,
        Err(e) => {
            logger.fatal(format!("unable to read `rubigo.json`: {}", e));
            return
        },
    };

    let pool = CpuPool::new(1);
    let rubigo_lock_future = pool.spawn_fn(|| {
        match json_helper::read(Path::new("rubigo.lock")) {
            Ok(content_json) => Ok(content_json),
            Err(e) => Err(e),
        }
    });

    let pkg_import = helpers::strip_url_scheme(package_url);
    let mut pkg_path_buf = helpers::get_path_from_url(&pkg_import);

    let json_packages_object;
    let lock_packages_object;
    let rubigo_lock;

    if is_global {
        let global_ps = &rubigo_json[json_helper::PACKAGES_KEY][json_helper::GLOBAL_KEY];
        if !global_ps.is_null() {
            for i in 0..global_ps.len() {
                if match global_ps[i].as_str() {
                    Some(name) => name,
                    None => continue,
                } == pkg_import.as_str() {
                    logger.fatal(format!("the package `{}` already exists in `rubigo.json` file", pkg_import));
                    return;
                }
            }
        }

        match go::get(pkg_import.as_str(), false) {
            true => logger.verbose("Global package", &pkg_import),
            false => {
                logger.fatal(format!("unable to get package `{}`", pkg_import));
                return
            }
        }

        rubigo_lock = rubigo_lock_future.wait().unwrap_or(object!{});

        let mut global_pkgs = rubigo_json[json_helper::PACKAGES_KEY][json_helper::GLOBAL_KEY].clone();
        if global_pkgs.is_null() {
            global_pkgs = array![pkg_import.clone()];
        } else {
            let _ = global_pkgs.push(pkg_import.clone());
        }

        let mut lock_global_pkgs = rubigo_lock[json_helper::GLOBAL_KEY].clone();
        if lock_global_pkgs.is_null() {
            lock_global_pkgs = array![pkg_import];
        } else {
            let _ = lock_global_pkgs.push(pkg_import);
        }

        json_packages_object = object!{
            json_helper::GIT_KEY => rubigo_json[json_helper::PACKAGES_KEY][json_helper::GIT_KEY].clone(),
            json_helper::LOCAL_KEY => rubigo_json[json_helper::PACKAGES_KEY][json_helper::LOCAL_KEY].clone(),
            json_helper::GLOBAL_KEY => global_pkgs
        };

        lock_packages_object = object!{
            json_helper::GIT_KEY => rubigo_lock[json_helper::GIT_KEY].clone(),
            json_helper::LOCAL_KEY => rubigo_lock[json_helper::LOCAL_KEY].clone(),
            json_helper::GLOBAL_KEY => lock_global_pkgs
        };
    } else if is_local {
        let local_ps = &rubigo_json[json_helper::PACKAGES_KEY][json_helper::LOCAL_KEY];
        if !local_ps.is_null() {
            for i in 0..local_ps.len() {
                if match local_ps[i].as_str() {
                    Some(name) => name,
                    None => continue,
                } == pkg_import.as_str() {
                    logger.fatal(format!("the package `{}` already exists in `rubigo.json` file", pkg_import));
                    return;
                }
            }
        }

        let pkg_path = pkg_path_buf.as_path();
        if pkg_path.exists() {
            logger.fatal(format!("the package `{}` already exists", pkg_import));
            return
        }

        match create_dir_all(pkg_path) {
            Ok(_) => logger.verbose("Local package", &pkg_import),
            Err(e) => {
                logger.fatal(e);
                return
            }
        }

        rubigo_lock = rubigo_lock_future.wait().unwrap_or(object!{});

        let mut local_pkgs = rubigo_json[json_helper::PACKAGES_KEY][json_helper::LOCAL_KEY].clone();
        if local_pkgs.is_null() {
            local_pkgs = array![pkg_import.clone()];
        } else {
            let _ = local_pkgs.push(pkg_import.clone());
        }

        let mut lock_local_pkgs = rubigo_lock[json_helper::LOCAL_KEY].clone();
        if lock_local_pkgs.is_null() {
            lock_local_pkgs = array![pkg_import];
        } else {
            let _ = lock_local_pkgs.push(pkg_import);
        }

        json_packages_object = object!{
            json_helper::GIT_KEY => rubigo_json[json_helper::PACKAGES_KEY][json_helper::GIT_KEY].clone(),
            json_helper::GLOBAL_KEY => rubigo_json[json_helper::PACKAGES_KEY][json_helper::GLOBAL_KEY].clone(),
            json_helper::LOCAL_KEY => local_pkgs
        };

        lock_packages_object = object!{
            json_helper::GIT_KEY => rubigo_lock[json_helper::GIT_KEY].clone(),
            json_helper::GLOBAL_KEY => rubigo_lock[json_helper::GLOBAL_KEY].clone(),
            json_helper::LOCAL_KEY => lock_local_pkgs
        };
    } else {
        let git_ps = &rubigo_json[json_helper::PACKAGES_KEY][json_helper::GIT_KEY];
        if !git_ps.is_null() {
            for i in 0..git_ps.len() {
                if match git_ps[i][json_helper::IMPORT_KEY].as_str() {
                    Some(name) => name,
                    None => continue,
                } == pkg_import.as_str() {
                    logger.fatal(format!("the package `{}` already exists in `rubigo.json` file", pkg_import));
                    return;
                }
            }
        }

        let mut pkg_json = object!{
            json_helper::IMPORT_KEY => pkg_import.clone()
        };

        let (pkg_import_url, modified_pkg_path) = helpers::modify_golang_org(pkg_import.as_str());
        let modified_pkg_import = if modified_pkg_path.is_some() {
            modified_pkg_path.unwrap()
        } else {
            pkg_import.clone()
        };
        pkg_path_buf = helpers::get_path_from_url(&modified_pkg_import);
        let pkg_path = pkg_path_buf.as_path();
        if pkg_path.exists() {
            logger.error(format!("the package `{}` already exists in `vendor` directory", pkg_import));
            match remove_dir_all(pkg_path) {
                Ok(_) => logger.verbose("Delete directory", &pkg_import),
                Err(e) => {
                    logger.fatal(e);
                    return
                },
            }
        }
        match create_dir_all(pkg_path) {
            Ok(_) => logger.verbose("Create directory", &pkg_import),
            Err(e) => {
                logger.fatal(e);
                return
            },
        }

        let repo = match Repository::clone(match repo_url {
            Some(url) => {
                pkg_json[json_helper::REPO_KEY] = url.into();
                url
            },
            None => &pkg_import_url,
        }, pkg_path) {
            Ok(repo) => {
                logger.verbose("Clone repository", &pkg_import);
                repo
            },
            Err(e) => {
                let _ = remove_dir_all(pkg_path);
                logger.fatal(e);
                return
            },
        };

        let mut lock_pkg_json = pkg_json.clone();

        let version;
        if !no_prompt {
            let (ver, rule) = match helpers::version_prompt(&repo) {
                Some(ver) => ver,
                None => {
                    let _ = remove_dir_all(pkg_path);
                    logger.fatal("unable to get latest commit of package");
                    return
                },
            };
            version = ver;
            pkg_json[json_helper::VERSION_KEY] = rule.into();
        } else {
            version = match git_helper::get_latest_commit(&repo) {
                Some(ver) => ver,
                None => {
                    let _ = remove_dir_all(pkg_path);
                    logger.fatal("unable to get latest commit of package");
                    return
                },
            };

            pkg_json[json_helper::VERSION_KEY] = version.clone().into();
        }

        let version_object = match git_helper::get_revision_object(&repo, pkg_import.clone(), version, true, logger) {
            Some(tup) => {
                lock_pkg_json[json_helper::VERSION_KEY] = tup.1.into();
                tup.0
            },
            None => {
                let _ = remove_dir_all(pkg_path);
                logger.fatal("unable to parse the version of package");
                return
            }
        };

        match repo.set_head_detached(version_object.id()) {
            Ok(_) => (),
            Err(e) => {
                let _ = remove_dir_all(pkg_path);
                logger.fatal(e);
                return
            },
        }

        match repo.reset(&version_object, ResetType::Hard, None){
            Ok(_) => (),
            Err(e) => {
                let _ = remove_dir_all(pkg_path);
                logger.fatal(e);
                return
            },
        }

        let mut git_pkgs = rubigo_json[json_helper::PACKAGES_KEY][json_helper::GIT_KEY].clone();
        if git_pkgs.is_null() {
            git_pkgs = array![pkg_json];
        } else {
            let _ = git_pkgs.push(pkg_json);
        }

        rubigo_lock = rubigo_lock_future.wait().unwrap_or(object!{});
        let mut lock_git_pkgs = rubigo_lock[json_helper::GIT_KEY].clone();
        if lock_git_pkgs.is_null() {
            lock_git_pkgs = array![lock_pkg_json];
        } else {
            let _ = lock_git_pkgs.push(lock_pkg_json);
        }

        json_packages_object = object!{
            json_helper::GIT_KEY => git_pkgs,
            json_helper::LOCAL_KEY => rubigo_json[json_helper::PACKAGES_KEY][json_helper::LOCAL_KEY].clone(),
            json_helper::GLOBAL_KEY => rubigo_json[json_helper::PACKAGES_KEY][json_helper::GLOBAL_KEY].clone()
        };

        lock_packages_object = object!{
            json_helper::GIT_KEY => lock_git_pkgs,
            json_helper::LOCAL_KEY => rubigo_lock[json_helper::LOCAL_KEY].clone(),
            json_helper::GLOBAL_KEY => rubigo_lock[json_helper::GLOBAL_KEY].clone()
        };
    }

    match json_helper::write("rubigo.json", "", Some(object!{
        json_helper::INFO_KEY => rubigo_json[json_helper::INFO_KEY].clone(),
        json_helper::PACKAGES_KEY => json_packages_object
    })) {
        Ok(_) => logger.verbose("Update file", "rubigo.json"),
        Err(e) => {
            if !is_global {
                let _ = remove_dir_all(pkg_path_buf.as_path());
            }
            logger.fatal(format!("unable to write to `rubigo.json`: {}", e));
            return
        },
    }

    match json_helper::write("rubigo.lock", "", Some(lock_packages_object)) {
        Ok(_) => logger.verbose("Update file", "rubigo.lock"),
        Err(e) => {
            let _ = json_helper::write("rubigo.json", "", Some(rubigo_json));
            let _ = remove_dir_all(pkg_path_buf.as_path());
            logger.fatal(format!("unable to write to `rubigo.lock`: {}", e))
        },
    }
}

pub fn remove(package_dir: &str, logger: Logger) {
    let json_content = match json_helper::read(Path::new("rubigo.json")) {
        Ok(content) => content,
        Err(e) => {
            logger.fatal(format!("unable to read `rubigo.json`: {}", e));
            return
        }
    };

    let lock_content = match json_helper::read(Path::new("rubigo.lock")) {
        Ok(content) => content,
        Err(e) => {
            logger.fatal(format!("unable to read `rubigo.lock`: {}", e));
            return
        }
    };

    let new_json_git = json_helper::remove_package_from_array(package_dir, &json_content[json_helper::PACKAGES_KEY][json_helper::GIT_KEY], false);
    let new_lock_git = json_helper::remove_package_from_array(package_dir, &lock_content[json_helper::GIT_KEY], false);
    let new_json_local = json_helper::remove_package_from_array(package_dir, &json_content[json_helper::PACKAGES_KEY][json_helper::LOCAL_KEY], true);
    let new_lock_local = json_helper::remove_package_from_array(package_dir, &lock_content[json_helper::LOCAL_KEY], true);
    let new_json_global = json_helper::remove_package_from_array(package_dir, &json_content[json_helper::PACKAGES_KEY][json_helper::GLOBAL_KEY], true);
    let new_lock_global = json_helper::remove_package_from_array(package_dir, &lock_content[json_helper::GLOBAL_KEY], true);

    match json_helper::write("rubigo.json", "", Some(object!{
        json_helper::INFO_KEY => json_content[json_helper::INFO_KEY].clone(),
        json_helper::PACKAGES_KEY => object!{
            json_helper::GIT_KEY => new_json_git,
            json_helper::LOCAL_KEY => new_json_local,
            json_helper::GLOBAL_KEY => new_json_global
        }
    })) {
        Ok(_) => logger.verbose("Update file", "rubigo.json"),
        Err(e) => {
            logger.fatal(e);
            return
        },
    }

    match json_helper::write("rubigo.lock", "", Some(object!{
            json_helper::GIT_KEY => new_lock_git,
            json_helper::LOCAL_KEY => new_lock_local,
            json_helper::GLOBAL_KEY => new_lock_global
    })) {
        Ok(_) => logger.verbose("Update file", "rubigo.lock"),
        Err(e) => {
            match json_helper::write("rubigo.json", "", Some(json_content)) {
                Ok(_) => logger.verbose("Revert file", "rubigo.json"),
                Err(e) => logger.error(format!("unable to revert `rubigo.json`: {}", e)),
            }
            logger.fatal(e);
            return
        },
    }

    let pkg_path_buf = helpers::get_path_from_url(package_dir);
    let pkg_path = pkg_path_buf.as_path();
    if pkg_path.exists() {
        if !helpers::remove_package(package_dir, logger) {
            match json_helper::write("rubigo.json", "", Some(json_content)) {
                Ok(_) => logger.verbose("Revert file", "rubigo.json"),
                Err(e) => logger.error(format!("unable to revert `rubigo.json`: {}", e)),
            }
            match json_helper::write("rubigo.lock", "", Some(lock_content)) {
                Ok(_) => logger.verbose("Revert file", "rubigo.lock"),
                Err(e) => logger.error(format!("unable to revert `rubigo.lock`: {}", e)),
            }
            process::exit(1);
        }
    }
}

pub fn update(package_url: Option<&str>, should_clean: bool, logger: Logger) {
    let json_content = match json_helper::read(Path::new("rubigo.json")) {
        Ok(content) => content,
        Err(e) => {
            logger.fatal(format!("unable to read `rubigo.json`: {}", e));
            return
        }
    };

    if package_url.is_some() {
        let mut git_pkgs = json_content[json_helper::PACKAGES_KEY][json_helper::GIT_KEY].clone();
        let mut pkg = None;
        if !git_pkgs.is_null() {
            for i in 0..git_pkgs.len() {
                if match git_pkgs[i][json_helper::IMPORT_KEY].as_str() {
                    Some(name) => name,
                    None => continue,
                } == package_url.unwrap() {
                    pkg = Some(git_pkgs.array_remove(i));
                    break;
                }
            }
        }
        if pkg.is_none() {
            let mut global_pkgs = json_content[json_helper::PACKAGES_KEY][json_helper::GLOBAL_KEY].clone();
            if global_pkgs.is_null() {
                logger.fatal(format!("the package `{0}` did not find in `rubigo.json` file", package_url.unwrap()));
                return
            }
            let mut global_pkg = None;
            for i in 0..global_pkgs.len() {
                if match global_pkgs[i].as_str() {
                    Some(name) => name,
                    None => continue,
                } == package_url.unwrap() {
                    global_pkg = Some(match global_pkgs.array_remove(i).as_str() {
                        Some(name) => name.to_owned(),
                        None => {
                            logger.fatal(format!("the package `{0}` is not installed, it could be installed using `rubigo get {0}`", package_url.unwrap()));
                            return
                        }
                    });
                    break;
                }
            }

            if global_pkg.is_none() {
                logger.fatal(format!("the package `{0}` is not installed, it could be installed using `rubigo get {0}`", package_url.unwrap()));
                return
            }

            let g_pkg = global_pkg.unwrap();
            match go::get(&g_pkg, true) {
                true => {
                    logger.verbose("Global package", &g_pkg);
                    let _ = global_pkgs.push(g_pkg);
                },
                false => {
                    logger.fatal(format!("unable to update global package of `{}`", &g_pkg));
                    return
                },
            }

            match json_helper::write("rubigo.lock", "", Some(object!{
                json_helper::GIT_KEY => json_content[json_helper::PACKAGES_KEY][json_helper::GIT_KEY].clone(),
                json_helper::LOCAL_KEY => json_content[json_helper::PACKAGES_KEY][json_helper::LOCAL_KEY].clone(),
                json_helper::GLOBAL_KEY => global_pkgs
            })) {
                Ok(_) => logger.verbose("Update file", "rubigo.lock"),
                Err(e) => logger.error(e),
            }

            return
        }

        let (tx, rx) = channel();
        thread::spawn(move|| {
            vendor::update_package(pkg.unwrap(), should_clean, false, tx, logger);
        });

        match rx.recv() {
            Ok(p) => {
                logger.verbose("Update package", match p[json_helper::IMPORT_KEY].as_str() {
                    Some(import_str) => import_str,
                    None => "unknown",
                });
                let _ = git_pkgs.push(p);
            },
            Err(e) => logger.fatal(e),
        }

        match json_helper::write("rubigo.lock", "", Some(object!{
            json_helper::GIT_KEY => git_pkgs,
            json_helper::LOCAL_KEY => json_content[json_helper::PACKAGES_KEY][json_helper::LOCAL_KEY].clone(),
            json_helper::GLOBAL_KEY => json_content[json_helper::PACKAGES_KEY][json_helper::GLOBAL_KEY].clone()
        })) {
            Ok(_) => logger.verbose("Update file", "rubigo.lock"),
            Err(e) => logger.error(e),
        }

        return;
    }

    let pool = CpuPool::new(2);

    let old_lock_future = pool.spawn_fn(|| {
        match json_helper::read(Path::new("rubigo.lock")) {
            Ok(content_json) => Ok(content_json),
            Err(e) => Err(e),
        }
    });

    let c_json = json_content[json_helper::PACKAGES_KEY][json_helper::LOCAL_KEY].clone();
    let local_packages = pool.spawn_fn(move || {
        Ok::<JsonValue, ()>(vendor::install_local_packages(&c_json, logger))
    });

    let c_json2 = json_content[json_helper::PACKAGES_KEY][json_helper::GLOBAL_KEY].clone();
    let global_packages = pool.spawn_fn(move || {
        Ok::<JsonValue, ()>(vendor::install_global_packages(&c_json2, true, logger))
    });

    let git_packages = vendor::install_git_packages(&json_content[json_helper::PACKAGES_KEY][json_helper::GIT_KEY], "Update package", should_clean, false, logger);

    let new_lock = object!{
        json_helper::GIT_KEY => git_packages,
        json_helper::LOCAL_KEY => local_packages.wait().unwrap_or(array![]),
        json_helper::GLOBAL_KEY => global_packages.wait().unwrap_or(array![])
    };

    helpers::remove_diff_packages(&old_lock_future.wait().unwrap_or(object![]), &new_lock, logger);

    match json_helper::write("rubigo.lock", "", Some(new_lock)) {
        Ok(_) => logger.verbose("Update file", "rubigo.lock"),
        Err(e) => logger.error(e),
    }
}
