use std::{path::PathBuf, process::Command};

use anyhow::Context;

use crate::util::FullPath;

// TODO: maybe do some diffing
pub(crate) fn execute(directory: &str) -> Result<(), anyhow::Error> {
    let syncer = Syncer::with_dir(directory);
    syncer.pull()?;

    println!("Running clean...");
    crate::commands::clean::execute(directory)?;
    println!();

    println!("Running link...");
    crate::commands::link::execute(directory)
}

struct Syncer {
    directory: FullPath,
    pull_fn: Box<dyn FnOnce(&PathBuf) -> anyhow::Result<()>>,
}

impl Default for Syncer {
    fn default() -> Self {
        Self {
            directory: "~/.mage".into(),
            pull_fn: Box::new(git_pull),
        }
    }
}

impl Syncer {
    fn with_dir<D: Into<FullPath>>(directory: D) -> Self {
        Self {
            directory: directory.into(),
            ..Default::default()
        }
    }

    fn pull(self) -> anyhow::Result<()> {
        (self.pull_fn)(&self.directory.path())
    }
}

fn git_pull(dir: &PathBuf) -> anyhow::Result<()> {
    Command::new("git")
        .arg("pull")
        .current_dir(dir)
        .status()
        .context("git pull failed")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syncer_with_dir() {
        let dir = FullPath::from("/tmp");
        let syncer = Syncer::with_dir(dir);

        assert_eq!(syncer.directory.path(), PathBuf::from("/tmp"));
    }
}
