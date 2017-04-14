use inner::logger::Logger;
use inner::json_helper;
use std::path::Path;
use json::JsonValue;
use inner::list_helper::{print_header, print_git_packages, print_str_packages};

pub fn list(is_local: bool, is_remote: bool, is_global: bool, logger: &Logger) {
    let lock_content = match json_helper::read(Path::new("rubigo.lock")) {
        Ok(content) => content,
        Err(e) => {
            logger.fatal(format!("unable to read `rubigo.lock`: {}", e));
            return
        }
    };

    let is_all = !(is_local || is_remote || is_global);

    if is_remote || is_all {
        list_remote(&lock_content[json_helper::GIT_KEY]);
    }

    if is_local || is_all {
        list_local(&lock_content[json_helper::LOCAL_KEY]);
    }

    if is_global || is_all {
        list_global(&lock_content[json_helper::GLOBAL_KEY]);
    }
}

fn list_global(content: &JsonValue) {
    if content.len() == 0 {
        return
    }
    print_header("Global packages", content.len());
    print_str_packages(content);
}

fn list_local(content: &JsonValue) {
    if content.len() == 0 {
        return
    }
    print_header("Local packages", content.len());
    print_str_packages(content);
}

fn list_remote(content: &JsonValue) {
    if content.len() == 0 {
        return
    }
    print_header("Remote packages", content.len());
    print_git_packages(content);
}
