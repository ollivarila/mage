use anyhow::Context;
use std::{fs::File, io::Write, path::PathBuf};

pub(crate) fn execute(path: impl Into<PathBuf>) -> anyhow::Result<()> {
    let mut path: PathBuf = path.into();
    path.push("magefile.toml");
    let mut magefile = Magefile {
        file: File::create(path).context("create magefile")?,
    };

    magefile.writeln("[\"example.config\"]")?;
    magefile.writeln("target_path = \"~/.config/example.config\"")?;

    Ok(())
}

struct Magefile {
    file: File,
}

impl Magefile {
    fn writeln(&mut self, s: impl Into<String>) -> anyhow::Result<()> {
        let mut buf: String = s.into();
        buf.push('\n');
        let bytes = buf.as_bytes();

        let n = self.file.write(bytes).context("write to magefile")?;

        let len = bytes.len();
        anyhow::ensure!(n == len, "wrote less bytes than expected");
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::{fs, path::PathBuf};

    use super::*;

    #[test]
    fn test_init_cmd() {
        let res = execute("/tmp");

        assert!(res.is_ok());

        let path = PathBuf::from("/tmp/magefile.toml");
        let exists = path.exists();

        fs::remove_file(path).unwrap();
        assert!(exists)
    }
}
