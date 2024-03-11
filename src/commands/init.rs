use anyhow::Context;
use std::{fs::File, io::Write};

pub(crate) fn execute() -> anyhow::Result<()> {
    let mut magefile = Magefile {
        file: File::create("./magefile.toml").context("create magefile")?,
    };

    magefile.writeln("[\"example.config\"]")?;
    magefile.writeln("target_path = \"~/.config/example.config\"")?;
    magefile.writeln("is_installed_cmd = \"which echo\"")?;

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

        let n = self.file.write(&bytes).context("write to magefile")?;

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
        let res = execute();

        assert!(res.is_ok());

        let path = PathBuf::from("./magefile.toml");
        let exists = path.exists();

        fs::remove_file(path).unwrap();
        assert!(exists)
    }
}
