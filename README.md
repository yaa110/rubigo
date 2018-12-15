Rubigo
======
[![Build Status](https://travis-ci.org/yaa110/rubigo.svg?branch=master)](https://travis-ci.org/yaa110/rubigo) [![Build status](https://ci.appveyor.com/api/projects/status/gaj2qh18963d0hp1?svg=true)](https://ci.appveyor.com/project/yaa110/rubigo) [![License](http://img.shields.io/:license-mit-blue.svg)](https://github.com/yaa110/rubigo/blob/master/LICENSE) [![Version](https://img.shields.io/badge/version-1.0.4-blue.svg)](https://github.com/yaa110/rubigo/releases)

**Rubigo** is a *DEPRECATED* dependency tool and package manager for [Golang](https://golang.org/), written in [Rust](https://www.rust-lang.org/en-US/). Rubigo uses `vendor` directory (starting from Go 1.5) to install packages, however it is possible to add packages globally (in `GOPATH/src` directory) or make a local package in `vendor` directory. Rubigo respects to manual changes in `vendor` directory and does not delete custom packages. Currently, Rubigo only supports `git` repositories. This source code is licensed under MIT license that can be found in the LICENSE file.

## Deprecation
Consider using [Go versioned modules](https://github.com/golang/go/wiki/Modules):

- `rm -r vendor rubigo.json rubigo.lock`
- `export GO111MODULE=on`
- `go mod init`

## Features
- Manage `vendor`, `global` and `local` packages
- Use a custom repository to clone a package
- Support [semantic versioning](http://semver.org/)
- Define package information
- Start a new project (binary or library)

## How it works
Rubigo creates two JSON (manifest) files (`rubigo.json` and `rubigo.lock`) inside the directory of Golang project. The `rubigo.json` contains the information of the project and packages which should be installed and maintained, and `rubigo.lock` contains the information of packages which have already been installed in `vendor` directory or globally in `GOPATH/src`. You could edit both files manually or using Rubigo sub-commands, then you can apply them to project's dependencies. Also, it is feasible to start Rubigo in an existing project.

## How to install
You can download a pre-built binary from [releases](https://github.com/yaa110/rubigo/releases) page or you can build it manually as following:
1. Install [Rust](https://www.rust-lang.org/en-US/) programming language.
  * On Linux and Mac OS: install `cmake`, `libcurl4-openssl-dev`, `libelf-dev`, `libssl-dev` and `libdw-dev`.
  * On Windows: install `cmake`, `Visual Studio C++`.
2. Use Rust's package manager `cargo` to install the application: `cargo install --git https://github.com/yaa110/rubigo.git`

## Sub-commands
- **init, start**: Initializes Rubigo project in an existing directory, e.g. `rubigo init`. This sub-command searches the `vendor` directory for packages which has already been installed.
- **new, create**: Creates a new Golang project, e.g. `rubigo new my-project` or `rubigo new --lib my-library`. This sub-command creates a new directory with the name provided to it containing a new `.go` file and manifest files.
- **get, add**: Adds a package to dependencies and clones it into `vendor` directory, e.g. `rubigo get github.com/blah/blah --repo=github.com/my/custom/repo` (the `--repo` argument is optional). This sub-command could also install packages globally to `GOPATH/src` directory using `--global` flag or create a local package using `--local` flag.
- **update, up**: Updates one or all packages and applies the changes of `rubigo.json` to `rubigo.lock` and packages in `vendor` directory, e.g. `rubigo update github.com/blah/blah`. This sub-command could also delete the package's directory and clone it again using `--clean` flag. If no package name is provided, it updates all the packages.
- **remove, rm**: Removes a package from manifest files and `vendor` directory, e.g. `rubigo remove github.com/blah/blah`.
- **apply, install**: Applies the changes of `rubigo.lock` to packages in `vendor` directory, e.g. `rubigo apply`. This sub-command could also delete the package's directory and clone it again using `--clean` flag. Most of the time, it is used when you have cloned a project and wanted to install missing packages.
- **reset, sync**: Updates manifest files to the list of packages which have already been installed in `vendor` directory, e.g. `rubigo reset`. It is used when you have manually changed the `vendor` directory and wanted to update manifest files. Please note that this subcommand only collects git packages and ignores local packages.
- **list, ls**: Displays a list of packages from `rubigo.lock` file, e.g. `rubigo list`. This sub-command could only list git, local or global packages (or a combination of them) using `--remote`, `--local` or `--global` flags, respectively.
- **info, about**: Displays the information about the project from `rubigo.json` file, e.g. `rubigo info`.
- **help**: Displays the help message, e.g. `rubigo help`. It is also possible to get the information of a sub-command, e.g. `rubigo help get`.

## Flags
- **--verbose, -v**: Uses verbose output.
- **--quiet, -q**: Prints no output.
- **--yes, -y**: Continues without prompt for a confirmation.
- **--help, -h**: Displays the help message.
- **--version, -V**: Displays the version of Rubigo.

## The manifest format
You can find the template of [rubigo.json](https://github.com/yaa110/rubigo/blob/master/templates/rubigo.json) and [rubigo.lock](https://github.com/yaa110/rubigo/blob/master/templates/rubigo.lock) files in `templates` directory. Both files have a JSON format with the following objects:

- **info**: Contains the (optional) information about the project. Only `rubigo.json` contains this object.
  * **name**: The name of project
  * **import**: The import path of project
  * **description**: Short description about the project
  * **homepage**: Url to the project homepage (should contain the protocol scheme, such as `http://`)
  * **license**: The license of the project
  * **authors**: An array of project's authors
    * **name**: The name of author
    * **email**: The email address of author
    * **website**: The website url of author (should contain the protocol scheme, such as `http://`)
- **packages**: Containg the information about packages.
  * **git**: An array of dependencies cloned from a git repository
    * **import**: The import path of package
    * **repo**: A custom url to clone the repository
    * **version**: The version (a git revision or semantic version) of the project. For more information about the semantic rules, please check [semver](https://github.com/steveklabnik/semver) documentation.
  * **local**: An array of local packages in `vendor` directory.
  * **global**: An array of global packages in `GOPATH/src` directory.

## Contribution
Please feel free to open an issue to report a bug or ask a question, or open a pull request to debug or add more features to Rubigo.
