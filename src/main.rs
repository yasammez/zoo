mod daemon;
mod pwd;
mod cmd;
mod path;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_current_dir(path::get_path()?)?;
    unsafe { libc::umask(0o0177); }
    let passwort = pwd::prompt_password("Passwort: ")?;
    let mut cmd = cmd::Cmd::new(passwort)?;
    daemon::daemonize()?;
    cmd.run()
}
