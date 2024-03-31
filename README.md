# Mage

Mage is a tool for setting up your dotfiles quickly.

## How it works

It basically clones a repository and then creates symlinks to the correct places specified in **magefile.toml**.
The repository should contain all of the config files and the [magefile](#magefile).
Mage does not do anything if the target_path or the repository clone path exists. (trying not to break anything)

## Magefile

Magefile is in the toml format.
It contains entries for each of the configurations you want to set up.
For example:

```toml
[".bashrc"]
target_path = "~/.bashrc"
```

or

```toml
["nested/.bashrc"]
target_path = "~/.bashrc"
```

### Entry format

- key: name of the file that the configuration is for,  
the path is assumed to be relative to the root of the repository
- target_path: full path (~ is expanded), this is the target for the symlink

## Usage

See:

```sh
mage --help
```

## Requirements

- git

## Installation

Install with cargo:

```sh
cargo install --git https://github.com/ollivarila/mage
```

## Future plans

- subcommands
  - push ?
- use [git2-rs](https://github.com/rust-lang/git2-rs)
