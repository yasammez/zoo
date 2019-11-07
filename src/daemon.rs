use std::error::Error;
use std::process::exit;
use std::fmt::{Display};
use std::os::unix::io::AsRawFd;
use std::fs::OpenOptions;

macro_rules! tryret {
    ($expr:expr, $err:expr) => (
        if unsafe { $expr } < 0 {
            Err($err(errno()))
        } else {
            Ok(())
        }
    )
}

pub fn daemonize() -> Result<(), Box<dyn Error>> {
    fork()?;
    tryret!(libc::setsid(), DaemonizeError::Detach)?;
    fork()?;
    redirect_streams()
}

fn fork() -> Result<(), Box<dyn Error>> {
    unsafe {
        match libc::fork() {
            0 => Ok(()),
            e if e < 0 => Err(Box::new(DaemonizeError::Fork(e))),
            _ => exit(0),
        }
    }
}

fn redirect_streams() -> Result<(), Box<dyn Error>> {
    let devnull = OpenOptions::new().read(true).write(true).open("/dev/null")?;
    let log = OpenOptions::new().append(true).create(true).open("log")?;
    tryret!(libc::dup2(devnull.as_raw_fd(), libc::STDIN_FILENO), DaemonizeError::RedirectStreams)?;
    tryret!(libc::dup2(log.as_raw_fd(), libc::STDOUT_FILENO), DaemonizeError::RedirectStreams)?;
    tryret!(libc::dup2(log.as_raw_fd(), libc::STDERR_FILENO), DaemonizeError::RedirectStreams)?;
    Ok(())
}

fn errno() -> libc::c_int {
    std::io::Error::last_os_error().raw_os_error().expect("errno")
}

#[derive(Debug)]
pub enum DaemonizeError {
    Fork(i32),
    RedirectStreams(i32),
    Detach(i32),
}

impl Display for DaemonizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DaemonizeError {
    fn description(&self) -> &str {
        "DaemonizeError"
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}
