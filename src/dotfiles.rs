use anyhow::{anyhow, bail, Context, Ok, Result};
use regex::Regex;
use std::{
    fs::{self},
    path::PathBuf,
    str::FromStr,
};
use toml::Table;
use tracing::{debug, debug_span};

use crate::util::get_full_path;

/// Represents one program-config in the dotfiles directory, that can be configured by mage.
#[derive(Debug, Clone)]
pub struct ProgramOptions {
    /// Path of the config file or folder located in dotfiles also the key in magefile
    pub origin_path: PathBuf,
    /// Target path for symlink
    pub target_path: PathBuf,
}

#[derive(PartialEq, Debug)]
pub enum DotfilesOrigin {
    Directory(PathBuf),
    Repository(String, String),
}

fn magefile(path: PathBuf) -> anyhow::Result<Table> {
    let magefile = fs::read_to_string(path)?;
    let thing: Table = toml::from_str(&magefile).map_err(|e| {
        let msg = format!("Failed to parse magefile:\n{}", e);
        anyhow!(msg)
    })?;

    Ok(thing)
}

pub(crate) fn find_magefile<P: Into<PathBuf>>(path: P) -> anyhow::Result<Table> {
    let dir = fs::read_dir(path.into())?;
    for entry in dir {
        let entry = entry?;
        let filename = entry
            .file_name()
            .to_str()
            .expect("should be able to convert dir entry")
            .to_string();

        if filename.starts_with("magefile") {
            return Ok(magefile(entry.path())?);
        }
    }

    Err(anyhow::anyhow!("Magefile not found"))
}

pub(crate) fn ensure_repo_is_setup(origin: DotfilesOrigin) -> anyhow::Result<PathBuf> {
    match origin {
        DotfilesOrigin::Repository(url, path) => {
            let target_path = PathBuf::from(path);
            if target_path.exists() {
                return Ok(target_path);
            }

            clone_repo(&url, target_path.to_str().expect("should not fail"))?;

            return Ok(target_path);
        }
        DotfilesOrigin::Directory(dir) => Ok(dir),
    }
}

pub(crate) fn clone_repo<'a>(url: &str, path: &'a str) -> anyhow::Result<&'a str> {
    if PathBuf::from(path).exists() {
        bail!("Target path {path} already exists")
    }

    debug!(url = url, "cloning repo");

    let success = std::process::Command::new("git")
        .args(["clone", url, path])
        .status()
        .map(|s| s.success())?;

    if !success {
        bail!("Failed to clone repository {url}")
    }

    debug!("done");

    Ok(path)
}

impl FromStr for DotfilesOrigin {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        let span = debug_span!("parse DotfilesOrigin", from = s);
        let _guard = span.enter();

        let default_location = "~/.mage".to_string();
        let result = match s {
            dir if is_dir(dir) => Ok(DotfilesOrigin::Directory(get_full_path(dir))),
            url if is_valid_repo_url(url) => Ok(DotfilesOrigin::Repository(
                url.to_string(),
                default_location,
            )),
            url if is_github_repo(url) => Ok(DotfilesOrigin::Repository(
                full_repo_url(url),
                default_location,
            )),
            _ => Err(anyhow!("This url seems to be invalid: {s}")),
        };

        debug!(got = ?result);

        result
    }
}

fn is_valid_repo_url(s: &str) -> bool {
    let regexes = vec![
        Regex::new(r"git@github.com:[A-z-\d]+\/[A-z-\d_]+.git").unwrap(),
        Regex::new(r"https://github.com/[A-z-\d]+\/[A-z-\d_]+.git").unwrap(),
    ];
    return regexes
        .iter()
        .map(|r| r.find(s).is_some_and(|m| m.len() == s.len()))
        .any(|v| v);
}

fn is_github_repo(s: &str) -> bool {
    if is_valid_repo_url(s) {
        return true;
    }

    let path = PathBuf::from(s);

    if path.exists() {
        return false;
    }

    let repo_re = Regex::new(r"[A-z-\d]+\/[A-z-\d_]+").expect("should be able to construct regex");

    repo_re.find(s).is_some_and(|m| m.len() == s.len())
}

fn full_repo_url(s: &str) -> String {
    format!("git@github.com:{s}.git") // Assume ssh
}

fn is_dir(s: &str) -> bool {
    get_full_path(s).is_dir()
}

pub fn generate_options(magefile: Table, base_path: &str) -> Result<Vec<ProgramOptions>> {
    let span = debug_span!("read_dotfiles");
    let _guard = span.enter();

    let keys = magefile.keys();
    let mut result = vec![];

    for origin_path in keys {
        let item = magefile.get(origin_path).expect("should always get value");
        let target_path = item
            .get("target_path")
            .context(format!("target_path not found in {item}"))?
            .as_str()
            .expect("should be able to convert to str")
            .to_string();
        let full_origin_path = get_full_origin_path(base_path, &origin_path);
        let full_target_path = get_full_path(target_path);

        let opts = ProgramOptions {
            origin_path: full_origin_path,
            target_path: full_target_path,
        };
        result.push(opts)
    }

    Ok(result)
}

fn get_full_origin_path(base_path: &str, magefile_origin: &str) -> PathBuf {
    let mut path = PathBuf::from(base_path);
    path.push(magefile_origin);
    get_full_path(path)
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn invalid_magefile() {
        let path = PathBuf::from("examples/test-dotfiles/example.config");
        let magefile = magefile(path);
        assert!(magefile.is_err());
    }

    #[test]
    fn does_not_clone_if_path_exists() {
        let result = clone_repo("empty", "examples/test-dotfiles");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert_eq!(msg, "Target path examples/test-dotfiles already exists");
    }

    #[test]
    #[ignore = "clones repo and requires internet connection"]
    fn test_clone_repo() {
        fs::remove_dir_all("/tmp/mage").unwrap_or_default();
        clone_repo("https://github.com/ollivarila/brainfckr.git", "/tmp/mage").unwrap();
        let dir_exists = PathBuf::from("/tmp/mage").exists();
        assert!(dir_exists);
        fs::remove_dir_all("/tmp/mage").unwrap_or_default();
    }

    #[test]
    fn dotfiles_origin_from_str() {
        let df_origin: DotfilesOrigin = "/tmp".parse().unwrap();

        assert_eq!(df_origin, DotfilesOrigin::Directory(PathBuf::from("/tmp")));

        let df_origin: DotfilesOrigin = "https://github.com/test/repo.git".parse().unwrap();
        let should_be = DotfilesOrigin::Repository(
            "https://github.com/test/repo.git".to_string(),
            "~/.mage".to_string(),
        );
        assert_eq!(df_origin, should_be);

        let res: Result<DotfilesOrigin, anyhow::Error> = "asdf".parse();
        assert!(res.is_err())
    }

    struct Context {
        path: String,
    }

    impl Drop for Context {
        fn drop(&mut self) {
            fs::remove_dir_all(self.path.clone()).unwrap_or_default();
        }
    }

    #[allow(unused)]
    fn setup() -> Context {
        let path = "/tmp/mage".to_string();
        if fs::read_dir(&path).is_ok() {
            fs::remove_dir_all(&path).unwrap_or_default();
        }
        Context { path }
    }

    #[test]
    fn test_is_not_github_repo() {
        assert!(!is_github_repo("/tmp/test"));
        assert!(is_github_repo("test/test"));
        assert!(!is_github_repo("git@something"));
        assert!(is_github_repo("git@github.com:test/test-repo.git"));
        assert!(is_github_repo("https://github.com/test/test-repo.git"));
    }
}
