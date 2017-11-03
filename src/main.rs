// In the name of Allah
// --------------------------------

extern crate clap;
extern crate git2;
#[macro_use]
extern crate json;
extern crate threadpool;
extern crate num_cpus;
extern crate futures;
extern crate futures_cpupool;
extern crate semver;
extern crate regex;
extern crate curl;

mod inner;
mod controller;
#[cfg(test)]
mod tests;

use clap::{Arg, App, SubCommand, AppSettings};
use std::process;
use controller::*;
use inner::logger::{Logger, Verbosity};

const VERSION: &'static str = "1.0.3";

fn main() {
    let matches = App::new("Rubigo")
        .version(VERSION)
        .name("Rubigo")
        .setting(AppSettings::VersionlessSubcommands)
        .about("Golang dependency tool and package manager\nFor more information, please visit https://github.com/yaa110/rubigo")
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Use verbose output")
            .takes_value(false))
        .arg(Arg::with_name("no-prompt")
            .short("y")
            .long("yes")
            .help("Continue without prompt for a confirmation")
            .takes_value(false))
        .arg(Arg::with_name("quiet")
            .short("q")
            .long("quiet")
            .conflicts_with("verbose")
            .help("Print no output")
            .takes_value(false))
        .subcommand(SubCommand::with_name("new")
            .visible_alias("create")
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
            .about("Create a new Golang project"))
        .subcommand(SubCommand::with_name("init")
            .visible_alias("start")
            .about("Initialize Rubigo project in an existing directory"))
        .subcommand(SubCommand::with_name("reset")
            .visible_alias("sync")
            .about("Update `rubigo.json` and `rubigo.lock` to the list of packages in `vendor` directory"))
        .subcommand(SubCommand::with_name("get")
            .visible_alias("add")
            .arg(Arg::with_name("package")
                .help("The import path of package")
                .required(true))
            .arg(Arg::with_name("repository")
                .short("r")
                .long("repo")
                .value_name("repository")
                .help("Clone the package from the provided `repository` rather than its main url")
                .require_equals(true)
                .required(false)
                .conflicts_with_all(&["local", "global"])
                .takes_value(true))
            .arg(Arg::with_name("global")
                .short("g")
                .long("global")
                .help("Install the package in `GOPATH/src` directory")
                .required(false))
            .arg(Arg::with_name("local")
                .short("l")
                .long("local")
                .conflicts_with("global")
                .help("Create a new local package in `vendor` directory")
                .required(false))
            .about("Add a package to dependencies and clone it into `vendor` directory"))
        .subcommand(SubCommand::with_name("remove")
            .visible_alias("rm")
            .arg(Arg::with_name("package")
                .help("The import path of package")
                .required(true))
            .about("Remove a package from manifest and `vendor` directory"))
        .subcommand(SubCommand::with_name("update")
            .visible_alias("up")
            .arg(Arg::with_name("clean")
                .short("c")
                .long("clean")
                .help("Remove the package directory and clone from the repository")
                .takes_value(false))
            .arg(Arg::with_name("package")
                .help("The import path of package"))
            .arg(Arg::with_name("all")
                .short("a")
                .long("all")
                .help("Update all packages (Default)")
                .conflicts_with("package")
                .takes_value(false))
            .about("Update one or all packages and apply the changes of `rubigo.json` to `rubigo.lock` and packages in `vendor` directory"))
        .subcommand(SubCommand::with_name("list")
            .visible_alias("ls")
            .arg(Arg::with_name("all")
                .short("a")
                .long("all")
                .conflicts_with_all(&["global", "remote", "local"])
                .help("List all packages from `rubigo.lock` file (Default)")
                .takes_value(false))
            .arg(Arg::with_name("local")
                .short("l")
                .long("local")
                .help("List local packages from `rubigo.lock` file")
                .takes_value(false))
            .arg(Arg::with_name("global")
                .short("g")
                .long("global")
                .help("List global packages from `rubigo.lock` file")
                .takes_value(false))
            .arg(Arg::with_name("remote")
                .short("r")
                .long("remote")
                .help("List remote (git) packages from `rubigo.lock` file")
                .takes_value(false))
            .about("Display a list of dependencies from `rubigo.lock` file"))
        .subcommand(SubCommand::with_name("apply")
            .visible_alias("install")
            .arg(Arg::with_name("clean")
                .short("c")
                .long("clean")
                .help("Remove the package directory and clone from the repository")
                .takes_value(false))
            .about("Apply the changes of `rubigo.lock` to packages in `vendor` directory"))
        .subcommand(SubCommand::with_name("info")
            .visible_alias("about")
            .about("Display the information about this Rubigo project"))
        .get_matches();

    let logger = Logger::new(if matches.is_present("verbose") {
        Verbosity::High
    } else if matches.is_present("quiet") {
        Verbosity::None
    } else {
        Verbosity::Low
    });

    match matches.subcommand_name() {
        Some("apply") => {
            let apply_matches = match matches.subcommand_matches("apply") {
                Some(args) => args,
                None => {
                    logger.fatal("unable to get argument of `apply` sub command");
                    return
                },
            };
            project::apply(apply_matches.is_present("clean"), logger)
        },
        Some("get") => {
            let get_matches = match matches.subcommand_matches("get") {
                Some(args) => args,
                None => {
                    logger.fatal("unable to get argument of `get` sub command");
                    return
                },
            };
            package::get(match get_matches.value_of("package") {
                Some(pkg) => pkg,
                None => {
                    logger.fatal("unable to get `package` argument of `get` sub command");
                    return
                }
            }, get_matches.value_of("repository"), matches.is_present("no-prompt"), get_matches.is_present("global"), get_matches.is_present("local"), logger);
        },
        Some("info") => info::display(&logger),
        Some("init") => project::init(logger),
        Some("reset") => project::reset(matches.is_present("no-prompt"), logger),
        Some("list") => {
            let list_matches = match matches.subcommand_matches("list") {
                Some(args) => args,
                None => {
                    logger.fatal("unable to get argument of `list` sub command");
                    return
                },
            };
            list::list(list_matches.is_present("local"), list_matches.is_present("remote"), list_matches.is_present("global"), &logger);
        },
        Some("new") => {
            let new_matches = match matches.subcommand_matches("new") {
                Some(args) => args,
                None => {
                    logger.fatal("unable to get argument of `new` sub command");
                    return
                },
            };
            project::new(match new_matches.value_of("name") {
                Some(value) => value,
                None => {
                    logger.fatal("unable to get `name` argument of `new` sub command");
                    return
                },
            }, new_matches.is_present("library"), &logger)
        },
        Some("remove") => package::remove(match matches.subcommand_matches("remove") {
            Some(args) => match args.value_of("package") {
                Some(value) => value,
                None => {
                    logger.fatal("unable to get argument of `remove` sub command");
                    return
                },
            },
            None => {
                logger.fatal("unable to get argument of `remove` sub command");
                return
            },
        }, logger),
        Some("update") => {
            let update_matches = match matches.subcommand_matches("update") {
                Some(args) => args,
                None => {
                    logger.fatal("unable to get argument of `update` sub command");
                    return
                },
            };
            if update_matches.is_present("package") {
                package::update(Some(match update_matches.value_of("package") {
                    Some(value) => value,
                    None => {
                        logger.fatal("unable to get `package` argument of `update` sub command");
                        return
                    },
                }), update_matches.is_present("clean"), logger)
            } else {
                package::update(None, update_matches.is_present("clean"), logger)
            }
        },
        _ => {
            logger.error("No sub command has been provided. Please run `rubigo --help` for more information");
            process::exit(1)
        },
    }
}
