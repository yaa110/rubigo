use std::fs::File;
use std::io::Write;
use json::{self, JsonValue};
use std::path::Path;
use std::io::{self, Read};

pub const IMPORT_KEY: &'static str = "import";
pub const VERSION_KEY: &'static str = "version";
pub const REPO_KEY: &'static str = "repo";

pub const INFO_KEY: &'static str = "info";
pub const NAME_KEY: &'static str = "name";
pub const LICENSE_KEY: &'static str = "license";
pub const HOMEPAGE_KEY: &'static str = "homepage";
pub const AUTHORS_KEY: &'static str = "authors";
pub const DESCRIPTION_KEY: &'static str = "description";
pub const WEBSITE_KEY: &'static str = "website";
pub const EMAIL_KEY: &'static str = "email";

pub const PACKAGES_KEY: &'static str = "packages";
pub const GIT_KEY: &'static str = "git";
pub const LOCAL_KEY: &'static str = "local";
pub const GLOBAL_KEY: &'static str = "global";

pub fn write<P: AsRef<Path>>(json_path: P, project_name: &str, data: Option<JsonValue>) -> io::Result<()> {
    match File::create(json_path) {
        Ok(mut file) => {
            match file.write_all(format!("{:#}", if data.is_none() {
                object!{
                    INFO_KEY => object!{
                        NAME_KEY => project_name
                    },
                    PACKAGES_KEY => object!{
                        GIT_KEY => array![],
                        LOCAL_KEY => array![],
                        GLOBAL_KEY => array![]
                    }
                }
            } else {
                data.unwrap()
            }).as_bytes()) {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
        },
        Err(e) => return Err(e),
    }

    Ok(())
}

pub fn read(json_path: &Path) -> io::Result<JsonValue> {
    let mut file = File::open(json_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    match json::parse(contents.as_str()) {
        Ok(content) => Ok(content),
        Err(_) => Err(io::Error::new(io::ErrorKind::Other, format!("unable to parse json file: {:?}", json_path.to_str().unwrap_or("unknown")).as_str())),
    }
}

pub fn remove_package_from_array(pkg_import: &str, json_array: &JsonValue, is_local: bool) -> JsonValue {
    let mut result_array = json_array.clone();
    for i in 0..json_array.len() {
        let pkg_name = if is_local {
            match json_array[i].as_str() {
                Some(name) => name,
                None => continue,
            }
        } else {
            match json_array[i][IMPORT_KEY].as_str() {
                Some(name) => name,
                None => continue,
            }
        };
        if pkg_import == pkg_name {
            let _ = result_array.array_remove(i);
            break
        }
    }
    result_array
}
