use inner::logger::Verbosity;

pub fn apply(verb: &Verbosity) {
    println!("{:?}", verb);
    // TODO
}

pub fn get(package_url: &str, verb: &Verbosity) {
    println!("{}:{:?}", package_url, verb);
    // TODO
}

pub fn global(package_url: &str, verb: &Verbosity) {
    println!("{}:{:?}", package_url, verb);
    // TODO
}

pub fn local(dir_name: &str, verb: &Verbosity) {
    println!("{}:{:?}", dir_name, verb);
    // TODO
}

pub fn remove(package_dir: &str, verb: &Verbosity) {
    println!("{}:{:?}", package_dir, verb);
    // TODO
}

pub fn update(package_url: Option<&str>, verb: &Verbosity) {
    println!("{}:{:?}", package_url.unwrap_or("all"), verb);
    // TODO
}
