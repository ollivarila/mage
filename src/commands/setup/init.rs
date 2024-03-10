use crate::dotfiles::{DotfilesEntry, DotfilesOrigin, ProgramOptions};
use anyhow::{Context, Result};
use std::{fs::ReadDir, path::PathBuf};
use toml::Table;
use tracing::{debug, debug_span};

pub fn run(origin: &str, dotfiles_path: &str) -> Result<Vec<ProgramOptions>> {
    debug_span!("setup").in_scope(|| {
        let read_dir = init_origin(origin, dotfiles_path)?;
        read_dotfiles(read_dir)
    })
}

fn init_origin(origin: &str, dotfiles_path: &str) -> Result<ReadDir> {
    let origin: DotfilesOrigin = (origin, dotfiles_path).try_into()?;
    origin.try_into()
}

fn read_dotfiles(dotfiles: ReadDir) -> Result<Vec<ProgramOptions>> {
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
        let program_config = magefile
            .get(&name)
            .context(format!("find {name} from magefile"))?;

        let target_path = program_config
            .get("target_path")
            .map(|p| p.as_str().unwrap())
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
        assert_eq!(program.name, "example");
        let target_path = program.target_path.to_str().unwrap();

        assert_eq!(target_path, "/tmp/example.config")
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
    fn init_with_invalid_args() {
        // Invalid origin
        let result = run("asdf", "asdf");
        assert!(result.is_err());
    }

    #[test]
    fn setup_with_valid_args() {
        let mut ctx = setup();
        ctx.path = "/tmp/valid".to_string();
        let result = run("examples/test-dotfiles", "/temp/valid");

        assert!(result.is_ok());
        let programs = result.unwrap();
        assert_eq!(programs.len(), 1);
    }
}
