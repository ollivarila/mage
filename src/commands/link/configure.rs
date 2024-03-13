use crate::dotfiles::ProgramOptions;
use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{fs, os::unix::fs::symlink, path::PathBuf};
use tracing::{debug, debug_span};

// FIXME make bar messages pretty

pub trait Configure {
    fn configure(&self, bar: &impl Bar) -> Result<()>;
}

pub trait Bar {
    fn set_message(&self, msg: impl Into<String>);
    fn finish_with_message(&self, msg: impl Into<String>);
}

impl Bar for ProgressBar {
    fn finish_with_message(&self, msg: impl Into<String>) {
        self.finish_with_message(msg.into())
    }

    fn set_message(&self, msg: impl Into<String>) {
        self.set_message(msg.into())
    }
}

impl Configure for ProgramOptions {
    fn configure(&self, bar: &impl Bar) -> Result<()> {
        bar.set_message(format!("Configuring {}", self.name));
        let name = self.name.clone();

        // Check if the config file already exists
        if self.target_path.exists() {
            debug!(target = ?self.target_path, "exists");
            bar.finish_with_message(format!("{} already configured ✔️", self.name));
            return Ok(());
        }

        // Check if the path to the config file exists
        ensure_path_ok(&self.target_path)?;

        // Create symlink from dotfiles to target path
        symlink(&self.path, &self.target_path)?;

        debug!(origin = ?self.path, target = ?self.target_path, "symlink");

        bar.finish_with_message(format!("{} configured ✔️", self.name));
        Ok(())
    }
}

fn ensure_path_ok(full_path: &PathBuf) -> Result<()> {
    let parent = full_path.parent().context("get parent path")?;
    if !parent.exists() {
        debug!(path = ?parent, "created");
        fs::create_dir_all(parent)?;
    }

    Ok(())
}

#[derive(Debug, PartialEq)]
pub enum ConfigureDetails {
    Installed(String),
    NotInstalled(String),
    SomethingWrong(String),
}

pub fn configure<T>(programs: Vec<T>) -> Vec<anyhow::Result<()>>
where
    T: Into<ProgramOptions>,
{
    let span = debug_span!("configure");
    let _guard = span.enter();
    let programs = programs
        .into_iter()
        .map(|t| t.into())
        .collect::<Vec<ProgramOptions>>();

    let mp = MultiProgress::new();
    programs
        .into_par_iter()
        .map(|program| {
            let name = &program.name;
            let span = debug_span!("program", name);
            let _guard = span.enter();
            let bar = ProgressBar::new_spinner();
            let bar = mp.add(bar);
            program.configure(&bar)?;
            debug!("done");
            Ok(())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::test_context::Ctx;

    struct MockBar;

    impl Bar for MockBar {
        fn finish_with_message(&self, _: impl Into<String>) {}
        fn set_message(&self, _: impl Into<String>) {}
    }
    #[test]
    fn test_configure_program_with_file() {
        let ctx = Ctx::default();
        let bar = MockBar;

        assert!(ctx.opts.configure(&bar).is_ok());
        assert!(&ctx.target_file.clone().unwrap().exists());
    }

    #[test]
    fn test_configure_many() {
        let ctx = Ctx::default();
        let target_file = ctx.target_file.clone().unwrap();
        assert!(!target_file.exists());
        let programs = vec![ctx.opts.clone()];
        let configured = configure(programs);

        assert!(configured.get(0).unwrap().is_ok());
        assert_eq!(configured.len(), 1);
        assert!(target_file.exists());
        assert!(target_file.is_symlink());
    }
}
