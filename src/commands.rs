mod clean;
mod init;
mod setup;

pub trait Exec {
    fn execute(&self) -> anyhow::Result<()>;
}

impl Exec for crate::Command {
    fn execute(&self) -> anyhow::Result<()> {
        match self {
            Self::Setup {
                origin,
                dotfiles_path,
            } => setup::execute(origin, dotfiles_path),
            Self::Clean { dotfiles_path } => clean::execute(dotfiles_path),
            Self::Init => init::execute(),
            // cmd => Err(anyhow::anyhow!("Unimplemented command: {:?}", cmd)),
        }
    }
}
