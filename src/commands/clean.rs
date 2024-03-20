use anyhow::ensure;
use anyhow::Context;
use std::fs;
use tracing::{debug, debug_span};

use crate::dotfiles::find_magefile;
use crate::dotfiles::generate_options;
use crate::dotfiles::ProgramOptions;
use crate::util::show_errors;

pub(crate) fn execute(dotfiles_path: &str) -> anyhow::Result<()> {
    let span = debug_span!("clean");
    let _guard = span.enter();
    let full_path = crate::util::get_full_path(dotfiles_path);

    ensure!(
        full_path.exists(),
        format!("invalid path: {}", full_path.display())
    );

    let magefile = find_magefile(&full_path)?;
    let programs = generate_options(magefile, full_path.to_str().expect("should not fail"))?;
    let errors = programs.iter().map(Undo::undo).collect::<Vec<_>>();
    show_errors(errors);

    Ok(())
}

trait Undo {
    fn undo(&self) -> anyhow::Result<()>;
}

impl Undo for ProgramOptions {
    fn undo(&self) -> anyhow::Result<()> {
        let span = debug_span!("program", origin = ?self.origin_path);
        let _guard = span.enter();

        // Only remove file if it is a symlink
        if self.target_path.exists() && self.target_path.is_symlink() {
            fs::remove_dir_all(self.target_path.clone())
                .context(format!("delete symlink for: {:?}", self.origin_path))?;
            debug!(symlink = ?self.target_path, "delete");
        } else {
            debug!(target = ?self.target_path, "not a symlink");
            println!(
                "{:?} is not a symlink or it doesn't exists, skipping ✔️",
                self.origin_path
            );
            return Ok(());
        }

        println!("{:?} cleaned ✔️", self.origin_path);
        debug!("done");
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::{os::unix::fs::symlink, path::PathBuf};

    use super::*;

    fn setup() -> PathBuf {
        let original = PathBuf::from("examples/test-dotfiles/example.config")
            .canonicalize()
            .expect("should be able to canonicalize");
        let target = PathBuf::from("/tmp/example.config");
        if target.exists() {
            fs::remove_dir_all(&target).expect("should be able to remove this dir");
        }

        symlink(original, target).expect("should be able to create this symlink");

        PathBuf::from("examples/test-dotfiles")
            .canonicalize()
            .unwrap()
    }

    #[test]
    fn test_clean_cmd() {
        let dotfiles_path = setup();
        let _res = execute(dotfiles_path.to_str().unwrap()).unwrap();

        let target_path = PathBuf::from("/tmp/example.config");
        assert!(!target_path.exists());
    }

    #[test]
    fn invalid_path() {
        let invalid_path = "asdfsdf";
        let err = execute(invalid_path).unwrap_err().to_string();

        assert_eq!(err, "invalid path");
    }

    #[test]
    fn no_magefile() {
        let invalid_path = "/tmp";
        let err = execute(invalid_path).unwrap_err().to_string();
        assert_eq!(err, "Magefile not found");
    }
}
