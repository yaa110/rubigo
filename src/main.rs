// In the name of Allah
// --------------------------------

extern crate clap;

mod verbosity;
mod project;

use clap::{Arg, App, SubCommand};
use std::process;
use verbosity::Verbosity;
use project::new_project;

const VERSION: &'static str = "v0.1.0";

fn main() {
    let matches = App::new(r#"
 _____       _     _
|  __ \     | |   (_)
| |__) |   _| |__  _  __ _  ___
|  _  / | | | '_ \| |/ _` |/ _ \
| | \ \ |_| | |_) | | (_| | (_) |
|_|  \_\__,_|_.__/|_|\__, |\___/
                      __/ |
                     |___/

"#)
        .version(VERSION)
        .name("rubigo")
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
                .help("Update all packages")
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
        .subcommand(SubCommand::with_name("run")
            .version(VERSION)
            .about("Execute `go run`"))
        .subcommand(SubCommand::with_name("build")
            .version(VERSION)
            .about("Build the Rubigo project by running `go build`"))
        .subcommand(SubCommand::with_name("list")
            .alias("ls")
            .version(VERSION)
            .about("Display a list of all dependencies"))
        .subcommand(SubCommand::with_name("apply")
            .version(VERSION)
            .about("Apply the changes of `rubigo.json` to dependencies in `vendor` directory"))
        .subcommand(SubCommand::with_name("info")
            .version(VERSION)
            .arg(Arg::with_name("edit")
                .short("e")
                .long("edit")
                .help("Edit information about this Rubigo project")
                .conflicts_with("package")
                .takes_value(false))
            .about("Display information about this Rubigo project"))
        .get_matches();
}
