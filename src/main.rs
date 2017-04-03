// In the name of Allah
// --------------------------------

extern crate clap;
extern crate ansi_term;
extern crate git2;
#[macro_use]
extern crate json;

mod inner;
mod controller;

use clap::{Arg, App, SubCommand};
use std::process;
use controller::*;
use inner::logger::Verbosity;
use inner::logger::log_fatal;

const VERSION: &'static str = "0.1.0";

fn main() {
    let matches = App::new("Rubigo")
        .version(VERSION)
        .name("Rubigo")
        .about(" Golang dependency tool and package manager\n For more information, please visit https://github.com/yaa110/rubigo")
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Use verbose output")
            .takes_value(false))
        .arg(Arg::with_name("quiet")
            .short("q")
            .long("quiet")
            .conflicts_with("verbose")
            .help("Print no output")
            .takes_value(false))
        .subcommand(SubCommand::with_name("new")
            .version(VERSION)
            .alias("create")
            .arg(Arg::with_name("name")
                .help("The name of project")
                .required(true))
            .arg(Arg::with_name("library")
                .short("l")
                .long("lib")
                .help("Create a new library project")
                .conflicts_with("binary")
                .takes_value(false))
            .arg(Arg::with_name("binary")
                .short("b")
                .long("bin")
                .help("Create a new executable project (Default)")
                .takes_value(false))
            .about("Create a new Rubigo project"))
        .subcommand(SubCommand::with_name("init")
            .version(VERSION)
            .alias("start")
            .about("Initialize Rubigo project in an existing directory"))
        .subcommand(SubCommand::with_name("get")
            .version(VERSION)
            .alias("add")
            .arg(Arg::with_name("package")
                .help("The path of package repository")
                .required(true))
            .about("Add a package to dependencies and clone it into `vendor` directory"))
        .subcommand(SubCommand::with_name("remove")
            .version(VERSION)
            .alias("rm")
            .arg(Arg::with_name("package")
                .help("The path of package repository")
                .required(true))
            .about("Remove a package from dependencies and `vendor` directory"))
        .subcommand(SubCommand::with_name("update")
            .version(VERSION)
            .alias("up")
            .arg(Arg::with_name("package")
                .help("The path of package repository"))
            .arg(Arg::with_name("all")
                .short("a")
                .long("all")
                .help("Update all packages (Default)")
                .conflicts_with("package")
                .takes_value(false))
            .about("Update one or all packages"))
        .subcommand(SubCommand::with_name("local")
            .version(VERSION)
            .arg(Arg::with_name("directory")
                .required(true)
                .help("The directory name of local package"))
            .about("Create a new local package in `vendor` directory"))
        .subcommand(SubCommand::with_name("global")
            .version(VERSION)
            .arg(Arg::with_name("package")
                .required(true)
                .help("The path of package repository"))
            .about("Install a package in `GOPATH/src` directory"))
        .subcommand(SubCommand::with_name("list")
            .alias("ls")
            .version(VERSION)
            .arg(Arg::with_name("all")
                .short("a")
                .long("all")
                .conflicts_with_all(&["global", "remote", "local"])
                .help("List all dependencies (Default)")
                .takes_value(false))
            .arg(Arg::with_name("local")
                .short("l")
                .long("local")
                .conflicts_with_all(&["global", "remote", "all"])
                .help("List local dependencies")
                .takes_value(false))
            .arg(Arg::with_name("global")
                .short("g")
                .long("global")
                .conflicts_with_all(&["remote", "local", "all"])
                .help("List local dependencies")
                .takes_value(false))
            .arg(Arg::with_name("remote")
                .short("r")
                .long("remote")
                .conflicts_with_all(&["global", "local", "all"])
                .help("List remote dependencies with git repositories")
                .takes_value(false))
            .about("Display a list of dependencies"))
        .subcommand(SubCommand::with_name("apply")
            .version(VERSION)
            .about("Apply the changes of `rubigo.json` to dependencies in `vendor` directory"))
        .subcommand(SubCommand::with_name("info")
            .version(VERSION)
            .arg(Arg::with_name("edit")
                .short("e")
                .long("edit")
                .help("Edit information about this Rubigo project")
                .takes_value(false))
            .about("Display information about this Rubigo project"))
        .get_matches();

    let mut verb = Verbosity::LOW;
    if matches.is_present("verbose") {
        verb = Verbosity::HIGH;
    } else if matches.is_present("quiet") {
        verb = Verbosity::NONE;
    }

    match matches.subcommand_name() {
        Some("apply") => package::apply(&verb),
        Some("get") => package::get(match matches.subcommand_matches("get") {
            Some(args) => match args.value_of("package") {
                Some(value) => value,
                None => {
                    log_fatal("unable to get argument of `get` sub command", &verb);
                    return
                },
            },
            None => {
                log_fatal("unable to get argument of `get` sub command", &verb);
                return
            },
        }, &verb),
        Some("global") => package::global(match matches.subcommand_matches("global") {
            Some(args) => match args.value_of("package") {
                Some(value) => value,
                None => {
                    log_fatal("unable to get argument of `global` sub command", &verb);
                    return
                },
            },
            None => {
                log_fatal("unable to get argument of `global` sub command", &verb);
                return
            },
        }, &verb),
        Some("info") => {
            if match matches.subcommand_matches("info") {
                Some(args) => args.is_present("edit"),
                None => {
                    log_fatal("unable to get argument of `info` sub command", &verb);
                    return
                },
            } {
                info::edit()
            } else {
                info::display()
            }
        },
        Some("init") => project::init(&verb),
        Some("list") => {
            let list_matches = match matches.subcommand_matches("list") {
                Some(args) => args,
                None => {
                    log_fatal("unable to get argument of `list` sub command", &verb);
                    return
                },
            };
            if list_matches.is_present("local") {
                list::local()
            } else if list_matches.is_present("remote") {
                list::remote()
            } else if list_matches.is_present("global") {
                list::global()
            } else {
                list::all()
            }
        },
        Some("local") => package::local(match matches.subcommand_matches("local") {
            Some(args) => match args.value_of("directory") {
                Some(value) => value,
                None => {
                    log_fatal("unable to get argument of `local` sub command", &verb);
                    return
                },
            },
            None => {
                log_fatal("unable to get argument of `local` sub command", &verb);
                return
            },
        }, &verb),
        Some("new") => {
            let new_matches = match matches.subcommand_matches("new") {
                Some(args) => args,
                None => {
                    log_fatal("unable to get argument of `new` sub command", &verb);
                    return
                },
            };
            project::new(match new_matches.value_of("name") {
                Some(value) => value,
                None => {
                    log_fatal("unable to get `name` argument of `new` sub command", &verb);
                    return
                },
            }, new_matches.is_present("library"), &verb)
        },
        Some("remove") => package::remove(match matches.subcommand_matches("remove") {
            Some(args) => match args.value_of("package") {
                Some(value) => value,
                None => {
                    log_fatal("unable to get argument of `remove` sub command", &verb);
                    return
                },
            },
            None => {
                log_fatal("unable to get argument of `remove` sub command", &verb);
                return
            },
        }, &verb),
        Some("update") => {
            let update_matches = match matches.subcommand_matches("update") {
                Some(args) => args,
                None => {
                    log_fatal("unable to get argument of `update` sub command", &verb);
                    return
                },
            };
            if update_matches.is_present("package") {
                package::update(Some(match update_matches.value_of("package") {
                    Some(value) => value,
                    None => {
                        log_fatal("unable to get `package` argument of `update` sub command", &verb);
                        return
                    },
                }), &verb)
            } else {
                package::update(None, &verb)
            }
        },
        _ => {
            println!("{}", matches.usage());
            process::exit(1)
        },
    }
}
