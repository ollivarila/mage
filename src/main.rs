use std::{
    fs::{DirEntry, ReadDir},
    io::Read,
    rc::Rc,
};

use clap::Parser;
use toml::Table;

fn main() {
    let args = Args::parse();
    let programs = parse_programs(args);
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    repo_url: String,
    #[arg(
        short,
        long,
        help = "Path where dotfiles are cloned into",
        default_value = "~"
    )]
    path: String,
}

struct ProgramConfig {
    dir_entry: DirEntry,
    options: ProgramOptions,
}

struct ProgramOptions {
    name: String,
    target: String,
    is_installed_cmd: Option<String>,
}

fn parse_programs(args: Args) -> Option<Vec<ProgramConfig>> {
    const S: &str = include_str!("../mage-example.toml");
    let table = S.parse::<Table>().unwrap();
    let dotfiles = clone_repo(&args.repo_url, &args.path)
        .expect("failed to clone repo")
        .map(|item| item.expect("failed to extract dir entry"))
        .map(|item| {
            item.file_name()
                .to_str()
                .unwrap()
                .trim_end_matches(".*")
                .to_string()
        })
        .collect::<Vec<_>>();
    todo!()
}

fn clone_repo(url: &str, path: &str) -> Option<ReadDir> {
    todo!()
}
