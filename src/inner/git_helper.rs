use git2::{Repository, Object, BranchType};
use semver::{Version, VersionReq};
use regex::Regex;
use inner::logger::Logger;

pub fn get_latest_commit(repo: &Repository) -> Option<String> {
    match repo.head() {
        Ok(r) => match r.resolve() {
            Ok(ref r) => match r.target() {
                Some(id) => Some(format!("{}", id)),
                None => None,
            },
            _ => None,
        },
        _ => None,
    }
}

pub fn get_current_branch(repo: &Repository) -> Option<String> {
    match repo.branches(Some(BranchType::Local)) {
        Ok(branches) => {
            for b in branches {
                let branch = match b {
                    Ok(b) => b.0,
                    _ => continue,
                };
                if branch.is_head() {
                    match branch.name() {
                        Ok(name) => match name {
                            Some(name_str) => return Some(name_str.to_owned()),
                            None => return None,
                        },
                        _ => return None,
                    }
                }
            }
            None
        },
        _ => None,
    }
}

pub fn get_latest_version(repo: &Repository, version_rule: Option<&VersionReq>) -> Option<(String, Version)> {
    let mut version = None;
    match repo.tag_names(None) {
        Ok(tag_names) => {
            let mut selected_tag = None;
            let re = match Regex::new(r"^v?([0-9]+)[.]?([0-9]*)[.]?([0-9]*)([-]?.*)") {
                Ok(re) => re,
                _ => return version,
            };
            for t in tag_names.iter() {
                let tag_name = match t {
                    Some(name) => name,
                    None => continue,
                };
                let tag_version_str = match re.captures(t.unwrap()) {
                    Some(caps) => format!("{}.{}.{}{}",
                                          match caps.get(1) {
                                              Some(c) => {
                                                  let n = c.as_str();
                                                  if n.is_empty() {
                                                      continue
                                                  } else {
                                                      n
                                                  }
                                              },
                                              _ => continue,
                                          },
                                          match caps.get(2) {
                                              Some(c) => {
                                                  let n = c.as_str();
                                                  if n.is_empty() {
                                                      "0"
                                                  } else {
                                                      n
                                                  }
                                              },
                                              _ => "0",
                                          },
                                          match caps.get(3) {
                                              Some(c) => {
                                                  let n = c.as_str();
                                                  if n.is_empty() {
                                                      "0"
                                                  } else {
                                                      n
                                                  }
                                              },
                                              _ => "0",
                                          },
                                          match caps.get(4) {
                                              Some(c) => c.as_str(),
                                              _ => "",
                                          }),
                    None => continue,
                };
                let tag_version = match Version::parse(tag_version_str.as_str()) {
                    Ok(ver) => ver,
                    _ => continue,
                };
                if (version_rule.is_none() || version_rule.unwrap().matches(&tag_version)) && (selected_tag.is_none() || tag_version > selected_tag.clone().unwrap()) {
                    version = Some((tag_name.to_owned(), tag_version.clone()));
                    selected_tag = Some(tag_version);
                }
            }
        },
        _ => (),
    }
    version
}

pub fn get_latest_compat_version(repo: &Repository, rule_tag_name: String) -> String {
    match VersionReq::parse(rule_tag_name.as_str()) {
        Ok(version_rule) => match get_latest_version(repo, Some(&version_rule)) {
            Some((tag_name, _)) => tag_name,
            None => rule_tag_name,
        },
        _ => rule_tag_name,
    }
}

pub fn get_revision_object(repo: &Repository, pkg_import: String, version: String, should_retry: bool, logger: Logger) -> Option<(Object, String)> {
    match repo.revparse_single(version.as_str()) {
        Ok(obj) => return Some((obj, version)),
        Err(e) => {
            if !should_retry {
                return None
            }
            match get_latest_commit(repo) {
                Some(ver) => {
                    logger.error(format!("the version of `{}` changed to `{}` due to {}", pkg_import, ver, e));
                    return get_revision_object(repo, pkg_import, ver, false, logger)
                },
                None => return None,
            }
        },
    }
}
