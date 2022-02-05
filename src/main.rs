use git2::build::RepoBuilder;
use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::path::Path;

/// This is what we're going to decode into. Each field is optional, meaning
/// that it doesn't have to be present in TOML.

#[derive(Debug, Deserialize)]
struct Config {
    package: Option<PackageConfig>,
    dependencies: Option<HashMap<String, DepType>>,
}

#[derive(Debug, Deserialize)]
struct PackageConfig {
    name: Option<String>,
    version: Option<String>,
    authors: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DepType {
    TableGit {
        git: String,
        version: Option<String>,
    },
    TablePath {
        path: String,
        version: Option<String>,
    },
    Version(String),
}

fn build_fo() -> FetchOptions<'static> {
    let mut cb = RemoteCallbacks::new();
    cb.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            std::path::Path::new(&format!("{}/.ssh/id_rsa", env::var("HOME").unwrap())),
            None,
        )
    });
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);
    fo
}

fn clone(url: &str, path: &Path, name: &str) -> Result<Repository, git2::Error> {
    let mut builder = RepoBuilder::new();
    builder.fetch_options(build_fo());
    println!("Cloning UI dep: {}", name);
    builder.clone(url, path)
}

fn update<'a>(name: &str) -> Result<&'a str, git2::Error> {
    let repo = git2::Repository::open(format!("ui/deps/{}", name)).unwrap();
    repo.find_remote("origin")
        .unwrap()
        .fetch(&["main"], Some(&mut build_fo()), None)
        .unwrap();
    repo.find_remote("origin").unwrap().get_refspec(1);
    repo.find_reference(name);
    dbg!("Fetch Complete");
    Ok("Fetch Complete")
}

/// Handle a Dependency
/// Match on type - Table Git, Table Path, Version
/// If the folder of the name doesn't exist clone it and check out the optional version
/// else:
///     check status with listed remote repo. If version provided check version. If no version
///     provided check commit hash of head
///     if they don't match git fetch and git checkout
///
///
fn handle_dep(dep: (String, DepType)) -> String {
    let (package_name, dep_type) = dep;
    let path_to_dep = format!("ui/deps/{}", package_name);
    match dep_type {
        DepType::TableGit { git, version } => {
            if !Path::new(&path_to_dep).is_dir() {
                let re = Regex::new(r".com/").unwrap();
                let git = re.replace_all(&git, r".com:");
                let re = Regex::new(r"https?://").unwrap();
                let git = re.replace(&git, "");
                if version.is_some() {
                    let _ = 5 + 5;
                }
                if let Err(e) = clone(
                    &format!("git@{}", git),
                    Path::new(&path_to_dep),
                    &package_name,
                ) {
                    // If there is an error when cloning
                    panic!("{}", e);
                };
                // if let Some(version) = version {};
            } else {
                // If the folder already exists
                if let Err(e) = update(&package_name) {
                    panic!("{}", e);
                }
            }
        }
        DepType::TablePath { path, version } => {
            dbg!(path, version);
        }
        DepType::Version(version) => {
            dbg!(package_name, version);
        }
    }
    // if !Path::new(&path_to_dep).is_dir() {
    // if let Some(git) = &dep_type.git {
    //     let re = Regex::new(r".com/").unwrap();
    //     let git = re.replace_all(git, r".com:");
    //     if let Err(e) = clone(&format!("git@{}", git), Path::new(&path_to_dep)) {
    //         println!("{}", e);
    // }

    String::from("this")
}

fn main() {
    let decoded: Config = toml::from_str(&std::fs::read_to_string("ui.toml").unwrap()).unwrap();
    let deps = decoded.dependencies.unwrap();
    for dep in deps {
        handle_dep(dep);
    }
}
