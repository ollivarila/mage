use crate::dotfiles::ProgramOptions;
use anyhow::{ensure, Context, Result};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{fs, os::unix::fs::symlink, path::Path};
use tracing::{debug, debug_span};

pub trait Configure {
    fn configure(&self) -> Result<()>;
}

impl Configure for ProgramOptions {
    fn configure(&self) -> Result<()> {
        // Ensure the origin path exists
        ensure!(
            self.origin_path.exists(),
            format!("{} does not exist", self.origin_path.display())
        );

        // Check if the config file already exists
        if self.target_path.exists() {
            debug!(target = ?self.target_path, "exists");
            println!("{:?} already linked ✔️", self.origin_path);
            return Ok(());
        }

        // Check if the path to the config file exists
        ensure_path_ok(&self.target_path)?;

        // Create symlink from dotfiles to target path
        symlink(&self.origin_path, &self.target_path)?;

        debug!(origin = ?self.origin_path, target = ?self.target_path, "symlink");

        println!("{:?} linked ✔️", self.origin_path);
        Ok(())
    }
}

fn ensure_path_ok(full_path: &Path) -> Result<()> {
    let parent = full_path.parent().context("get parent path")?;
    if !parent.exists() {
        fs::create_dir_all(parent)?;
        debug!(path = ?parent, "created");
    }

    Ok(())
}

pub fn configure<T>(programs: T) -> Vec<anyhow::Result<()>>
where
    T: IntoParallelIterator<Item = ProgramOptions>,
{
    let span = debug_span!("configure");
    let _guard = span.enter();

    programs
        .into_par_iter()
        .map(|program| {
            let span = debug_span!("program", origin = ?program.origin_path);
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
