use crate::{
    dotfiles::{ensure_repo_is_setup, find_magefile, DotfilesOrigin, ProgramOptions},
    util::FullPath,
};
use anyhow::Result;
use tracing::debug_span;

pub fn run(directory_or_repository: &str) -> Result<Vec<ProgramOptions>> {
    debug_span!("init").in_scope(|| {
        let full_path = init_dir(directory_or_repository)?;
        let magefile = find_magefile(full_path.as_ref())?;
        ProgramOptions::generate(magefile, full_path)
    })
}

/// Clones repository or uses local directory
fn init_dir(directory: &str) -> Result<FullPath> {
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
