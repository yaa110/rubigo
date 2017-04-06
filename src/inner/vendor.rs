use std::path::{Path, PathBuf, Component};
use std::fs::read_dir;
use git2::Repository;
use std::ffi::OsStr;
use json::JsonValue;
use threadpool::ThreadPool;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use num_cpus;

pub fn find_packages() -> Arc<Mutex<JsonValue>> {
    let packages = Arc::new(Mutex::new(array![]));
    let pool = ThreadPool::new(num_cpus::get());
    let (tx, rx) = channel();
    let counter = Arc::new(Mutex::new(0));
    let c_counter = counter.clone();

    let cp_tx = tx.clone();
    let cp_counter = counter.clone();
    let cp_pkgs = packages.clone();
    pool.execute(move || {
        find_package(String::from("vendor"), cp_pkgs, cp_tx, cp_counter);
    });

    while match c_counter.lock() {
        Ok(ptr) => *ptr > 0,
        _ => false,
    } {
        match c_counter.lock() {
            Ok(mut ptr) => *ptr -= 1,
            _ => (),
        };
        match rx.recv() {
            Ok(path_opt) => match path_opt {
                Some(p) => {
                    let cp_tx = tx.clone();
                    let cp_counter = counter.clone();
                    let cp_pkgs = packages.clone();
                    pool.execute(move || {
                        find_package(p, cp_pkgs, cp_tx, cp_counter);
                    });
                },
                None => (),
            },
            _ => (),
        }
    }
    packages
}

fn find_package(dir_path: String, packages: Arc<Mutex<JsonValue>>, tx: Sender<Option<String>>, counter: Arc<Mutex<i32>>) {
    let c_counter = counter.clone();
    match c_counter.lock() {
        Ok(mut ptr) => *ptr += 1,
        _ => (),
    };

    match read_dir(Path::new(dir_path.as_str())) {
        Ok(paths) => {
            for entry in paths {
                match entry {
                    Ok(p) => {
                        let path_buf: PathBuf = p.path();
                        let path: &Path = path_buf.as_path();
                        if path.is_dir() {
                            let c_counter = counter.clone();
                            match c_counter.lock() {
                                Ok(mut ptr) => *ptr += 1,
                                _ => (),
                            };
                            let c_tx = tx.clone();
                            if path.join(".git").as_path().is_dir() {
                                match Repository::open(path) {
                                    Ok(repo) => {
                                        match parse_repository(repo) {
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
                                c_tx.send(None).unwrap();
                            } else {
                                match path.to_str() {
                                    Some(path_str) => c_tx.send(Some(path_str.to_owned())).unwrap(),
                                    None => c_tx.send(None).unwrap(),
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
    let c_tx = tx.clone();
    c_tx.send(None).unwrap();
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

fn parse_repository(repo: Repository) -> Option<JsonValue> {
    let mut pkg = object!{};
    match repo.head() {
        Ok(ref reference) => match reference.name() {
            Some(ref_name) => {
                pkg["version"] = ref_name.into();
                // TODO set it via default
                pkg["update"] = "fixed".into();
            }
            None => return None,
        },
        _ => return None,
    }
    match repo.remotes() {
        Ok(rmts) => match rmts.get(0) {
            Some(url) => {
                pkg["repo"] = url.into();
            }
            None => (),
        },
        _ => (),
    }
    pkg["import"] = parse_import(repo.path()).into();
    Some(pkg)
}
