mod configure;
mod init;
use configure::configure;
use tracing::debug_span;

use crate::util::show_errors;

pub fn execute(directory: &str) -> anyhow::Result<()> {
    debug_span!("link").in_scope(|| {
        let programs = init::run(directory)?;
        let result = configure(programs);
        show_errors(result);
        Ok(())
    })
}
