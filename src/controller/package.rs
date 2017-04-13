use inner::logger::Logger;
use futures::Future;
use futures_cpupool::CpuPool;
use inner::{json_helper, vendor};
use std::path::Path;
use json::JsonValue;

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

pub fn update(package_url: Option<&str>, should_clean: bool, logger: Logger) {
    let json_content = match json_helper::read(Path::new("rubigo.json")) {
        Ok(content) => content,
        Err(e) => {
            logger.fatal(e);
            return
        }
    };

    let pool = CpuPool::new(2);

    let c_json = json_content.clone();
    let local_packages = pool.spawn_fn(move || {
        let res: Result<JsonValue, ()> = Ok(vendor::install_local_packages(&c_json["packages"]["local"], logger));
        res
    });

    let c_json2 = json_content.clone();
    let global_packages = pool.spawn_fn(move || {
        let res: Result<JsonValue, ()> = Ok(vendor::install_global_packages(&c_json2["packages"]["global"], true, logger));
        res
    });

    let git_packages = vendor::install_git_packages(&json_content["packages"]["git"], "Check package", should_clean, false, logger);

    match json_helper::write("rubigo.lock", "", Some(object!{
        "git" => git_packages,
        "local" => local_packages.wait().unwrap_or(array![]),
        "global" => global_packages.wait().unwrap_or(array![])
    })) {
        Ok(_) => logger.verbose("Create file", "rubigo.lock"),
        Err(e) => logger.error(e),
    }
}
