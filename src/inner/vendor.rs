use std::path::{Path, PathBuf, Component};
use std::fs::read_dir;
use git2::Repository;
use std::ffi::OsStr;
use json::JsonValue;
use threadpool::ThreadPool;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use num_cpus;
use inner::logger::Logger;

pub fn find_packages(logger: Logger) -> JsonValue {
    let packages = Arc::new(Mutex::new(array![]));
    let threads_num = num_cpus::get();
    let pool = ThreadPool::new(if threads_num > 1 {
        threads_num
    } else {
        2
    });
    let (tx, rx) = channel();
    let counter = Arc::new(Mutex::new(0));

    match counter.lock() {
        Ok(mut ptr) => *ptr += 1,
        _ => (),
    };
    let cp_tx = tx.clone();
    let cp_counter = counter.clone();
    let cp_pkgs = packages.clone();
    pool.execute(move || {
        parse_dir(String::from("vendor"), cp_pkgs, cp_tx, cp_counter, logger);
    });

    while match counter.lock() {
        Ok(ptr) => *ptr > 0,
        _ => false,
    } {
        match counter.lock() {
            Ok(mut ptr) => *ptr -= 1,
            _ => (),
        };
        match rx.recv() {
            Ok(path_opt) => match path_opt {
                Some(p) => {
                    match counter.lock() {
                        Ok(mut ptr) => *ptr += 1,
                        _ => (),
                    };
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

fn parse_dir(dir_path: String, packages: Arc<Mutex<JsonValue>>, tx: Sender<Option<String>>, counter: Arc<Mutex<i32>>, logger: Logger) {
    match read_dir(Path::new(dir_path.as_str())) {
        Ok(paths) => {
            for entry in paths {
                match entry {
                    Ok(p) => {
                        let path_buf: PathBuf = p.path();
                        let path: &Path = path_buf.as_path();
                        if path.is_dir() {
                            match counter.lock() {
                                Ok(mut ptr) => *ptr += 1,
                                _ => (),
                            };
                            if path.join(".git").as_path().is_dir() {
                                match Repository::open(path) {
                                    Ok(repo) => {
                                        match parse_repository(repo, &logger) {
                                            Some(pkg) => match packages.lock() {
                                                Ok(mut ptr) => match ptr.push(pkg) {
                                                    _ => (),
                                                },
                                                _ => (),
                                            },
                                            None => (),
                                        }
                                    },
                                    _ => (),
                                };
                                tx.send(None).unwrap();
                            } else {
                                match path.to_str() {
                                    Some(path_str) => {
                                        match tx.send(Some(path_str.to_owned())) {
                                           _ => (),
                                        };
                                    },
                                    None => {
                                        match tx.send(None) {
                                            _ => (),
                                        };
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
    match tx.send(None) {
        _ => (),
    };
}

fn parse_import(path: &Path) -> String {
    let mut parts = Vec::new();
    let mut is_vendor_found = false;
    let vendor_os_str = OsStr::new("vendor");
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
        Ok(ref reference) => {
            match reference.target() {
                Some(o_id) => {
                    pkg["version"] = if reference.is_tag() {
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
            };

            // TODO set it via default
            pkg["update"] = "fixed".into();
        },
        _ => return None,
    }
    let pkg_import = parse_import(repo.path());
    logger.verbose("Find package", &pkg_import);
    pkg["import"] = pkg_import.into();
    Some(pkg)
}
