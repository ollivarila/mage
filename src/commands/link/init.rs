use crate::dotfiles::{read_dotfiles, DotfilesOrigin, ProgramOptions};
use anyhow::{Context, Result};
use std::{fs::ReadDir, path::PathBuf};
use toml::Table;
use tracing::debug_span;

pub fn run(directory: &str) -> Result<Vec<ProgramOptions>> {
    debug_span!("init").in_scope(|| {
        let read_dir = init_dir(directory)?;
        read_dotfiles(read_dir)
    })
}

fn init_dir(directory: &str) -> Result<ReadDir> {
    let origin: DotfilesOrigin = directory.parse()?;
    origin.try_into()
}

impl TryFrom<(&Table, String, PathBuf)> for ProgramOptions {
    type Error = anyhow::Error;

    fn try_from((magefile, name, path): (&Table, String, PathBuf)) -> Result<Self, Self::Error> {
        let program_config = magefile
            .get(&name)
            .context(format!("find {name} from magefile"))?;

        let target_path = program_config
            .get("target_path")
            .map(|p| p.as_str().expect("should be able to convert value to str"))
            .map(PathBuf::from)
            .map(crate::util::get_full_path)
            .context(format!("{name} missing key: target_path"))?;

        let is_installed_cmd = program_config
            .get("is_installed_cmd")
            .map(|cmd| cmd.to_string());

        Ok(ProgramOptions {
            name,
            path,
            target_path,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::util::test_context::Ctx;

    #[test]
    fn link_init_with_invalid_args() {
        // Invalid origin
        let result = run("sdfdsf");
        assert!(result.is_err());
    }

    #[test]
    fn link_init_with_valid_args() {
        let mut _ctx = Ctx::default();
        let programs = run("examples/test-dotfiles").unwrap();

        assert_eq!(programs.len(), 1);
    }
}
