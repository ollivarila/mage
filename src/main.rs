use anyhow::Result;
use clap::{command, Parser};
use indicatif::ProgressBar;

use crate::configure::Configure;
mod configure;
mod setup;

static DEV: bool = true;

fn main() -> Result<()> {
    let args = Args::parse();
    let bar = ProgressBar::new_spinner();
    if DEV {
        let args = Args {
            repo_url: "".to_string(),
            path: "/tmp/mage".to_string(),
        };
        let programs = setup::run(&args)?;
        programs.configure(&bar)?;
    } else {
        let programs = setup::run(&args)?;
        programs.configure(&bar)?;
    }

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
