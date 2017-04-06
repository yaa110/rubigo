use json::JsonValue;

#[derive(PartialEq, Eq, Debug)]
pub enum PackageUpdate {
    Fixed,
    Minor,
    Patch,
    Latest,
}

pub struct Package {
    pub import: String,
    pub update: Option<PackageUpdate>,
    pub version: Option<String>,
    pub repo: Option<String>,
}

impl Package {
    pub fn new() -> Self {
        Package {
            import: String::new(),
            update: None,
            version: None,
            repo: None,
        }
    }

    pub fn to_json(&self) -> JsonValue {
        // TODO add self.repo
        match self.version {
            Some(ref ver) => {
                object!{
                    "import" => self.import.as_str(),
                    "update" => match self.update {
                        Some(ref up) => match up {
                            &PackageUpdate::Fixed => "fixed",
                            &PackageUpdate::Minor => "minor",
                            &PackageUpdate::Patch => "patch",
                            &PackageUpdate::Latest => "latest",
                        },
                        None => "fixed"
                    },
                    "version" => ver.as_str()
                }
            },
            None => {
                object!{
                    "import" => self.import.as_str()
                }
            },
        }
    }
}
