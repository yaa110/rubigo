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
