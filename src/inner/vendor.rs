use std::path::{Path, Component};
use std::fs::{read_dir, remove_dir_all, create_dir_all};
use git2::{Repository, BranchType, ResetType};
use std::ffi::OsStr;
use json::JsonValue;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use inner::logger::Logger;
use inner::{git_helper, go, helpers, json_helper};

pub const VENDOR_DIR: &'static str = "vendor";

pub fn find_packages(logger: Logger) -> JsonValue {
    let packages = Arc::new(Mutex::new(array![]));
    let pool = helpers::new_thread_pool();
    let (tx, rx) = channel();
    let counter = Arc::new(Mutex::new(0));

    match counter.lock() {
        Ok(mut ptr) => *ptr += 1,
        _ => (),
    }
    let cp_tx = tx.clone();
    let cp_counter = counter.clone();
    let cp_pkgs = packages.clone();
    pool.execute(move || {
        parse_dir(String::from(VENDOR_DIR), cp_pkgs, cp_tx, cp_counter, logger);
    });

    while match counter.lock() {
        Ok(ptr) => *ptr > 0,
        _ => false,
    } {
        match counter.lock() {
            Ok(mut ptr) => *ptr -= 1,
            _ => (),
        }
        match rx.recv() {
            Ok(path_opt) => match path_opt {
                Some(p) => {
                    match counter.lock() {
                        Ok(mut ptr) => *ptr += 1,
                        _ => (),
                    }
                    let cp_tx = tx.clone();
                    let cp_counter = counter.clone();
                    let cp_pkgs = packages.clone();
                    pool.execute(move || {
                        parse_dir(p, cp_pkgs, cp_tx, cp_counter, logger);
                    });
                },
                None => (),
            },
            _ => (),
        }
    }
    match Arc::try_unwrap(packages) {
        Ok(pkgs_mut) => match pkgs_mut.into_inner() {
            Ok(pkgs_array) => pkgs_array,
            _ => array![],
        },
        _ => array![],
    }
}

pub fn install_local_packages(local_packages: &JsonValue, logger: Logger) -> JsonValue {
    let mut installed_packages = array![];
    if !local_packages.is_null() {
        for i in 0..local_packages.len() {
            let local_pkg = match local_packages[i].as_str() {
                Some(val_str) => val_str,
                None => continue,
            };
            let dir_path = Path::new(VENDOR_DIR).join(local_pkg);
            if !dir_path.is_dir() {
                match create_dir_all(dir_path) {
                    Ok(_) => {
                        let _ = installed_packages.push(local_pkg);
                        logger.verbose("Create directory", local_pkg)
                    },
                    Err(e) => logger.error(e),
                }
            } else {
                let _ = installed_packages.push(local_pkg);
            }
        }
    }
    installed_packages
}

pub fn install_global_packages(global_packages: &JsonValue, should_update: bool, logger: Logger) -> JsonValue {
    let mut installed_packages = array![];
    if !global_packages.is_null() {
        for i in 0..global_packages.len() {
            let global_pkg = match global_packages[i].as_str() {
                Some(val_str) => val_str,
                None => continue,
            };
            match go::get(global_pkg, should_update) {
                true => {
                    let _ = installed_packages.push(global_pkg);
                    logger.verbose("Global package", global_pkg)
                },
                false => logger.error(format!("Unable to install global package `{}`", global_pkg)),
            }
        }
    }
    installed_packages
}

pub fn install_git_packages(packages: &JsonValue, msg_title: &str, should_clean: bool, is_apply: bool, logger: Logger) -> JsonValue {
    if packages.is_null() {
        return array![]
    }

    let length = packages.len();
    if length == 0 {
        return array![]
    }

    let pool = helpers::new_thread_pool();
    let (tx, rx) = channel();

    for i in 0..length {
        let package = packages[i].clone();
        let c_tx = tx.clone();
        pool.execute(move || {
            update_package(package, should_clean, is_apply, c_tx, logger);
        });
    }

    let mut git_packages = array![];
    for pkg in rx.iter().take(length) {
        logger.verbose(msg_title, match pkg[json_helper::IMPORT_KEY].as_str() {
            Some(import_str) => import_str,
            None => continue,
        });
        if !is_apply {
            let _ = git_packages.push(pkg);
        }
    }
    git_packages
}

pub fn update_package(package: JsonValue, should_clean: bool, is_apply: bool, tx: Sender<JsonValue>, logger: Logger) {
    let mut mut_pkg = package.clone();
    let pkg_import_raw = helpers::strip_url_scheme(match package[json_helper::IMPORT_KEY].as_str() {
        Some(import_str) => import_str,
        None => {
            logger.error("unable to get `import` value");
            let _ = tx.send(mut_pkg);
            return
        },
    });
    let pkg_import = pkg_import_raw.as_str();

    let http_import = helpers::modify_golang_org(pkg_import);
    let repo_url = match package[json_helper::REPO_KEY].as_str() {
        Some(repo_str) => repo_str,
        None => http_import.as_str(),
    };

    let pkg_path_buf = helpers::get_path_from_url(pkg_import);
    let pkg_path = pkg_path_buf.as_path();
    if should_clean && pkg_path.exists() {
        match remove_dir_all(pkg_path) {
            Ok(_) => logger.verbose("Clean package", pkg_path.to_str().unwrap_or("unknown")),
            Err(e) => {
                logger.error(format!("{} {}", pkg_import, e));
                let _ = tx.send(mut_pkg);
                return
            }
        }
    }

    let repo = if should_clean || !pkg_path.is_dir() {
        match Repository::clone(repo_url, pkg_path) {
            Ok(repo) => {
                logger.verbose("Clone repository", pkg_import);
                repo
            },
            Err(e) => {
                logger.error(format!("{} {}", pkg_import, e));
                let _ = tx.send(mut_pkg);
                return
            },
        }
    } else {
        match Repository::open(pkg_path) {
            Ok(repo) => {
                logger.verbose("Open repository", pkg_import);
                if !is_apply {
                    match repo.remotes() {
                        Ok(remotes) => match remotes.get(0) {
                            Some(remote_name) => match repo.find_remote(remote_name) {
                                Ok(mut remote) => match remote.fetch(&[], None, None) {
                                    Ok(_) => {
                                        logger.verbose("Fetch repository", pkg_import);
                                        match repo.branches(Some(BranchType::Local)) {
                                            Ok(branches) => for branch in branches {
                                                match branch {
                                                    Ok(br) => {
                                                        let branch = br.0;
                                                        let branch_ref_name = match branch.get().name() {
                                                            Some(name) => name.to_owned(),
                                                            None => continue,
                                                        };
                                                        let remote_name = match branch.upstream() {
                                                            Ok(remote_branch) => match remote_branch.name() {
                                                                Ok(remote_branch_name) => match remote_branch_name {
                                                                    Some(name) => name.to_owned(),
                                                                    None => continue,
                                                                },
                                                                _ => continue,
                                                            },
                                                            _ => continue,
                                                        };
                                                        let remote_object = match repo.revparse_single(remote_name.as_str()) {
                                                            Ok(obj) => obj,
                                                            _ => continue,
                                                        };
                                                        match repo.set_head(branch_ref_name.as_str()) {
                                                            Ok(_) => (),
                                                            _ => continue,
                                                        }
                                                        match repo.reset(&remote_object, ResetType::Hard, None) {
                                                            Ok(_) => logger.verbose("Update branch", format!("{} {}", pkg_import, branch.name().unwrap_or(None).unwrap_or("unknown"))),
                                                            _ => continue,
                                                        }
                                                    },
                                                    _ => (),
                                                }
                                            },
                                            _ => (),
                                        }
                                    },
                                    Err(e) => {
                                        logger.error(format!("{} {}", pkg_import, e));
                                        let _ = tx.send(mut_pkg);
                                        return
                                    },
                                },
                                Err(e) => {
                                    logger.error(format!("{} {}", pkg_import, e));
                                    let _ = tx.send(mut_pkg);
                                    return
                                },
                            },
                            None => {
                                logger.error(format!("{} unable to get remote name of", pkg_import));
                                let _ = tx.send(mut_pkg);
                                return
                            },
                        },
                        Err(e) => {
                            logger.error(format!("{} {}", pkg_import, e));
                            let _ = tx.send(mut_pkg);
                            return
                        },
                    }
                }
                repo
            },
            Err(e) => {
                logger.error(format!("{} {}", pkg_import, e));
                let _ = tx.send(mut_pkg);
                return
            }
        }
    };

    let mut version = match package[json_helper::VERSION_KEY].as_str() {
        Some(version_str) => version_str.to_owned(),
        None => {
            logger.error(format!("{} unable to get `version` value", pkg_import));
            let _ = tx.send(mut_pkg);
            return
        },
    };

    if !is_apply {
        version = git_helper::get_latest_compat_version(&repo, version);
    }

    let version_object = match git_helper::get_revision_object(&repo, pkg_import.to_owned(), version, true, logger) {
        Some(tup) => {
            mut_pkg[json_helper::VERSION_KEY] = tup.1.clone().into();
            tup.0
        },
        None => {
            logger.error(format!("unable to parse the version of `{}`", pkg_import));
            let _ = tx.send(mut_pkg);
            return
        }
    };

    match repo.set_head_detached(version_object.id()) {
        Ok(_) => (),
        Err(e) => {
            logger.error(format!("{} {}", pkg_import, e));
            let _ = tx.send(mut_pkg);
            return
        },
    }

    match repo.reset(&version_object, ResetType::Hard, None){
        Ok(_) => (),
        Err(e) => {
            logger.error(format!("{} {}", pkg_import, e));
            let _ = tx.send(mut_pkg);
            return
        },
    }

    let _ = tx.send(mut_pkg);
}

fn parse_dir(dir_path: String, packages: Arc<Mutex<JsonValue>>, tx: Sender<Option<String>>, counter: Arc<Mutex<i32>>, logger: Logger) {
    match read_dir(Path::new(dir_path.as_str())) {
        Ok(paths) => {
            for entry in paths {
                match entry {
                    Ok(p) => {
                        let path_buf = p.path();
                        let path: &Path = path_buf.as_path();
                        if path.is_dir() {
                            match counter.lock() {
                                Ok(mut ptr) => *ptr += 1,
                                _ => (),
                            }
                            if path.join(".git").as_path().is_dir() {
                                match Repository::open(path) {
                                    Ok(repo) => {
                                        match parse_repository(repo, &logger) {
                                            Some(pkg) => match packages.lock() {
                                                Ok(mut ptr) => {
                                                    let _ = ptr.push(pkg);
                                                },
                                                _ => (),
                                            },
                                            None => (),
                                        }
                                    },
                                    _ => (),
                                }
                                tx.send(None).unwrap();
                            } else {
                                match path.to_str() {
                                    Some(path_str) => {
                                        let _ = tx.send(Some(path_str.to_owned()));
                                    },
                                    None => {
                                        let _ = tx.send(None);
                                    },
                                }
                            }
                        }
                    },
                    _ => (),
                }
            }
        },
        _ => (),
    }
    let _ = tx.send(None);
}

fn parse_import(path: &Path) -> String {
    let mut parts = Vec::new();
    let mut is_vendor_found = false;
    let vendor_os_str = OsStr::new(VENDOR_DIR);
    let git_os_str = OsStr::new(".git");
    for comp in path.components() {
        match comp {
            Component::Normal(c) => {
                if is_vendor_found && c != git_os_str {
                    let c_str = match c.to_str() {
                        Some(s) => s,
                        None => continue,
                    };
                    parts.push(c_str);
                } else if c == vendor_os_str {
                    is_vendor_found = true;
                }
            },
            _ => (),
        }
    }
    parts.join("/")
}

fn parse_repository(repo: Repository, logger: &Logger) -> Option<JsonValue> {
    let mut pkg = object!{};
    match repo.head() {
        Ok(r) => match r.resolve() {
            Ok (ref reference) => match reference.target() {
                Some(o_id) => {
                    pkg[json_helper::VERSION_KEY] = if reference.is_tag() {
                        match repo.find_tag(o_id) {
                            Ok(tag) => tag.name().unwrap_or(format!("{}", o_id).as_str()).into(),
                            _ => format!("{}", o_id).into(),
                        }
                    } else if reference.is_branch() {
                        reference.shorthand().unwrap_or(format!("{}", o_id).as_str()).into()
                    } else {
                        format!("{}", o_id).into()
                    };
                },
                None => return None,
            },
            _ => return None,
        },
        _ => return None,
    }
    let pkg_import = parse_import(repo.path());
    logger.verbose("Find package", &pkg_import);
    pkg[json_helper::IMPORT_KEY] = pkg_import.into();
    Some(pkg)
}
