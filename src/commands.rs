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
            cmd => Err(anyhow::anyhow!("Unimplemented command: {:?}", cmd)),
        }
    }
}
