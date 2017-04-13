use std::process::Command;
use std::ffi::OsStr;

pub fn get(package_name: &str, should_update: bool) -> bool {
    match should_update {
        true => run(&["get", "-u", package_name]),
        false => run(&["get", package_name])
    }
}

fn run<S: AsRef<OsStr>>(args: &[S]) -> bool {
    match Command::new("go").args(args).status() {
        Ok(exit_status) => exit_status.success(),
        _ => false,
    }
}
