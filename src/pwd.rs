use libc::{c_int, isatty, tcgetattr, tcsetattr, ECHO, ECHONL, STDIN_FILENO, TCSANOW};
use std::io::{self, Write};
use std::mem::{MaybeUninit};

/// Prompts for a password on STDOUT and reads it from STDIN
pub fn prompt_password(prompt: &str) -> std::io::Result<String> {
    let mut stdout = std::io::stdout();
    write!(stdout, "{}", prompt)?;
    stdout.flush()?;
    read_password()
}

/// Reads a password from stdin
pub fn read_password() -> ::std::io::Result<String> {
    let mut password = String::new();
    let input;
    let tty_fd = STDIN_FILENO;
    let input_is_tty = unsafe { isatty(tty_fd) } == 1;

    if input_is_tty {
        let mut term = unsafe { MaybeUninit::uninit().assume_init() };
        let mut term_orig = unsafe { MaybeUninit::uninit().assume_init() };
        io_result(unsafe { tcgetattr(STDIN_FILENO, &mut term) })?;
        io_result(unsafe { tcgetattr(STDIN_FILENO, &mut term_orig) })?;

        // Hide the password. This is what makes this function useful.
        term.c_lflag &= !ECHO;
        term.c_lflag |= ECHONL;

        // Save the settings for now.
        io_result(unsafe { tcsetattr(tty_fd, TCSANOW, &term) })?;

        // Read the password.
        input = io::stdin().read_line(&mut password);

        // Reset the terminal.
        io_result(unsafe { tcsetattr(tty_fd, TCSANOW, &term_orig) })?;
    } else {
        // If we don't have a TTY, the input was piped so we bypass
        // terminal hiding code
        input = io::stdin().read_line(&mut password);
    }
    if password.ends_with('\n') {
        password.pop();
    }
    input.map(move |_| password)
}

/// Turns a C function return into an IO Result
fn io_result(ret: c_int) -> ::std::io::Result<()> {
    match ret {
        0 => Ok(()),
        _ => Err(::std::io::Error::last_os_error()),
    }
}

