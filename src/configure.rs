use crate::setup::ProgramOptions;
use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{fs, os::unix::fs::symlink, path::PathBuf};

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

impl Configure<bool> for ProgramOptions {
    fn configure(&self, bar: &impl Bar) -> Result<bool> {
        bar.set_message(format!("Configuring {}", self.name));
        if self.target_path.exists() {
            bar.finish_with_message(format!("{} already configured ✔", self.name));
            return Ok(true);
        }

        // ~/.config/file
        // ~/file
        let parent = self
            .target_path
            .parent()
            .context("Could not get parent dir")?;
        if !parent.exists() {
            fs::create_dir_all(parent)?
        }

        symlink(self.path.clone(), self.target_path.clone()).context(format!(
            "Failed to create symlink from {} to {}",
            self.path.display(),
            self.target_path.display()
        ))?;

        let installed = match &self.is_installed_cmd {
            Some(cmd) => check_installed(cmd),
            None => true, // Assume it is already installed
        };

        bar.finish_with_message(format!("{} ✔", self.name));
        Ok(installed)
    }
}

fn check_installed(cmd: &str) -> bool {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map(|out| out.status.success())
        .unwrap_or_default()
}

impl Configure<Vec<String>> for Vec<ProgramOptions> {
    fn configure(&self, _: &impl Bar) -> Result<Vec<String>> {
        let mp = MultiProgress::new();
        let not_installed = self
            .par_iter()
            .filter_map(|program| {
                let bar = ProgressBar::new_spinner();
                let bar = mp.add(bar);
                match program.configure(&bar) {
                    Ok(installed) => {
                        if !installed {
                            Some(program.name.clone())
                        } else {
                            None
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to configure {}: {}", program.name, e);
                        None
                    }
                }
            })
            .collect();

        Ok(not_installed)
    }
}

pub fn configure_dotfiles(programs: Vec<ProgramOptions>) -> Result<Vec<String>> {
    let bar = ProgressBar::new_spinner();

    programs.configure(&bar)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

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
        let dotfiles_path = PathBuf::from("test-dotfiles").canonicalize().unwrap();
        let target_path = PathBuf::from("/tmp/example.config");

        fs::remove_file(&target_path).unwrap_or_default();

        let opts =
            ProgramOptions::new("test".to_string(), dotfiles_path, target_path.clone(), None);
        TestContext { target_path, opts }
    }

    #[test]
    fn test_configure_program_with_file() {
        let ctx = setup();
        let bar = MockBar;
        let installed = ctx.opts.configure(&bar).unwrap();

        assert!(&ctx.target_path.exists());
        assert!(installed);
    }

    #[test]
    fn test_not_installed() {
        let mut ctx = setup();
        ctx.opts.is_installed_cmd = Some("false".to_string());
        let bar = MockBar;
        let installed = ctx.opts.configure(&bar).unwrap();
        assert!(!installed)
    }

    #[test]
    fn test_check_installed() {
        assert!(check_installed("true"));
        assert!(!check_installed("false"));
    }
}
