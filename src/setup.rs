use crate::{
    error::MageError::{
        self, InvalidDotfilesOrigin, InvalidMageFile, MageFileNotFound, Unexpected,
    },
    Args, MageResult,
};
use std::{
    fs::{self, read_dir, DirEntry, ReadDir},
    path::PathBuf,
};
use toml::Table;

#[derive(Debug, Clone)]
pub struct ProgramOptions {
    pub name: String,
    pub path: PathBuf,
    pub target_path: PathBuf,
    pub is_installed_cmd: Option<String>,
}

#[derive(PartialEq, Debug)]
pub enum DotfilesEntry {
    ConfigFileOrDir(String, PathBuf),
    Magefile(Table),
}

#[derive(PartialEq, Debug)]
enum DotfilesOrigin {
    Directory(PathBuf),
    Repository(String, String),
}

impl TryFrom<&Args> for ReadDir {
    type Error = MageError;

    fn try_from(args: &Args) -> Result<Self, Self::Error> {
        let origin: DotfilesOrigin =
            (args.origin.as_str(), args.dotfiles_path.as_str()).try_into()?;
        origin.try_into()
    }
}

impl TryFrom<(&str, &str)> for DotfilesOrigin {
    type Error = MageError;

    fn try_from((origin, clone_path): (&str, &str)) -> Result<Self, Self::Error> {
        match origin {
            url if is_url(url) => Ok(DotfilesOrigin::Repository(
                url.to_string(),
                clone_path.to_string(),
            )),
            dir if is_dir(dir) => Ok(DotfilesOrigin::Directory(PathBuf::from(dir))),
            _ => Err(InvalidDotfilesOrigin(format!("{origin}"))),
        }
    }
}

fn is_url(s: &str) -> bool {
    s.starts_with("https://") || s.starts_with("http://")
}

fn is_dir(s: &str) -> bool {
    let path = PathBuf::from(s);
    path.is_dir()
}

impl TryFrom<DotfilesOrigin> for ReadDir {
    type Error = MageError;
    fn try_from(origin: DotfilesOrigin) -> Result<Self, Self::Error> {
        match origin {
            DotfilesOrigin::Directory(dir) => read_dir(dir).map_err(Into::into),
            DotfilesOrigin::Repository(url, path) => clone_repo_and_read(&url, &path),
        }
    }
}

impl TryFrom<DirEntry> for DotfilesEntry {
    type Error = MageError;

    fn try_from(entry: DirEntry) -> Result<Self, Self::Error> {
        let is_magefile = entry
            .file_name()
            .to_str()
            .ok_or(Unexpected("Failed to convert to string".to_string()))?
            .starts_with("magefile");

        if is_magefile {
            let magefile = magefile(entry.path())?;
            return Ok(DotfilesEntry::Magefile(magefile));
        }

        let path = entry
            .path()
            .canonicalize()
            .or(Err(Unexpected("Failed to canonicalize path".to_string())))?;

        let key = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or(Unexpected(format!(
                "Failed to get file stem for {:?}",
                path
            )))?
            .to_string();

        Ok(DotfilesEntry::ConfigFileOrDir(key, path))
    }
}

fn get_full_path(path: PathBuf) -> PathBuf {
    let home = std::env::var("HOME").unwrap();

    if path.starts_with("~") {
        let mut full_path = PathBuf::from(home);
        for item in path.iter() {
            if item.to_str().unwrap() != "~" {
                full_path.push(item)
            }
        }
        return full_path;
    }

    path
}

pub fn run(args: &Args) -> MageResult<Vec<ProgramOptions>> {
    parse_dotfiles(args.try_into()?)
}

fn magefile(path: PathBuf) -> MageResult<Table> {
    let magefile = fs::read_to_string(path)?;
    let thing: Table = toml::from_str(&magefile).map_err(|e| {
        let msg = format!("Failed to parse magefile:\n{}", e);
        InvalidMageFile(msg)
    })?;

    Ok(thing)
}

pub fn parse_dotfiles(dotfiles: ReadDir) -> MageResult<Vec<ProgramOptions>> {
    let mut mage_config: Option<Table> = None;
    let mut programs = vec![];

    for item in dotfiles {
        match item?.try_into()? {
            DotfilesEntry::Magefile(magefile) => mage_config = Some(magefile),
            DotfilesEntry::ConfigFileOrDir(name, path) => programs.push((name, path)),
        }
    }
    let mage_config = mage_config.ok_or(MageFileNotFound)?;

    // Skip config files that are not in the magefile
    programs.retain(|(name, _)| mage_config.contains_key(name));

    let mut result = vec![];

    for (name, path) in programs.into_iter() {
        let program = (&mage_config, name, path).try_into()?;
        result.push(program);
    }

    Ok(result)
}

impl TryFrom<(&Table, String, PathBuf)> for ProgramOptions {
    type Error = MageError;

    fn try_from((magefile, name, path): (&Table, String, PathBuf)) -> Result<Self, Self::Error> {
        let program_config = magefile.get(&name).ok_or(InvalidMageFile(format!(
            "Missing key: {name} from magefile"
        )))?;

        let target_path = program_config
            .get("target_path")
            .map(|p| p.as_str().unwrap())
            .map(PathBuf::from)
            .map(get_full_path)
            .ok_or(InvalidMageFile(format!("{name} missing key: target_path")))?;

        let is_installed_cmd = program_config
            .get("is_installed_cmd")
            .map(|cmd| cmd.to_string());

        Ok(ProgramOptions {
            name,
            path,
            target_path,
            is_installed_cmd,
        })
    }
}

fn clone_repo_and_read(url: &str, path: &str) -> MageResult<ReadDir> {
    if PathBuf::from(path).exists() {
        return Err(Unexpected("Target clone path already exists".to_string()));
    }

    std::process::Command::new("git")
        .args(["clone", url, path])
        .output()?;

    let res = fs::read_dir(path)?;

    Ok(res)
}

#[cfg(test)]
mod tests {

    use super::*;

    struct Context {
        path: String,
    }

    impl Drop for Context {
        fn drop(&mut self) {
            fs::remove_dir_all(self.path.clone()).unwrap_or_default();
        }
    }

    fn setup() -> Context {
        let path = "/tmp/mage".to_string();
        if fs::read_dir(&path).is_ok() {
            fs::remove_dir_all(&path).unwrap_or_default();
        }
        Context { path }
    }

    #[test]
    fn test_parse_dotfiles() {
        let _ctx = setup();

        let dotfiles = fs::read_dir("examples/test-dotfiles").unwrap();
        let programs = parse_dotfiles(dotfiles).unwrap();

        assert_eq!(programs.len(), 1);

        let program = &programs[0];
        assert_eq!(program.name, "example");
        let target_path = program.target_path.to_str().unwrap();

        assert_eq!(target_path, "/tmp/example.config")
    }

    #[test]
    fn does_not_clone_if_path_exists() {
        let result = clone_repo_and_read("empty", "examples/test-dotfiles");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert_eq!(msg, "Error: Target clone path already exists");
    }

    #[test]
    #[ignore]
    fn test_clone_repo() {
        fs::remove_dir_all("/tmp/mage").unwrap_or_default();
        clone_repo_and_read("https://github.com/ollivarila/brainfckr.git", "/tmp/mage").unwrap();
        let dir_exists = PathBuf::from("/tmp/mage").exists();
        assert!(dir_exists);
        fs::remove_dir_all("/tmp/mage").unwrap_or_default();
    }

    #[test]
    fn test_get_full_path() {
        let path = PathBuf::from("~/test");
        let home = std::env::var("HOME").unwrap();
        let expected = PathBuf::from(format!("{home}/test"));
        let path = get_full_path(path);
        assert_eq!(path, expected);
    }

    #[test]
    fn dotfiles_origin_from_str() {
        let origin = ("/tmp", "asdf");
        let df_origin: DotfilesOrigin = origin.try_into().unwrap();

        assert_eq!(df_origin, DotfilesOrigin::Directory(PathBuf::from("/tmp")));

        let origin = ("https://google.com", "bar");
        let df_origin: DotfilesOrigin = origin.try_into().unwrap();
        let should_be =
            DotfilesOrigin::Repository("https://google.com".to_string(), "bar".to_string());
        assert_eq!(df_origin, should_be);

        let origin = ("asdf", "bar");
        let df_origin: Result<DotfilesOrigin, _> = origin.try_into();
        assert!(df_origin.is_err())
    }

    #[test]
    fn dotfiles_origin_into_readdir() {
        let origin = DotfilesOrigin::Directory(PathBuf::from("examples/test-dotfiles"));
        let readdir: ReadDir = origin.try_into().unwrap();

        let n = readdir.count();

        // another
        // example.config
        // magefile.toml
        assert_eq!(n, 3)
    }

    #[test]
    fn invalid_magefile() {
        let path = PathBuf::from("examples/test-dotfiles/example.config");
        let magefile = magefile(path);
        assert!(magefile.is_err());
        match magefile.unwrap_err() {
            InvalidMageFile(_) => {}
            val => assert!(false, "Got: {}", val),
        }
    }

    #[test]
    fn setup_with_invalid_args() {
        // Invalid origin
        let args = Args {
            origin: "asdfsd".to_string(),
            dotfiles_path: "sadfas".to_string(),
        };

        let result = run(&args);
        assert!(result.is_err());

        match result.unwrap_err() {
            InvalidDotfilesOrigin(_) => {}
            val => assert!(false, "Got: {}", val),
        }
    }

    #[test]
    fn setup_with_valid_args() {
        let mut ctx = setup();
        ctx.path = "/tmp/valid".to_string();
        let args = Args {
            origin: "examples/test-dotfiles".to_string(),
            dotfiles_path: ctx.path.clone(),
        };
        let result = run(&args);

        assert!(result.is_ok());
        let programs = result.unwrap();
        assert_eq!(programs.len(), 1);
    }
}
