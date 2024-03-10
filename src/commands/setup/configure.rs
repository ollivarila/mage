use crate::dotfiles::ProgramOptions;
use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{fs, os::unix::fs::symlink, path::PathBuf};
use tracing::{debug, debug_span};

pub trait Configure<T> {
    fn configure(&self, bar: &impl Bar) -> Result<T>;
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

impl Configure<ConfigureDetails> for ProgramOptions {
    fn configure(&self, bar: &impl Bar) -> Result<ConfigureDetails> {
        bar.set_message(format!("Configuring {}", self.name));
        let name = self.name.clone();

        // Check if the config file already exists
        if self.target_path.exists() {
            debug!(target = ?self.target_path, "exists");
            bar.finish_with_message(format!("{} already configured ✔️", self.name));
            return Ok(ConfigureDetails::Installed(name));
        }

        // Check if the path to the config file exists
        ensure_path_ok(&self.target_path)?;

        // Create symlink from dotfiles to target path
        symlink(&self.path, &self.target_path)?;

        debug!(origin = ?self.path, target = ?self.target_path, "symlink");

        let details = match &self.is_installed_cmd {
            Some(cmd) if is_installed(cmd) => ConfigureDetails::Installed(name),
            Some(_) => ConfigureDetails::NotInstalled(name),
            None => ConfigureDetails::Installed(name), // Assume it is already installed
        };

        bar.finish_with_message(format!("{} configured ✔️", self.name));
        Ok(details)
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

fn is_installed(cmd: &str) -> bool {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map(|out| out.status.success())
        .unwrap_or_default()
}

#[derive(Debug, PartialEq)]
pub enum ConfigureDetails {
    Installed(String),
    NotInstalled(String),
    SomethingWrong(String),
}

pub fn configure<T>(programs: Vec<T>) -> Vec<ConfigureDetails>
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
            let _guard2 = span.enter();
            let bar = ProgressBar::new_spinner();
            let bar = mp.add(bar);
            let result = match program.configure(&bar) {
                Ok(details) => details,
                Err(e) => ConfigureDetails::SomethingWrong(e.to_string()),
            };
            debug!(result = ?result, "configured");
            result
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf, time::Duration};

    #[derive(Debug)]
    struct TestContext {
        target_path: PathBuf,
        opts: ProgramOptions,
    }

    impl Drop for TestContext {
        fn drop(&mut self) {
            fs::remove_file(&self.target_path).unwrap_or_default();
        }
    }

    struct MockBar;

    impl Bar for MockBar {
        fn finish_with_message(&self, _: impl Into<String>) {}
        fn set_message(&self, _: impl Into<String>) {}
    }

    fn setup() -> TestContext {
        let dotfiles_path = PathBuf::from("examples/test-dotfiles")
            .canonicalize()
            .unwrap();
        let target_path = PathBuf::from("/tmp/example.config");

        fs::remove_file(&target_path).unwrap_or_default();

        // Wait for file to be removed
        while target_path.exists() {
            std::thread::sleep(Duration::from_millis(100));
        }

        let opts = ProgramOptions {
            name: "example".to_string(),
            path: dotfiles_path,
            target_path: target_path.clone(),
            is_installed_cmd: None,
        };
        TestContext { target_path, opts }
    }

    #[test]
    fn test_configure_program_with_file() {
        let ctx = setup();
        let bar = MockBar;
        let installed = ctx.opts.configure(&bar).unwrap();

        assert!(&ctx.target_path.exists());
        assert_eq!(
            installed,
            ConfigureDetails::Installed("example".to_string())
        );
    }

    #[test]
    fn test_not_installed() {
        let mut ctx = setup();
        ctx.opts.is_installed_cmd = Some("false".to_string());
        let bar = MockBar;
        let installed = ctx.opts.configure(&bar).unwrap();
        assert_eq!(
            installed,
            ConfigureDetails::NotInstalled("example".to_string())
        );
    }

    #[test]
    fn test_check_installed() {
        assert!(is_installed("true"));
        assert!(!is_installed("false"));
    }

    #[test]
    fn test_configure_many() {
        let ctx = setup();
        assert!(!ctx.target_path.exists());
        let programs = vec![ctx.opts.clone()];
        let configured = configure(programs);

        let program = configured.get(0).unwrap();

        assert_eq!(configured.len(), 1);
        assert_eq!(*program, ConfigureDetails::Installed(ctx.opts.name.clone()));
        assert!(ctx.target_path.exists());
        assert!(ctx.target_path.is_symlink());
    }
}
