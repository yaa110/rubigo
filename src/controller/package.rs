use inner::logger::Logger;

pub fn apply(logger: &Logger) {
    // TODO
}

pub fn get(package_url: &str, logger: &Logger) {
    // TODO
}

pub fn global(package_url: &str, logger: &Logger) {
    // TODO
}

pub fn local(dir_name: &str, logger: &Logger) {
    // TODO
}

pub fn remove(package_dir: &str, logger: &Logger) {
    // TODO
}

pub fn update(package_url: Option<&str>, logger: &Logger) {
    println!("{}", package_url.unwrap_or("all"));
    // TODO
}
