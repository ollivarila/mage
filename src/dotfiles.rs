use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use std::{
    fs::{self, read_dir, DirEntry, ReadDir},
    path::PathBuf,
    str::FromStr,
};
use toml::Table;
use tracing::{debug, debug_span};

/// Represents one program-config in the dotfiles directory, that can be configured by mage.
#[derive(Debug, Clone)]
pub struct ProgramOptions {
    /// Name of the key in magefile.toml
    pub name: String,
    /// Name of the config file or folder located in dotfiles
    pub path: PathBuf,
    /// Target path for symlink
    pub target_path: PathBuf,
}

#[derive(PartialEq, Debug)]
pub enum DotfilesEntry {
    ConfigFileOrDir(String, PathBuf),
    Magefile(Table),
}

#[derive(PartialEq, Debug)]
pub enum DotfilesOrigin {
    Directory(PathBuf),
    Repository(String, String),
}

impl TryFrom<DirEntry> for DotfilesEntry {
    type Error = anyhow::Error;

    fn try_from(entry: DirEntry) -> Result<Self, Self::Error> {
        let is_magefile = entry
            .file_name()
            .to_str()
            .expect("should be able to convert file name to str")
            .starts_with("magefile");

        if is_magefile {
            let magefile = magefile(entry.path())?;
            return Ok(DotfilesEntry::Magefile(magefile));
        }
        let path = entry.path();

        let key = path
            .file_name()
            .and_then(|s| s.to_str())
            .context("extract filename")?
            .to_string();

        debug!(name = ?key, path = ?path, "parsed");

        Ok(DotfilesEntry::ConfigFileOrDir(key, path))
    }
}

fn magefile(path: PathBuf) -> anyhow::Result<Table> {
    let magefile = fs::read_to_string(path)?;
    let thing: Table = toml::from_str(&magefile).map_err(|e| {
        let msg = format!("Failed to parse magefile:\n{}", e);
        anyhow!(msg)
    })?;

    Ok(thing)
}

impl TryFrom<DotfilesOrigin> for ReadDir {
    type Error = anyhow::Error;
    fn try_from(origin: DotfilesOrigin) -> Result<Self, Self::Error> {
        match origin {
            DotfilesOrigin::Directory(dir) => read_dir(dir).context("read dotfiles dir"),
            DotfilesOrigin::Repository(url, path) => clone_repo(&url, &path)
                .map(fs::read_dir)?
                .context("clone and read repository"),
        }
    }
}

pub(crate) fn clone_repo<'a>(url: &str, path: &'a str) -> anyhow::Result<&'a str> {
    if PathBuf::from(path).exists() {
        bail!("Target path {path} already exists")
    }

    debug!(url = url, "cloning repo");

    std::process::Command::new("git")
        .args(["clone", url, path])
        .output()?;

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
            dir if is_dir(dir) => Ok(DotfilesOrigin::Directory(PathBuf::from(dir))),
            url if is_valid_repo_url(url) => Ok(DotfilesOrigin::Repository(
                url.to_string(),
                default_location,
            )),
            url if is_github_repo(url) => Ok(DotfilesOrigin::Repository(
                full_repo_url(url),
                default_location,
            )),
            _ => Err(anyhow!("I do not how to get dotfiles with this: {s}")),
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
    format!("git@github.com/{s}.git") // Assume ssh
}

fn is_dir(s: &str) -> bool {
    let path = PathBuf::from(s);
    path.is_dir()
}

pub fn read_dotfiles(dotfiles: ReadDir) -> Result<Vec<ProgramOptions>> {
    let span = debug_span!("read_dotfiles");
    let _guard = span.enter();

    let mut mage_config: Option<Table> = None;
    let mut programs = vec![];

    for item in dotfiles {
        let item = item?;
        let filename = item.file_name();
        debug!(filename = ?filename, "entry");
        match item.try_into()? {
            DotfilesEntry::Magefile(magefile) => mage_config = Some(magefile),
            DotfilesEntry::ConfigFileOrDir(name, path) => programs.push((name, path)),
        }
    }
    let mage_config = mage_config.context("magefile not found")?;

    // Skip config files that are not in the magefile
    programs.retain(|(name, _)| {
        let retain = mage_config.contains_key(name);
        if !retain {
            debug!(name, "skipping");
        }
        retain
    });

    let mut result = vec![];

    for (name, path) in programs.into_iter() {
        let program = (&mage_config, name, path).try_into()?;
        result.push(program);
    }

    debug!("done");
    Ok(result)
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

    #[test]
    fn dotfiles_origin_into_readdir() {
        let origin = DotfilesOrigin::Directory(PathBuf::from("examples/test-dotfiles"));
        let readdir: ReadDir = origin.try_into().unwrap();

        let n = readdir.count();

        // another
        // example.config
        // magefile.toml
        assert_eq!(n, 3)
    }
    struct Context {
        path: String,
    }

    impl Drop for Context {
        fn drop(&mut self) {
            fs::remove_dir_all(self.path.clone()).unwrap_or_default();
        }
    }

    fn setup() -> Context {
        let path = "/tmp/mage".to_string();
        if fs::read_dir(&path).is_ok() {
            fs::remove_dir_all(&path).unwrap_or_default();
        }
        Context { path }
    }

    #[test]
    fn test_read_dotfiles() {
        let _ctx = setup();

        let dotfiles = fs::read_dir("examples/test-dotfiles").unwrap();
        let programs = read_dotfiles(dotfiles).unwrap();

        assert_eq!(programs.len(), 1);

        let program = &programs[0];
        assert_eq!(program.name, "example.config");
        let target_path = program.target_path.to_str().unwrap();

        assert_eq!(target_path, "/tmp/example.config")
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
