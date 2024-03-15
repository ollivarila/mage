use crate::dotfiles::ProgramOptions;
use anyhow::{Context, Result};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{fs, os::unix::fs::symlink, path::Path};
use tracing::{debug, debug_span};

pub trait Configure {
    fn configure(&self) -> Result<()>;
}

impl Configure for ProgramOptions {
    fn configure(&self) -> Result<()> {
        // Check if the config file already exists
        if self.target_path.exists() {
            debug!(target = ?self.target_path, "exists");
            println!("{} already configured ✔️", self.name);
            return Ok(());
        }

        // Check if the path to the config file exists
        ensure_path_ok(&self.target_path)?;

        // Create symlink from dotfiles to target path
        symlink(&self.path, &self.target_path)?;

        debug!(origin = ?self.path, target = ?self.target_path, "symlink");

        println!("{} configured ✔️", self.name);
        Ok(())
    }
}

fn ensure_path_ok(full_path: &Path) -> Result<()> {
    let parent = full_path.parent().context("get parent path")?;
    if !parent.exists() {
        debug!(path = ?parent, "created");
        fs::create_dir_all(parent)?;
    }

    Ok(())
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

    programs
        .into_par_iter()
        .map(|program| {
            let name = &program.name;
            let span = debug_span!("program", name);
            let _guard = span.enter();
            program.configure()?;
            debug!("done");
            Ok(())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::test_context::Ctx;

    #[test]
    fn test_configure_program_with_file() {
        let ctx = Ctx::default();

        assert!(ctx.opts.configure().is_ok());
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
