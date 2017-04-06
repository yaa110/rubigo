use std::fs;
use std::path::{Component, Path};

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
