use json::JsonValue;

#[derive(PartialEq, Eq, Debug)]
enum PackageUpdate {
    Fixed,
    Minor,
    Patch,
    Latest,
}

struct Package {
    import: &'static str,
    update: Option<PackageUpdate>,
    version: Option<&'static str>,
}

impl Package {
    pub fn to_json(&self) -> JsonValue {
        match self.version {
            Some(ver) => {
                object!{
                    "import" => self.import,
                    "update" => match self.update {
                        Some(ref up) => match up {
                            &PackageUpdate::Fixed => "fixed",
                            &PackageUpdate::Minor => "minor",
                            &PackageUpdate::Patch => "patch",
                            &PackageUpdate::Latest => "latest",
                        },
                        None => "fixed"
                    },
                    "version" => ver
                }
            },
            None => {
                object!{
                    "import" => self.import
                }
            },
        }
    }
}
