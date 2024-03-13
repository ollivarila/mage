mod clean;
mod init;
mod link;

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
            Self::Init => init::execute(),
            Self::Clone {
                repository,
                directory,
            } => todo!(), // cmd => Err(anyhow::anyhow!("Unimplemented command: {:?}", cmd)),
        }
    }
}
