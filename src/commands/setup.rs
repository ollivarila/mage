mod configure;
mod init;

use configure::configure;

use self::configure::ConfigureDetails;

pub fn execute(origin: &str, dotfiles_path: &str) -> anyhow::Result<()> {
    let programs = init::run(origin, dotfiles_path)?;
    let result = configure(programs);
    display_result(result);
    Ok(())
}

fn display_result(result: Vec<ConfigureDetails>) {
    let mut err_msg = String::new();
    let mut not_installed_msg = String::new();

    for r in result {
        match r {
            ConfigureDetails::SomethingWrong(e) => {
                err_msg.push_str(&format!("{}\n", e));
            }
            ConfigureDetails::NotInstalled(p) => {
                not_installed_msg.push_str(&format!("{}\n", p));
            }
            _ => {}
        }
    }

    if !err_msg.is_empty() {
        eprintln!("Some errors occurred:\n{err_msg}")
    }

    if !not_installed_msg.is_empty() {
        println!("The following programs are not installed on this system:\n{not_installed_msg}")
    }
}
