use std::fs::File;
use std::io::Write;
use json::{self, JsonValue};
use std::path::Path;
use std::io::{self, Read};

pub fn write<P: AsRef<Path>>(json_path: P, project_name: &str, data: Option<JsonValue>) -> io::Result<()> {
    match File::create(json_path) {
        Ok(mut file) => {
            match file.write_all(format!("{:#}", if data.is_none() {
                object!{
                    "info" => object!{
                        "name" => project_name
                    },
                    "packages" => object!{
                        "git" => array![],
                        "local" => array![],
                        "global" => array![]
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
