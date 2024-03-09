# Mage

Mage is a tool for setting up your dotfiles quickly.

## How it works

It basically clones a repository and then creates symlinks to the correct places specified in **magefile.toml**.

The repository should contain all of the config files and the [magefile](#magefile).

Mage does not do anything if the target_path or the repository clone path exists. (trying not to break anything)

Mage currently makes one assumption when creating symlinks. Which is that all of the origin paths are in the root of the dotfiles.  
For example

```toml
[bash]
target_path = "~/.bashrc"
```

Means that the origin needs to be in **dotfiles/.bashrc**, making nested configurations impossible currently.

Something like

```toml
[nvim]
target_path = "~/.config/nvim"
```

Where **nvim** is a folder located in **dotfiles** is still valid.

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

```sh
mage --help
```

For most cases

## Requirements

- git

## Installation

Install with cargo:

```sh
cargo install --git https://github.com/ollivarila/mage
```

## Future plans

- subcommands
  - sync (pull changes to repo and refresh symlinks)
  - push ?
  - init
- tracing for debugging
- automatic installation of missing programs ?
