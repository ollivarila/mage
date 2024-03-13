mod configure;
mod init;
use configure::configure;
use tracing::debug_span;

pub fn execute(directory: &str) -> anyhow::Result<()> {
    debug_span!("link").in_scope(|| {
        let programs = init::run(directory)?;
        let result = configure(programs);
        show_errors(result);
        Ok(())
    })
}

fn show_errors(result: Vec<anyhow::Result<()>>) {
    let errors = result
        .iter()
        .map(|res| {
            if res.is_err() {
                res.as_ref().unwrap_err().to_string()
            } else {
                "".to_string()
            }
        })
        .reduce(|acc, e| format!("{acc}\n{e}"));

    match errors {
        Some(errors) => eprintln!("Some errors occurred:\n{errors}"),
        _ => {}
    };
}
