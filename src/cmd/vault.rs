mod crypt;

use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write, Read};
use std::collections::HashMap;
use std::error::Error;
use crypt::Crypt;
use base64::{encode, decode};

pub struct Vault {
    crypt: Crypt,
    datei: File,
    entries: HashMap<String, String>,
}

impl Vault {
    pub fn new(passwort: String) -> Result<Vault, Box<dyn Error>> {
        let datei = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("vault")?;
        let mut vault = Vault { crypt: Crypt::new(passwort, Self::obtain_salt()?), datei: datei, entries: HashMap::new() };
        vault.read_from_file()?;
        Ok(vault)
    }

    pub fn put(&mut self, key: &str, value: &str) {
        self.entries.insert(key.to_owned(), value.to_owned());
        let _ = self.write_to_file();
    }

    pub fn del(&mut self, key: &str) {
        if let Some(_) = self.entries.remove(key) {
            let _ = self.write_to_file();
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(String::as_str)
    }

    pub fn lst(&self) -> String {
        self.entries.keys().map(String::clone).collect::<Vec<String>>().join(" ")
    }

    fn read_from_file(&mut self) -> Result<(), Box<dyn Error>> {
        self.datei.seek(SeekFrom::Start(0))?;
        let mut buffer = Vec::new();
        self.datei.read_to_end(&mut buffer)?;
        if buffer.len() > 0 {
            self.deserialize(self.crypt.decrypt(buffer)?)?;
        }
        Ok(())
    }

    fn write_to_file(&mut self) -> Result<(), Box<dyn Error>> {
        self.datei.set_len(0)?;
        self.datei.seek(SeekFrom::Start(0))?;
        let serialized = self.serialize();
        self.datei.write_all(&self.crypt.encrypt(serialized))?;
        Ok(())
    }

    fn deserialize(&mut self, input: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let input = String::from_utf8(input)?;
        for line in input.lines() {
            let split: Vec<&str> = line.split(':').collect();
            if split.len() > 1 {
                self.entries.insert(split[0].to_owned(), split[1].to_owned());
            }
        }
        Ok(())
    }

    fn serialize(&mut self) -> Vec<u8> {
        let mut result = Vec::new();
        for (key, val) in &self.entries {
            let _ = writeln!(&mut result, "{}:{}", key, val);
        }
        result
    }

    fn obtain_salt() -> Result<[u8; 16], Box<dyn Error>> {
        let mut buffer = String::new();
        let mut salt = [0; 16];
        let mut datei = OpenOptions::new().read(true).write(true).create(true).open("salt")?;
        datei.read_to_string(&mut buffer)?;
        if buffer.len() == 24 {
            salt.copy_from_slice(&decode(&buffer)?[0..16]);
        } else {
            salt = crypt::gen_salt();
            datei.set_len(0)?;
            datei.seek(SeekFrom::Start(0))?;
            datei.write_all(encode(&salt).as_bytes())?;
        }
        Ok(salt)
    }

}
