use crate::dotfiles::{
    ensure_repo_is_setup, find_magefile, generate_options, DotfilesOrigin, ProgramOptions,
};
use anyhow::Result;
use std::path::PathBuf;
use tracing::debug_span;

pub fn run(directory: &str) -> Result<Vec<ProgramOptions>> {
    debug_span!("init").in_scope(|| {
        let full_path = init_dir(directory)?;
        let magefile = find_magefile(full_path)?;
        generate_options(magefile, directory)
    })
}

/// Clones repository or uses local directory
fn init_dir(directory: &str) -> Result<PathBuf> {
    let origin: DotfilesOrigin = directory.parse()?;
    ensure_repo_is_setup(origin)
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
