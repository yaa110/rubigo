use std::fs::File;
use std::io::Write;
use json::JsonValue;
use std::path::Path;
use std::io;

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
