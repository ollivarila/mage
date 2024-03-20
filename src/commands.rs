use anyhow::Context;

mod clean;
mod clone;
mod init;
mod link;
mod sync;

pub trait Exec {
    fn execute(&self) -> anyhow::Result<()>;
}

impl Exec for crate::Command {
    fn execute(&self) -> anyhow::Result<()> {
        match self {
            Self::Link { directory } => link::execute(directory),
            Self::Clean {
                directory: dotfiles_path,
            } => clean::execute(dotfiles_path),
            Self::Init => {
                let pwd = std::env::var("PWD").context("PWD environment variable not set")?;
                init::execute(pwd)
            }
            Self::Clone {
                repository,
                directory,
            } => clone::execute(repository, directory),
            Self::Sync { directory } => sync::execute(directory),
        }
    }
}
