use anyhow::anyhow;

use crate::{
    dotfiles::{clone_repo, DotfilesOrigin},
    util::get_full_path,
};

pub(crate) fn execute(repository: &str, directory: &str) -> Result<(), anyhow::Error> {
    let full_dir_path = get_full_path(directory)
        .to_str()
        .expect("should be able to convert back to str")
        .to_string();

    match repository.parse()? {
        DotfilesOrigin::Repository(repo, _) => clone_repo(&repo, &full_dir_path).map(|_| ()),
        _ => Err(anyhow!("Invalid repository: {repository}")),
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::util::test_context::Ctx;

    use super::*;

    #[ignore = "clones repo always"]
    #[test]
    fn it_executes() {
        let mut ctx = Ctx::default();
        ctx.set_target_dir(PathBuf::from("/tmp/test"));
        let repo = "https://github.com/ollivarila/brainfckr";
        let dir = "/tmp/test";
        execute(repo, dir).unwrap();
        assert!(PathBuf::from(dir).exists())
    }

    #[test]
    fn invalid_url() {
        let repo = "invalid";
        let dir = "/tmp/test";
        let result = execute(repo, dir);

        assert!(result.is_err())
    }
}
