extern crate time;

use ansi_term::Color::{Red, Fixed, Yellow};
use std::process;
use std::fmt::Display;

#[derive(PartialEq, Eq, Debug)]
pub enum Verbosity {
    High,
    Low,
    None,
}

pub fn log_verbose<T: Display>(title: &str, msg: T, verb: &Verbosity) {
    if *verb == Verbosity::High {
        println!("[{}] {} {}", Fixed(8).paint(time::strftime("%T", &time::now()).unwrap_or(String::from("00:00:00"))), Yellow.paint(title), msg);
    }
}

pub fn log_error<T: Display>(err: T, verb: &Verbosity) {
    if *verb != Verbosity::None {
        println!("{} {}", Red.paint("error:"), err);
    }
}

pub fn log_fatal<T: Display>(err: T, verb: &Verbosity) {
    log_error(err, verb);
    process::exit(1)
}
