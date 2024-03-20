use std::{path::PathBuf, process::Command};

use anyhow::Context;

use crate::util::get_full_path;

pub(crate) fn execute(directory: &str) -> Result<(), anyhow::Error> {
    let path = get_full_path(directory);
    git_pull(&path)?;
    // TODO: Run configure
    todo!()
}

fn git_pull(dir: &PathBuf) -> anyhow::Result<()> {
    Command::new("git")
        .arg("pull")
        .current_dir(dir)
        .status()
        .context("git pull failed")?;

    Ok(())
}
