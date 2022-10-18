use openssl::rsa::{Padding, Rsa};
use openssl::symm::Cipher;

pub struct KeyData {
    pub public: Vec<u8>,
    pub private: Vec<u8>,
    pub passphrase: Vec<u8>,
}

impl KeyData {
    pub fn from_passphrase(passphrase: &[u8]) -> Self {
        let rsa = Rsa::generate(1024).unwrap();

        KeyData {
            private: rsa
                .private_key_to_pem_passphrase(Cipher::aes_128_cbc(), passphrase.into())
                .unwrap(),
            public: rsa.public_key_to_pem().unwrap(),
            passphrase: passphrase.into(),
        }
    }
}

pub fn encrypt(message: &str, key: &Vec<u8>) -> Vec<u8> {
    let rsa = Rsa::public_key_from_pem(key).expect("Could not create RSA from key.");
    let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
    let _ = rsa
        .public_encrypt(message.as_bytes(), &mut buf, Padding::PKCS1)
        .unwrap();
    buf
}

pub fn decrypt(encrypted: Vec<u8>, key: &Vec<u8>, passphrase: &Vec<u8>) -> Vec<u8> {
    let rsa =
        Rsa::private_key_from_pem_passphrase(key, passphrase).expect("Could not generate key");
    let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
    let _ = rsa
        .private_decrypt(&encrypted, &mut buf, Padding::PKCS1)
        .unwrap();
    buf
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::str;

    #[test]
    fn encrypt_decrypt_message() {
        let initial_message = "test message";

        let key = KeyData::from_passphrase("test".as_bytes());
        let encrypted = encrypt(initial_message, &key.public);
        let decrypted_message = decrypt(encrypted, &key.private, &key.passphrase);

        let str_decrypted_message = str::from_utf8(&decrypted_message)
            .expect("Failed to decrypt message.")
            .replace("\x00", "");
        // assert_ne!(initial_message, encrypted_message);
        assert_eq!(initial_message, str_decrypted_message);
    }
}
