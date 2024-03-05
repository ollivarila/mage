use crate::Args;
use anyhow::{bail, Context, Ok, Result};
use std::{
    fs::{self, ReadDir},
    path::PathBuf,
};
use toml::Table;

#[derive(Debug)]
#[allow(dead_code)]
pub struct ProgramOptions {
    pub name: String,
    pub path: PathBuf,
    pub target_path: String,
    pub is_installed_cmd: Option<String>,
}

impl ProgramOptions {
    pub fn new(
        name: String,
        path: PathBuf,
        target_path: String,
        is_installed_cmd: Option<String>,
    ) -> Self {
        ProgramOptions {
            name,
            path,
            target_path,
            is_installed_cmd,
        }
    }
}

pub fn run(args: &Args) -> Result<Vec<ProgramOptions>> {
    let cloned_repo = clone_repo(&args.repo_url, &args.path)?;
    parse_dotfiles(cloned_repo)
}

fn magefile(path: PathBuf) -> Result<Table> {
    let magefile = fs::read_to_string(path)?;
    toml::from_str(&magefile).context("Failed to parse magefile")
}

fn parse_dotfiles(dotfiles: ReadDir) -> Result<Vec<ProgramOptions>> {
    let mut mage_config: Option<Table> = None;
    let mut programs = vec![];

    for item in dotfiles {
        let entry = item.context("Failed to read dir entry")?;
        if entry.file_name().to_str().unwrap().starts_with("magefile") {
            mage_config = Some(magefile(entry.path())?);
            continue;
        }

        let path = entry.path();
        let key = path
            .file_stem()
            .and_then(|s| s.to_str())
            .context(format!("Failed to get filename for {}", path.display()))?
            .to_string();

        programs.push((key, path));
    }
    let mage_config = mage_config.context("No magefile found")?;

    programs.retain(|(name, _)| mage_config.contains_key(name));

    let mut result = vec![];

    for (name, path) in programs.into_iter() {
        let program = create_program_options(&mage_config, name, path)?;
        result.push(program);
    }

    Ok(result)
}

fn create_program_options(magefile: &Table, name: String, path: PathBuf) -> Result<ProgramOptions> {
    let program = magefile
        .get(&name)
        .context(format!("No entry in magefile for `{}`", name))?;

    let target_path = program
        .get("target_path")
        .context(format!("No target_path for `{}`", name))?
        .to_string();

    let is_installed_cmd = program.get("is_installed_cmd").map(|cmd| cmd.to_string());

    Ok(ProgramOptions::new(
        name,
        path,
        target_path,
        is_installed_cmd,
    ))
}

fn clone_repo(url: &str, path: &str) -> Result<ReadDir> {
    if PathBuf::from(path).exists() {
        bail!("Path `{}` already exists", path)
    }

    std::process::Command::new("git")
        .args(["clone", url, path])
        .output()
        .context(format!("Failed to clone repo `{url}` into `{path}`"))?;

    fs::read_dir(path).context(format!("Failed to read cloned repo from `{path}`"))
}

#[cfg(test)]
mod tests {

    use super::*;

    struct Context;

    impl Drop for Context {
        fn drop(&mut self) {
            fs::remove_dir_all("/tmp/mage").unwrap_or_default();
        }
    }

    fn setup() -> Context {
        if fs::read_dir("/tmp/mage").is_ok() {
            fs::remove_dir_all("/tmp/mage").unwrap_or_default();
        }
        fs::create_dir("/tmp/mage").unwrap();
        Context
    }

    #[test]
    fn test_parse_dotfiles() {
        let _ctx = setup();

        // let args = Args {
        //     path: "/tmp/mage".to_string(),
        //     repo_url: "".to_string(),
        // };
        let dotfiles = fs::read_dir("test-dotfiles").unwrap();
        let programs = parse_dotfiles(dotfiles).unwrap();

        assert_eq!(programs.len(), 1);
    }

    #[test]
    fn does_not_clone_if_path_exists() {
        let result = clone_repo("empty", "test-dotfiles");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert_eq!(msg, "Path `test-dotfiles` already exists");
    }

    #[test]
    #[ignore]
    fn test_clone_repo() {
        fs::remove_dir_all("/tmp/mage").unwrap_or_default();
        clone_repo("https://github.com/ollivarila/brainfckr.git", "/tmp/mage").unwrap();
        let dir_exists = PathBuf::from("/tmp/mage").exists();
        assert!(dir_exists);
        fs::remove_dir_all("/tmp/mage").unwrap_or_default();
    }
}
