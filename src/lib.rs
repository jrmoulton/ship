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
    _package: Option<PackageConfig>,
    dependencies: Option<HashMap<String, DepType>>,
}

#[derive(Debug, Deserialize)]
struct PackageConfig {
    _name: Option<String>,
    _version: Option<String>,
    _authors: Option<Vec<String>>,
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
//
fn build_fo() -> FetchOptions<'static> {
    let cb = build_cb();
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);
    fo
}

fn build_cb() -> RemoteCallbacks<'static> {
    let mut cb = RemoteCallbacks::new();
    cb.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            std::path::Path::new(&format!("{}/.ssh/id_rsa", env::var("HOME").unwrap())),
            None,
        )
    });
    cb
}

fn clone(url: &str, path: &Path, name: &str) -> Result<Repository, git2::Error> {
    let mut builder = RepoBuilder::new();
    builder.fetch_options(build_fo());
    println!("Cloning UI dep: {}", name);
    builder.clone(url, path)
}

fn update<'a>(name: &str, version: Option<String>) -> Result<&'a str, git2::Error> {
    let repo = git2::Repository::open(format!("ui/deps/{}", name))?;
    let mut remote = repo.find_remote("origin")?;
    let cb = build_cb();
    remote.connect_auth(git2::Direction::Fetch, Some(cb), None)?;
    let remote_branch = remote.default_branch()?;
    remote.fetch(
        &[remote_branch.as_str().unwrap()],
        Some(&mut build_fo()),
        None,
    )?;

    match version {
        // To speed things up here if the references match just return
        Some(version) => {
            // Set head detatched(annotatedComit)
            let commit = repo.revparse_single(&version)?;
            repo.checkout_tree(
                &commit,
                Some(git2::build::CheckoutBuilder::new().update_index(true)),
            )?;
        }
        None => {
            let remote_branch_string = remote_branch.as_str().unwrap().clone();
            let re = Regex::new(r"[^/]+$").unwrap();
            let remote_branch_string = re.captures(remote_branch_string).unwrap();
            repo.set_head(&format!(
                "refs/remotes/origin/{}",
                remote_branch_string[0].to_owned()
            ))?;
            repo.checkout_head(Some(
                git2::build::CheckoutBuilder::new()
                    .update_index(true)
                    .force(),
            ))?;
        }
    }
    repo.checkout_index(None, Some(git2::build::CheckoutBuilder::new().force()))?;
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
                clone(
                    &format!("git@{}", git),
                    Path::new(&path_to_dep),
                    &package_name,
                )
                .unwrap();
            } else {
                // If the folder already exists
                update(&package_name, version).unwrap();
            }
        }
        DepType::TablePath { path, version } => {
            if !Path::new(&path_to_dep).is_dir() {
                let path_to_dep = "ui/deps/";
                std::fs::create_dir_all(&path_to_dep).unwrap();
                fs_extra::dir::copy(path, path_to_dep, &fs_extra::dir::CopyOptions::new()).unwrap();
                if version.is_some() {
                    update(&package_name, version).unwrap();
                }
            } else {
                // Here I need to check a .lock file to check that the last copy option was as a path
                // not a git.
                let mut copy_options = fs_extra::dir::CopyOptions::new();
                copy_options.overwrite = true;
                fs_extra::dir::copy(path, path_to_dep, &copy_options).unwrap();
                if version.is_some() {
                    update(&package_name, version).unwrap();
                }
            }
        }
        DepType::Version(version) => {
            dbg!(package_name, version);
        }
    }
    String::from("this")
}

pub fn run() {
    let decoded: Config = toml::from_str(&std::fs::read_to_string("ui.toml").unwrap()).unwrap();
    let deps = decoded.dependencies.unwrap();
    for dep in deps {
        handle_dep(dep);
    }
}
