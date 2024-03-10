use anyhow::Context;
use std::{fs::File, io::Write};

pub(crate) fn execute() -> anyhow::Result<()> {
    let file = File::create("./magefile.toml").context("create magefile")?;
    write_contents(file)
}

fn write_contents(file: File) -> anyhow::Result<()> {
    let mut magefile = Magefile { file };

    magefile.writeln("[example]")?;
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
