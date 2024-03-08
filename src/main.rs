use std::fs;

use anyhow::Result;
use clap::{command, Parser};
use indicatif::ProgressBar;

use crate::configure::Configure;
mod configure;
mod setup;

const DEV: bool = std::option_env!("DEV").is_some();

fn main() -> Result<()> {
    let args = Args::parse();
    let bar = ProgressBar::new_spinner();
    if DEV {
        let dotfiles = fs::read_dir("test-dotfiles").unwrap();
        let programs = setup::parse_dotfiles(dotfiles)?;
        programs.configure(&bar)?;
    } else {
        let programs = setup::run(&args)?;
        configure::configure_dotfiles(programs)?;
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
