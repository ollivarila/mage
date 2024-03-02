use anyhow::Result;
use clap::{command, Parser};
mod setup;

fn main() -> Result<()> {
    let args = Args::parse();
    let programs = setup::run(&args)?;
    dbg!(programs);

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    repo_url: String,
    #[arg(
        short,
        long,
        help = "Path where dotfiles are cloned into",
        default_value = "~/dotfiles"
    )]
    path: String,
}
