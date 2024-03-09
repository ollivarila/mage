use clap::{command, Parser};
use configure::{configure, ConfigureDetails};
use error::MageError;
use std::{fmt::Debug, fs, process::exit};
use tracing::Level;
mod configure;
mod error;
mod setup;

pub type MageResult<T> = Result<T, MageError>;

const DEV: bool = std::option_env!("MAGE_DEV").is_some();
const DEBUG: bool = std::option_env!("MAGE_DEBUG").is_some();

fn main() {
    let result = if DEV { run_dev_mode() } else { run_prod_mode() };

    if let Err(err) = result {
        eprintln!("{err}");
        exit(1)
    }
}

fn run_dev_mode() -> MageResult<()> {
    let _args = Args::parse();

    if DEBUG {
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .init();
    }

    let dotfiles = fs::read_dir("examples/test-dotfiles").unwrap();
    let programs = setup::read_dotfiles(dotfiles)?;
    configure(programs);

    Ok(())
}

fn run_prod_mode() -> MageResult<()> {
    let args = Args::parse();

    if args.debug {
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .init();
    }

    let programs = setup::run(&args)?;
    let result = configure(programs);
    display_result(result);

    Ok(())
}

fn display_result(result: Vec<ConfigureDetails>) {
    let mut err_msg = String::new();
    let mut not_installed_msg = String::new();

    for r in result {
        match r {
            ConfigureDetails::SomethingWrong(e) => {
                err_msg.push_str(&format!("{}\n", e));
            }
            ConfigureDetails::NotInstalled(p) => {
                not_installed_msg.push_str(&format!("{}\n", p));
            }
            _ => {}
        }
    }

    if !err_msg.is_empty() {
        eprintln!("Some errors occurred:\n{err_msg}")
    }

    if !not_installed_msg.is_empty() {
        println!("The following programs are not installed on this system:\n{not_installed_msg}")
    }
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(help = "Can either be url to a repository or a path to existing folder on the system")]
    origin: String,
    #[arg(
        short = 'p',
        long,
        help = "Path where dotfiles are cloned into, ignored if origin is a path to an existing folder",
        default_value = "~/dotfiles"
    )]
    dotfiles_path: String,

    #[arg(short, long, default_value = "false", help = "Enable debug logging")]
    debug: bool,
}
