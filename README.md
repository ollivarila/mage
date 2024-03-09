# Mage

Mage is a tool for setting up your dotfiles quickly.

## How it works

It basically clones a repository and then creates symlinks to the correct places specified in **magefile.toml**.

The repository should contain all of the config files and the [magefile](#magefile).

Mage does not do anything if the target_path or the repository clone path exists. (trying not to break anything)

## Magefile

Magefile is in the toml format.

It contains entries for each of the configurations you want to set up.

For example

```toml
[bash]
target_path = "~/.bashrc"
is_installed_cmd = "which bash"
```

### Entry format

- target_path: full path (~ is expanded), this is the target for the symlink
- is_installed_cmd: optional, a command which exits with non zero exit code if the program is not installed. This can be helpful to understand which programs are missing from a fresh system.

## Usage

See:

```
mage --help
```

For most cases

## Requirements

- git

## Installation

Install with cargo:

```
cargo install --git https://github.com/ollivarila/mage
```

## Future plans

- subcommands
  - sync (pull changes to repo and refresh symlinks)
  - push ?
  - init
- tracing for debugging
- automatic installation of missing programs ?
