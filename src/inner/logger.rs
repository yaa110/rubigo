extern crate time;

use std::process;
use std::fmt::Display;
use std::io::{self, Write};

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Verbosity {
    High,
    Low,
    None,
}

#[derive(Copy, Clone)]
pub struct Logger {
    verbosity: Verbosity,
}

impl Logger {
    pub fn new(verbosity: Verbosity) -> Self {
        Logger {
            verbosity: verbosity,
        }
    }

    pub fn verbose<T: Display>(&self, title: &str, msg: T) {
        if self.verbosity == Verbosity::High {
            println!("[{}] {} {}", time::strftime("%T", &time::now()).unwrap_or(String::from("00:00:00")), title, msg);
            let _ = io::stdout().flush();
        }
    }

    pub fn error<T: Display>(&self, err: T) {
        if self.verbosity != Verbosity::None {
            let _ = writeln!(&mut io::stderr(), "{} {}", "error:", err);
        }
    }

    pub fn fatal<T: Display>(&self, err: T) {
        self.error(err);
        process::exit(1);
    }
}
