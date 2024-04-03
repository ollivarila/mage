use anyhow::{anyhow, ensure, Context, Result};
use regex::Regex;
use std::{
    fs::{self},
    path::{Path, PathBuf},
    str::FromStr,
};
use toml::Table;
use tracing::{debug, debug_span};

use crate::util::FullPath;

/// Represents one program-config in the dotfiles directory, that can be configured by mage.
#[derive(Debug, Clone)]
pub struct ProgramOptions {
    /// Path of the config file or folder located in dotfiles also the key in magefile
    pub origin_path: FullPath,
    /// Target path for symlink
    pub target_path: FullPath,
    // TODO: Force flag
}

impl ProgramOptions {
    pub fn generate(magefile: Table, base_path: FullPath) -> Result<Vec<ProgramOptions>> {
        let span = debug_span!("read_config");
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
            let full_origin_path = get_full_origin_path(&base_path.as_ref(), &origin_path);
            let full_target_path = FullPath::from(target_path);

            let opts = ProgramOptions {
                origin_path: full_origin_path,
                target_path: full_target_path,
            };
            result.push(opts)
        }

        Ok(result)
    }
}

#[derive(PartialEq, Debug)]
pub enum DotfilesOrigin {
    Directory(FullPath),
    Repository(String, FullPath),
}

fn magefile(path: PathBuf) -> anyhow::Result<Table> {
    let magefile = fs::read_to_string(path)?;
    let thing: Table =
        toml::from_str(&magefile).map_err(|e| anyhow!("Failed to parse magefile:\n{e}"))?;

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

    Err(anyhow!("Magefile not found"))
}

pub(crate) fn ensure_repo_is_setup(origin: DotfilesOrigin) -> anyhow::Result<FullPath> {
    match origin {
        DotfilesOrigin::Repository(url, path) => {
            let target_path = FullPath::from(path);
            if target_path.as_path().exists() {
                return Ok(target_path);
            }

            clone_repo(&url, target_path.to_str())?;

            return Ok(target_path);
        }
        DotfilesOrigin::Directory(dir) => Ok(dir),
    }
}

pub(crate) fn clone_repo<'a>(url: &str, path: &'a str) -> anyhow::Result<&'a str> {
    let p = FullPath::from(path);
    ensure!(
        !p.as_ref().exists(),
        "Target path {:?} already exists",
        path
    );

    debug!(url = url, "cloning repo");

    let success = std::process::Command::new("git")
        .args(["clone", "--recurse-submodules", url, path])
        .status()
        .map(|s| s.success())?;

    ensure!(success, "Failed to clone repository {url}");

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
            dir if is_dir(dir) => Ok(DotfilesOrigin::Directory(dir.into())),
            url if is_valid_repo_url(url) => Ok(DotfilesOrigin::Repository(
                url.to_string(),
                default_location.into(),
            )),
            url if is_github_repo(url) => Ok(DotfilesOrigin::Repository(
                full_repo_url(url),
                default_location.into(),
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
    let t: FullPath = s.into();
    t.as_ref().exists()
}

fn get_full_origin_path(base_path: &Path, path_in_magefile: &str) -> FullPath {
    let path = base_path.join(path_in_magefile);
    FullPath::from(path)
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    #[should_panic]
    fn invalid_magefile() {
        let path = PathBuf::from("examples/test-dotfiles/example.config");
        magefile(path).unwrap();
    }

    #[test]
    #[should_panic]
    fn does_not_clone_if_path_exists() {
        clone_repo("empty", "examples/test-dotfiles").unwrap();
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

        assert_eq!(df_origin, DotfilesOrigin::Directory("/tmp".into()));

        let df_origin: DotfilesOrigin = "https://github.com/test/repo.git".parse().unwrap();
        let should_be = DotfilesOrigin::Repository(
            "https://github.com/test/repo.git".to_string(),
            "~/.mage".into(),
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

    #[test]
    fn test_dotfiles_origin_github_url() {
        let origin: DotfilesOrigin = "test/test-repo".parse().unwrap();
        let should_be = DotfilesOrigin::Repository(
            "git@github.com:test/test-repo.git".into(),
            "~/.mage".into(),
        );
        assert_eq!(origin, should_be);
    }

    #[test]
    fn repo_is_setup_when_path_exists() {
        let path = "examples/test-dotfiles";
        let origin = DotfilesOrigin::Repository("empty".to_string(), path.into());
        let result = ensure_repo_is_setup(origin).unwrap();
        assert_eq!(result, path.into());
    }

    #[test]
    #[should_panic]
    fn ensure_repo_is_setup_errors_when_repo_does_not_exist() {
        let path = "/tmp/something".to_string();
        let origin = DotfilesOrigin::Repository("empty".into(), path.into());
        ensure_repo_is_setup(origin).unwrap();
    }
}
