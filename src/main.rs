use clap::{command, Parser, Subcommand};
use commands::Exec;
use std::fmt::Debug;
use tracing::Level;
mod commands;
mod dotfiles;
mod util;

// TODO: Push command
// TODO: Sync command
// TODO: More tests

const DEBUG: bool = std::option_env!("MAGE_DEBUG").is_some();

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.debug || DEBUG {
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .init();
    }

    args.command.execute()
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long, default_value = "false", help = "Show debug information")]
    debug: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(about = "Creates an example magefile in the working directory")]
    Init,
    #[command(about = "Removes all of the symlinks created by mage")]
    Clean {
        #[arg(
            short = 'p',
            long,
            help = "Location of the dotfiles",
            default_value = "~/.mage"
        )]
        directory: String,
    },
    #[command(about = "Link your dotfiles")]
    Link {
        #[arg(
            help = "Location of the dotfiles, can also be repository url",
            default_value = "~/.mage"
        )]
        directory: String,
    },
    #[command(about = "Clone your dotfiles repository")]
    Clone {
        #[arg(help = "Repository to be cloned, either full url or <github-username>/<repository>")]
        repository: String,
        #[arg(
            short,
            long,
            help = "Path where dotfiles are cloned into",
            default_value = "~/.mage"
        )]
        directory: String,
    },
    #[command(about = "Sync your dotfiles repository")]
    Sync {
        #[arg(
            short,
            long,
            help = "Location of the dotfiles, can be empty if they are in the default location",
            default_value = "~/.mage"
        )]
        directory: String,
    },
}
