mod vault;

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use vault::Vault;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::fmt::Display;
use std::path::Path;

pub struct Cmd {
    vault: Arc<Mutex<Vault>>,
}

impl Cmd {
    pub fn new(passwort: String) -> Result<Cmd, Box<dyn Error>> {
        Ok(Cmd { vault: Arc::new(Mutex::new(vault::Vault::new(passwort)?)) })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let socket = Path::new("socket");

        if socket.exists() {
            std::fs::remove_file(&socket)?;
        }

        // Bind to socket
        let listener = UnixListener::bind(&socket)?;

        // Iterate over clients, blocks if no client available
        for stream in listener.incoming() {
            let vault = self.vault.clone();
            match stream {
                // TODO: async machen?
                Ok(stream) => std::thread::spawn(move || Self::handle_client(vault, stream)),
                Err(err) => return Err(Box::new(err)),
            };
        }
        Ok(())
    }

    fn handle_client(vault: Arc<Mutex<Vault>>, stream: UnixStream) {
        if let Err(err) = Self::handle_client_inner(vault, stream) {
            println!("Fehler beim Handling eines Clients: {}", err);
        }
    }

    fn handle_client_inner(vault: Arc<Mutex<Vault>>, stream: UnixStream) -> Result<(), Box<dyn Error>> {
        let mut writer = stream.try_clone()?;
        let reader = BufReader::new(&stream);
        for line in reader.lines() {
            let line: String = line?;
            let words: Vec<&str> = line.split_whitespace().collect();
            if words.len() == 0 {
                break;
            }
            let mut vault = vault.lock().map_err(|_| { CmdError::Lock })?;
            if let Some(answer) = match words[0] {
                "put" => Self::handle_put(&mut vault, &words[1..]),
                "get" => Self::handle_get(&mut vault, &words[1..]),
                "del" => Self::handle_del(&mut vault, &words[1..]),
                "lst" => Self::handle_lst(&vault),
                "?" => Self::handle_help(),
                "off" => std::process::exit(0),
                _ => None,
            } {
                writeln!(writer, "{}", answer)?;
            }
        }
        Ok(())
    }

    fn handle_put(vault: &mut Vault, words: &[&str]) -> Option<String> {
        if words.len() > 1 {
            vault.put(words[0], words[1]);
        }
        None
    }

    fn handle_get(vault: &mut Vault, words: &[&str]) -> Option<String> {
        if words.len() > 0 {
            Some(vault.get(words[0]).map(String::from).map_or(String::from("nil"), |s| { format!("val {}", s) }))
        } else {
            None
        }
    }
    
    fn handle_del(vault: &mut Vault, words: &[&str]) -> Option<String> {
        if words.len() > 0 {
            vault.del(words[0]);
        }
        None
    }

    fn handle_lst(vault: &Vault) -> Option<String> {
        Some(vault.lst())
    }

    fn handle_help() -> Option<String> {
        Some(String::from("put get del off lst ?"))
    }

}

#[derive(Debug)]
pub enum CmdError {
    Lock,
}

impl Display for CmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for CmdError {
    fn description(&self) -> &str {
        "CmdError"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
