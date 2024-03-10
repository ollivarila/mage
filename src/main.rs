use clap::{command, Parser, Subcommand};
use commands::Exec;
use std::fmt::Debug;
use tracing::Level;
mod commands;
mod dotfiles;

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
    #[arg(short, long, default_value = "false", help = "Enable debug logging")]
    debug: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(about = "Creates an example magefile in the working directory")]
    Init,
    Clean {
        #[arg(
            short = 'p',
            long,
            help = "Path where dotfiles are located",
            default_value = "~/dotfiles"
        )]
        dotfiles_path: String,
    },
    Setup {
        #[arg(
            help = "Can either be url to a repository or a path to existing folder on the system"
        )]
        origin: String,
        #[arg(
            short = 'p',
            long,
            help = "Path where dotfiles are cloned into, ignored if origin is a path to an existing folder",
            default_value = "~/dotfiles"
        )]
        dotfiles_path: String,
    },
}
