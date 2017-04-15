extern crate tempdir;

use controller::*;
use self::tempdir::TempDir;
use inner::logger::{Logger, Verbosity};
use std::env;
use inner::json_helper;
use std::fs::{File, remove_dir_all, remove_file};
use std::io::Read;

#[test]
fn test_main() {
    // Note: Due to setting current working directory for each test, they must not run parallel.

    println!("running test_new_bin:");
    test_new_bin();

    println!("\nrunning test_new_lib:");
    test_new_lib();

    println!("\nrunning test_init:");
    test_init();

    println!("\nrunning test_get_git:");
    test_get_git();

    println!("\nrunning test_get_git_repo:");
    test_get_git_repo();

    println!("\nrunning test_get_local:");
    test_get_local();

    println!("\nrunning test_apply:");
    test_apply();

    println!("\nrunning test_reset:");
    test_reset();

    println!("\nrunning test_remove:");
    test_remove();

    println!("\nrunning test_update_one:");
    test_update_one();

    println!("\nrunning test_update_all:");
    test_update_all();
}

fn test_new_bin() {
    let project_name = "test-bin-project";

    let tmp_dir = TempDir::new("rubigo-new-bin").unwrap();
    env::set_current_dir(tmp_dir.path()).unwrap();

    let logger = Logger::new(Verbosity::High);

    project::new(project_name, false, &logger);
    let project_path_buf = tmp_dir.path().join(project_name);
    let project_path = project_path_buf.as_path();

    let json_content = json_helper::read(project_path.join("rubigo.json").as_path()).unwrap();
    let project_name_json = json_content[json_helper::INFO_KEY][json_helper::NAME_KEY].as_str().unwrap();
    assert_eq!(project_name, project_name_json);

    let mut file = File::open(project_path.join("main.go").as_path()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents.as_str(), "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Println(\"Hello, World!\")\n}\n\n");
}

fn test_new_lib() {
    let project_name = "test-lib-project";

    let tmp_dir = TempDir::new("rubigo-new-lib").unwrap();
    env::set_current_dir(tmp_dir.path()).unwrap();

    let logger = Logger::new(Verbosity::High);

    project::new(project_name, true, &logger);
    let project_path_buf = tmp_dir.path().join(project_name);
    let project_path = project_path_buf.as_path();

    let json_content = json_helper::read(project_path.join("rubigo.json").as_path()).unwrap();
    let project_name_json = json_content[json_helper::INFO_KEY][json_helper::NAME_KEY].as_str().unwrap();
    assert_eq!(project_name, project_name_json);

    let mut file = File::open(project_path.join(format!("{}.go", project_name)).as_path()).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents.as_str(), format!("package {}\n\n", project_name).as_str());
}

fn test_init() {
    let tmp_dir = TempDir::new("rubigo-init").unwrap();
    env::set_current_dir(tmp_dir.path()).unwrap();

    let logger = Logger::new(Verbosity::High);

    project::init(logger);

    let json_content = json_helper::read(tmp_dir.path().join("rubigo.json").as_path()).unwrap();
    let project_name_json = json_content[json_helper::INFO_KEY][json_helper::NAME_KEY].as_str().unwrap();
    assert_eq!(tmp_dir.path().file_name().unwrap().to_str().unwrap(), project_name_json);
}

fn test_get_git() {
    let tmp_dir = TempDir::new("rubigo-get-git").unwrap();
    env::set_current_dir(tmp_dir.path()).unwrap();

    let logger = Logger::new(Verbosity::High);

    project::init(logger);
    package::get("github.com/yaa110/test-repo-for-rubigo", None, true, false, false, logger);

    let mut file = File::open(tmp_dir.path().join("vendor").as_path().join("github.com").as_path().join("yaa110").as_path().join("test-repo-for-rubigo").as_path().join("file-to-read")).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents.as_str(), "rubigo\n");

    let json_content = json_helper::read(tmp_dir.path().join("rubigo.json").as_path()).unwrap();
    let import_json = json_content[json_helper::PACKAGES_KEY][json_helper::GIT_KEY][0][json_helper::IMPORT_KEY].as_str().unwrap();
    assert_eq!("github.com/yaa110/test-repo-for-rubigo", import_json);

    let lock_content = json_helper::read(tmp_dir.path().join("rubigo.lock").as_path()).unwrap();
    let import_lock = lock_content[json_helper::GIT_KEY][0][json_helper::IMPORT_KEY].as_str().unwrap();
    assert_eq!("github.com/yaa110/test-repo-for-rubigo", import_lock);
}

fn test_get_git_repo() {
    let tmp_dir = TempDir::new("rubigo-get-git").unwrap();
    env::set_current_dir(tmp_dir.path()).unwrap();

    let logger = Logger::new(Verbosity::High);

    project::init(logger);
    package::get("a/b/c", Some("https://github.com/yaa110/test-repo-for-rubigo"), true, false, false, logger);

    let mut file = File::open(tmp_dir.path().join("vendor").as_path().join("a").as_path().join("b").as_path().join("c").as_path().join("file-to-read")).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents.as_str(), "rubigo\n");

    let json_content = json_helper::read(tmp_dir.path().join("rubigo.json").as_path()).unwrap();
    let repo_json = json_content[json_helper::PACKAGES_KEY][json_helper::GIT_KEY][0][json_helper::REPO_KEY].as_str().unwrap();
    assert_eq!("https://github.com/yaa110/test-repo-for-rubigo", repo_json);

    let lock_content = json_helper::read(tmp_dir.path().join("rubigo.lock").as_path()).unwrap();
    let repo_lock = lock_content[json_helper::GIT_KEY][0][json_helper::REPO_KEY].as_str().unwrap();
    assert_eq!("https://github.com/yaa110/test-repo-for-rubigo", repo_lock);
}

fn test_get_local() {
    let tmp_dir = TempDir::new("rubigo-get-git").unwrap();
    env::set_current_dir(tmp_dir.path()).unwrap();

    let logger = Logger::new(Verbosity::High);

    project::init(logger);
    package::get("new-dir", None, true, false, true, logger);

    assert!(tmp_dir.path().join("vendor").as_path().join("new-dir").as_path().exists())
}

fn test_apply() {
    let tmp_dir = TempDir::new("rubigo-get-git").unwrap();
    env::set_current_dir(tmp_dir.path()).unwrap();

    let logger = Logger::new(Verbosity::High);

    project::init(logger);
    package::get("github.com/yaa110/test-repo-for-rubigo", None, true, false, false, logger);
    remove_dir_all(tmp_dir.path().join("vendor").as_path()).unwrap();

    project::apply(false, logger);

    let mut file = File::open(tmp_dir.path().join("vendor").as_path().join("github.com").as_path().join("yaa110").as_path().join("test-repo-for-rubigo").as_path().join("file-to-read")).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents.as_str(), "rubigo\n");
}

fn test_reset() {
    let tmp_dir = TempDir::new("rubigo-get-git").unwrap();
    env::set_current_dir(tmp_dir.path()).unwrap();

    let logger = Logger::new(Verbosity::High);

    project::init(logger);
    package::get("github.com/yaa110/test-repo-for-rubigo", None, true, false, false, logger);
    remove_file(tmp_dir.path().join("rubigo.json").as_path()).unwrap();
    remove_file(tmp_dir.path().join("rubigo.lock").as_path()).unwrap();

    project::reset(true, logger);

    let json_content = json_helper::read(tmp_dir.path().join("rubigo.json").as_path()).unwrap();
    let import_json = json_content[json_helper::PACKAGES_KEY][json_helper::GIT_KEY][0][json_helper::IMPORT_KEY].as_str().unwrap();
    assert_eq!("github.com/yaa110/test-repo-for-rubigo", import_json);

    let lock_content = json_helper::read(tmp_dir.path().join("rubigo.lock").as_path()).unwrap();
    let import_lock = lock_content[json_helper::GIT_KEY][0][json_helper::IMPORT_KEY].as_str().unwrap();
    assert_eq!("github.com/yaa110/test-repo-for-rubigo", import_lock);
}

fn test_remove() {
    let tmp_dir = TempDir::new("rubigo-get-git").unwrap();
    env::set_current_dir(tmp_dir.path()).unwrap();

    let logger = Logger::new(Verbosity::High);

    project::init(logger);
    package::get("github.com/yaa110/test-repo-for-rubigo", None, true, false, false, logger);

    package::remove("github.com/yaa110/test-repo-for-rubigo", logger);

    let json_content = json_helper::read(tmp_dir.path().join("rubigo.json").as_path()).unwrap();
    assert_eq!(json_content[json_helper::PACKAGES_KEY][json_helper::GIT_KEY].len(), 0);

    let lock_content = json_helper::read(tmp_dir.path().join("rubigo.lock").as_path()).unwrap();
    assert_eq!(lock_content[json_helper::GIT_KEY].len(), 0);

    assert!(!tmp_dir.path().join("vendor").as_path().exists())
}

fn test_update_one() {
    let tmp_dir = TempDir::new("rubigo-get-git").unwrap();
    env::set_current_dir(tmp_dir.path()).unwrap();

    let logger = Logger::new(Verbosity::High);

    project::init(logger);
    package::get("github.com/yaa110/test-repo-for-rubigo", None, true, false, false, logger);
    remove_dir_all(tmp_dir.path().join("vendor").as_path()).unwrap();

    package::update(Some("github.com/yaa110/test-repo-for-rubigo"), false, logger);

    let mut file = File::open(tmp_dir.path().join("vendor").as_path().join("github.com").as_path().join("yaa110").as_path().join("test-repo-for-rubigo").as_path().join("file-to-read")).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents.as_str(), "rubigo\n");
}

fn test_update_all() {
    let tmp_dir = TempDir::new("rubigo-get-git").unwrap();
    env::set_current_dir(tmp_dir.path()).unwrap();

    let logger = Logger::new(Verbosity::High);

    project::init(logger);
    package::get("github.com/yaa110/test-repo-for-rubigo", None, true, false, false, logger);
    remove_dir_all(tmp_dir.path().join("vendor").as_path()).unwrap();

    package::update(None, false, logger);

    let mut file = File::open(tmp_dir.path().join("vendor").as_path().join("github.com").as_path().join("yaa110").as_path().join("test-repo-for-rubigo").as_path().join("file-to-read")).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents.as_str(), "rubigo\n");
}
