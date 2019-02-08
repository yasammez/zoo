use ring::aead::*;
use ring::pbkdf2::*;
use ring::rand::{SecureRandom, SystemRandom};
use ring::digest::SHA256;
use std::error::Error;
use std::fmt::Display;

pub fn gen_salt() -> [u8; 16] {
    let mut salt = [0; 16];
    let rand = SystemRandom::new();
    rand.fill(&mut salt).unwrap();
    salt
}

pub struct Crypt {
    key: [u8; 32],
}

impl Crypt {
    pub fn new(passwort: String, salt: [u8; 16]) -> Crypt {
        let mut crypt = Crypt { key: [0; 32] };
        derive(&SHA256, core::num::NonZeroU32::new(100).unwrap(), &salt, passwort.as_bytes(), &mut crypt.key);
        crypt
    }

    pub fn encrypt(&self, mut input: Vec<u8>) -> Vec<u8> {
        input.resize(input.len() + CHACHA20_POLY1305.tag_len(), 0);
        let sealing_key = SealingKey::new(&CHACHA20_POLY1305, &self.key).unwrap();

        // Random data must be used only once per encryption
        let mut nonce = [0; 12];
        let rand = SystemRandom::new();
        rand.fill(&mut nonce).unwrap();
        let sealing_nonce = Nonce::assume_unique_for_key(nonce.clone());

        seal_in_place(
            &sealing_key,
            sealing_nonce,
            Aad::empty(),
            &mut input,
            CHACHA20_POLY1305.tag_len(),
        ).unwrap();
        input.splice(0..0, nonce.iter().cloned());
        input
    }

    pub fn decrypt(&self, mut input: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        let opening_key = OpeningKey::new(&CHACHA20_POLY1305, &self.key)?;
        let nonce = input[0..12].to_vec();
        let nonce = Nonce::try_assume_unique_for_key(&nonce)?;
        let decrypted_data = open_in_place(&opening_key, nonce, Aad::empty(), 12, &mut input).map_err(|_| { CryptError::Decrypt })?;
        let len = decrypted_data.len();
        input.resize(len, 0);
        Ok(input)
    }

}

#[derive(Debug)]
pub enum CryptError {
    Decrypt,
}

impl Display for CryptError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for CryptError {
    fn description(&self) -> &str {
        "CryptError: ist das Passwort korrekt?"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
