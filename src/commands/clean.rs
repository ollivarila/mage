use anyhow::Context;
use std::{fs, path::PathBuf};
use tracing::{debug, debug_span};

use crate::dotfiles::{read_dotfiles, DotfilesOrigin};

pub(crate) fn execute(dotfiles_path: &str) -> anyhow::Result<()> {
    todo!("not tested");

    let span = debug_span!("clean");
    let _guard = span.enter();
    let path = PathBuf::from(dotfiles_path);

    if !path.exists() {
        anyhow::bail!("invalid path")
    }

    let dotfiles_origin = DotfilesOrigin::Directory(path);
    let programs = read_dotfiles(dotfiles_origin.try_into()?)?;

    for program in programs {
        let span = debug_span!("program", name = ?program.name);
        let _guard = span.enter();

        if program.target_path.exists() {
            fs::remove_dir_all(program.target_path.clone())
                .context(format!("delete symlink for: {}", program.name))?;
            debug!(symlink = ?program.target_path, "delete");
        }

        debug!("done")
    }

    Ok(())
}

#[cfg(test)]
mod tests {}
