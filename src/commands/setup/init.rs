use crate::dotfiles::{read_dotfiles, DotfilesOrigin, ProgramOptions};
use anyhow::{Context, Result};
use std::{fs::ReadDir, path::PathBuf};
use toml::Table;
use tracing::debug_span;

pub fn run(origin: &str, dotfiles_path: &str) -> Result<Vec<ProgramOptions>> {
    debug_span!("init").in_scope(|| {
        let read_dir = init_origin(origin, dotfiles_path)?;
        read_dotfiles(read_dir)
    })
}

fn init_origin(origin: &str, dotfiles_path: &str) -> Result<ReadDir> {
    let origin: DotfilesOrigin = (origin, dotfiles_path).try_into()?;
    origin.try_into()
}

fn get_full_path(path: PathBuf) -> PathBuf {
    let home = std::env::var("HOME").unwrap();

    if path.starts_with("~") {
        let mut full_path = PathBuf::from(home);
        for item in path.iter() {
            if item.to_str().unwrap() != "~" {
                full_path.push(item)
            }
        }
        return full_path;
    }

    path
}

impl TryFrom<(&Table, String, PathBuf)> for ProgramOptions {
    type Error = anyhow::Error;

    fn try_from((magefile, name, path): (&Table, String, PathBuf)) -> Result<Self, Self::Error> {
        dbg!(magefile);

        let program_config = magefile
            .get(&name)
            .context(format!("find {name} from magefile"))?;

        let target_path = program_config
            .get("target_path")
            .map(|p| p.as_str().expect("should be able to convert value to str"))
            .map(PathBuf::from)
            .map(get_full_path)
            .context(format!("{name} missing key: target_path"))?;

        let is_installed_cmd = program_config
            .get("is_installed_cmd")
            .map(|cmd| cmd.to_string());

        Ok(ProgramOptions {
            name,
            path,
            target_path,
            is_installed_cmd,
        })
    }
}

#[cfg(test)]
mod tests {

    use std::fs;

    use super::*;

    struct Context {
        path: PathBuf,
    }

    impl Drop for Context {
        fn drop(&mut self) {
            fs::remove_dir_all(self.path.clone()).unwrap_or_default();
        }
    }

    fn setup() -> Context {
        let path = PathBuf::from("/tmp/mage");
        if path.exists() {
            fs::remove_dir_all(&path).unwrap_or_default();
        }
        Context { path }
    }

    #[test]
    fn test_get_full_path() {
        let path = PathBuf::from("~/test");
        let home = std::env::var("HOME").unwrap();
        let expected = PathBuf::from(format!("{home}/test"));
        let path = get_full_path(path);
        assert_eq!(path, expected);
    }

    #[test]
    fn setup_init_with_invalid_args() {
        // Invalid origin
        let result = run("asdf", "asdf");
        assert!(result.is_err());
    }

    #[test]
    fn setup_init_with_valid_args() {
        let mut ctx = setup();
        ctx.path = PathBuf::from("/tmp/valid");
        let programs = run("examples/test-dotfiles", "/temp/valid").unwrap();

        assert_eq!(programs.len(), 1);
    }
}
