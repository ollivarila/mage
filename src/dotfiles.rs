use anyhow::{anyhow, bail, Result};
use std::{
    fs::{self, read_dir, DirEntry, ReadDir},
    path::PathBuf,
};
use toml::Table;
use tracing::{debug, debug_span};

#[derive(Debug, Clone)]
pub struct ProgramOptions {
    pub name: String,
    pub path: PathBuf,
    pub target_path: PathBuf,
    pub is_installed_cmd: Option<String>,
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

        let Ok(path) = entry.path().canonicalize() else {
            bail!("failed to canonicalize path")
        };

        let key = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or(anyhow!(
                "failed to extract file stem as str for: {:?}",
                path
            ))?
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
            DotfilesOrigin::Directory(dir) => read_dir(dir).map_err(Into::into),
            DotfilesOrigin::Repository(url, path) => clone_repo_and_read(&url, &path),
        }
    }
}

fn clone_repo_and_read(url: &str, path: &str) -> anyhow::Result<ReadDir> {
    if PathBuf::from(path).exists() {
        bail!("Target path {path} already exists")
    }

    std::process::Command::new("git")
        .args(["clone", url, path])
        .output()?;

    debug!(path, "cloned repo");
    let res = fs::read_dir(path)?;

    debug!("read cloned repo");
    Ok(res)
}

impl TryFrom<(&str, &str)> for DotfilesOrigin {
    type Error = anyhow::Error;

    fn try_from((origin, clone_path): (&str, &str)) -> Result<Self, Self::Error> {
        let span = debug_span!("DotfilesOrigin", origin, target_clone_path = clone_path);
        let _guard = span.enter();

        let result = match origin {
            url if is_url(url) => Ok(DotfilesOrigin::Repository(
                url.to_string(),
                clone_path.to_string(),
            )),
            dir if is_dir(dir) => Ok(DotfilesOrigin::Directory(PathBuf::from(dir))),
            _ => Err(anyhow!("Invalid origin for dotfiles: {origin}")),
        };

        debug!(origin = ?result);

        result
    }
}

fn is_url(s: &str) -> bool {
    s.starts_with("https://") || s.starts_with("http://")
}

fn is_dir(s: &str) -> bool {
    let path = PathBuf::from(s);
    path.is_dir()
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
        let result = clone_repo_and_read("empty", "examples/test-dotfiles");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert_eq!(msg, "Target path examples/test-dotfiles already exists");
    }

    #[test]
    #[ignore]
    fn test_clone_repo() {
        fs::remove_dir_all("/tmp/mage").unwrap_or_default();
        clone_repo_and_read("https://github.com/ollivarila/brainfckr.git", "/tmp/mage").unwrap();
        let dir_exists = PathBuf::from("/tmp/mage").exists();
        assert!(dir_exists);
        fs::remove_dir_all("/tmp/mage").unwrap_or_default();
    }

    #[test]
    fn dotfiles_origin_from_str() {
        let origin = ("/tmp", "asdf");
        let df_origin: DotfilesOrigin = origin.try_into().unwrap();

        assert_eq!(df_origin, DotfilesOrigin::Directory(PathBuf::from("/tmp")));

        let origin = ("https://google.com", "bar");
        let df_origin: DotfilesOrigin = origin.try_into().unwrap();
        let should_be =
            DotfilesOrigin::Repository("https://google.com".to_string(), "bar".to_string());
        assert_eq!(df_origin, should_be);

        let origin = ("asdf", "bar");
        let df_origin: Result<DotfilesOrigin, _> = origin.try_into();
        assert!(df_origin.is_err())
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
}
