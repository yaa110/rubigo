use std::fs;
use std::path::{Component, Path};
use std::io::{self, Write};

pub fn get_current_dir() -> String {
    match fs::canonicalize(Path::new(Component::CurDir.as_os_str())) {
        Ok(p_buf) => match p_buf.as_path().components().last() {
            Some(Component::Normal(name_os_str)) => match name_os_str.to_str() {
                Some(name_str) => name_str.to_string(),
                None => return "unknown".to_string(),
            },
            _ => return "unknown".to_string(),
        },
        _ => return "unknown".to_string(),
    }
}

pub fn get_input(msg: &str) -> io::Result<String> {
    let mut input = String::new();
    print!("{}", msg);
    let _ = io::stdout().flush();
    match io::stdin().read_line(&mut input) {
        Ok(_) => Ok(input),
        Err(e) => Err(e),
    }
}

pub fn confirmation_prompt(msg: &str) -> io::Result<bool> {
    match get_input(msg) {
        Ok(input) => {
            match input.to_lowercase().as_str().trim() {
                "y" | "yes" | "yea" | "yeah" | "yep" | "yup" => Ok(true),
                _ => Ok(false),
            }
        },
        Err(e) => Err(e),
    }
}
