mod vault;

use tokio::net::{UnixListener, UnixStream};
use tokio::io::{BufReader};
use vault::Vault;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::path::Path;
use tokio::prelude::*;

pub struct Cmd {
    vault: Arc<Mutex<Vault>>,
}

impl Cmd {
    pub fn new(passwort: String) -> Result<Cmd, Box<dyn Error>> {
        Ok(Cmd { vault: Arc::new(Mutex::new(vault::Vault::new(passwort)?)) })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let socket = Path::new("socket");

        if socket.exists() {
            std::fs::remove_file(&socket)?;
        }

        // Bind to socket
        let mut incoming = UnixListener::bind(&socket)?.incoming();

        while let Some(stream) = incoming.next().await {
            let vault = self.vault.clone();
            match stream {
                Ok(stream) => tokio::spawn(async move { Self::handle_client(vault, stream).await } ),
                Err(err) => return Err(Box::new(err)),
            };
        }

        Ok(())
    }

    async fn handle_client(vault: Arc<Mutex<Vault>>, stream: UnixStream) {
        if let Err(err) = Self::handle_client_inner(vault, stream).await {
            println!("Fehler beim Handling eines Clients: {}", err);
        }
    }

    async fn handle_client_inner(mut vault: Arc<Mutex<Vault>>, mut stream: UnixStream) -> Result<(), Box<dyn Error>> {
        let (reader, mut writer) = stream.split();
        let reader = BufReader::new(reader);
        let mut lines = reader.lines();
        while let Some(line) = lines.next().await {
            let line = line?;
            let words: Vec<&str> = line.split_whitespace().collect();
            if words.len() == 0 {
                continue;
            }
            if let Some(answer) = match words[0] {
                "put" => Self::handle_put(&mut vault, &words[1..]),
                "get" => Self::handle_get(&mut vault, &words[1..]),
                "del" => Self::handle_del(&mut vault, &words[1..]),
                "lst" => Self::handle_lst(&mut vault),
                "?" => Self::handle_help(),
                "off" => std::process::exit(0),
                _ => None,
            } {
                writer.write_all(answer.as_bytes()).await?;
            }
        }
        Ok(())
    }

    fn handle_put(vault: &mut Arc<Mutex<Vault>>, words: &[&str]) -> Option<String> {
        let mut vault = vault.lock().ok()?;
        if words.len() > 1 {
            vault.put(words[0], words[1]);
        }
        None
    }

    fn handle_get(vault: &mut Arc<Mutex<Vault>>, words: &[&str]) -> Option<String> {
        let vault = vault.lock().ok()?;
        if words.len() > 0 {
            Some(vault.get(words[0]).map(String::from).map_or(String::from("nil\n"), |s| { format!("val {}\n", s) }))
        } else {
            None
        }
    }
    
    fn handle_del(vault: &mut Arc<Mutex<Vault>>, words: &[&str]) -> Option<String> {
        let mut vault = vault.lock().ok()?;
        if words.len() > 0 {
            vault.del(words[0]);
        }
        None
    }

    fn handle_lst(vault: &mut Arc<Mutex<Vault>>) -> Option<String> {
        let vault = vault.lock().ok()?;
        Some(format!("{}\n", vault.lst()))
    }

    fn handle_help() -> Option<String> {
        Some(String::from("put get del off lst ?\n"))
    }

}
