use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

/// Displays errors if there are any
pub(crate) fn show_errors(result: Vec<anyhow::Result<()>>) {
    let mut msg = String::new();

    for res in result {
        if let Err(e) = res {
            msg.push_str(format!("{}\n", e).as_str());
        }
    }

    if !msg.is_empty() {
        eprintln!("Some errors occurred:\n{msg}");
    }
}

/// Expands `~` to the home directory
pub(crate) fn get_full_path<P: Into<PathBuf>>(path: P) -> PathBuf {
    let path: PathBuf = path.into();

    if path.starts_with("~") {
        let home = std::env::var("HOME").unwrap();
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

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct FullPath {
    path: PathBuf,
}

impl Display for FullPath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

impl FullPath {
    pub(crate) fn path(self) -> PathBuf {
        self.path
    }
    pub fn as_path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn to_str(&self) -> &str {
        self.path
            .to_str()
            .expect("should not fail to convert path to str")
    }
}

impl AsRef<Path> for FullPath {
    fn as_ref(&self) -> &Path {
        self.path.as_path()
    }
}

impl<T: Into<PathBuf>> From<T> for FullPath {
    fn from(value: T) -> Self {
        Self {
            path: get_full_path(value.into()),
        }
    }
}

#[cfg(test)]
pub(crate) mod test_context {
    #![allow(dead_code)]
    use std::{fs, path::PathBuf};

    use crate::dotfiles::ProgramOptions;

    #[derive(Debug)]
    pub struct Ctx {
        pub target_file: Option<PathBuf>,
        pub target_dir: Option<PathBuf>,
        pub opts: ProgramOptions,
    }

    impl Ctx {
        pub fn set_target_file(&mut self, file: PathBuf) {
            self.target_file = Some(file);
        }
        pub fn set_target_dir(&mut self, dir: PathBuf) {
            self.target_dir = Some(dir);
        }
    }

    impl Drop for Ctx {
        fn drop(&mut self) {
            if let Some(file) = &self.target_file {
                fs::remove_file(file).unwrap_or_default();
            }
            if let Some(dir) = &self.target_dir {
                fs::remove_dir_all(dir).unwrap_or_default();
            }
        }
    }

    impl Default for Ctx {
        fn default() -> Self {
            let dotfiles_path = PathBuf::from("examples/test-dotfiles")
                .canonicalize()
                .unwrap();
            let target_path = PathBuf::from(unique_tmp_path());

            let opts = ProgramOptions {
                origin_path: dotfiles_path.into(),
                target_path: target_path.clone().into(),
            };

            Ctx {
                target_file: Some(target_path),
                target_dir: None,
                opts,
            }
        }
    }

    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;

    fn unique_tmp_path() -> String {
        let mut hash = std::hash::DefaultHasher::default();
        SystemTime::now().hash(&mut hash);
        let s = hash.finish();
        format!("/tmp/example{}.config", s)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_full_path() {
        let home = std::env::var("HOME").unwrap();
        let expected = PathBuf::from(format!("{home}/test"));
        let path = get_full_path("~/test");
        assert_eq!(path, expected);

        let expected = PathBuf::from("/tmp/test");
        let path = get_full_path("/tmp/test");
        assert_eq!(path, expected);
    }
}
