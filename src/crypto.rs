use aes_gcm::{
    aead::{generic_array::GenericArray, Aead},
    Aes256Gcm, KeyInit,
};
use bytes::{Buf, BufMut, BytesMut};
use pbkdf2::{password_hash::PasswordHasher, Pbkdf2};
use rand::{thread_rng, Rng, RngCore};

const SALT_LEN: usize = 24;
const NONCE_LEN: usize = 12;
const HEADER_LEN: usize = NONCE_LEN + 20;

#[derive(Clone)]
pub struct Secret<'a> {
    pub value: &'a str,
    pub password: &'a str,
}

struct Header {
    position: usize,
    length: usize,
}

impl Header {
    fn from_bytes(bytes: &mut BytesMut) -> Self {
        let position = bytes.get_u16() as usize;
        let length = bytes.get_u16() as usize;
        Self { position, length }
    }

    fn to_bytes(&self) -> BytesMut {
        let mut output = BytesMut::with_capacity(4);
        output.put_u16(self.position as u16);
        output.put_u16(self.length as u16);
        return output;
    }
}

fn random_bytes(length: usize) -> Vec<u8> {
    let mut rng = thread_rng();
    let mut data = vec![0u8; length];
    rng.fill_bytes(&mut data);
    return data;
}

fn derive_key(password: &str, salt: &[u8]) -> anyhow::Result<Vec<u8>> {
    let salt_str = base64::encode(salt);
    let password_bytes = password.as_bytes();
    let hash = Pbkdf2.hash_password(password_bytes, &salt_str)?.hash;
    let hash_bytes = hash.expect("Password hasher failed").as_bytes().to_owned();
    return Ok(hash_bytes);
}

pub fn encrypt_secrets<'a>(secrets: Vec<Secret<'a>>) -> anyhow::Result<Vec<u8>> {
    let mut rng = thread_rng();

    let salt = random_bytes(SALT_LEN);

    let mut header_buffer = BytesMut::new();
    let mut body_buffer = BytesMut::new();

    let body_offset = SALT_LEN + secrets.len() * HEADER_LEN;

    for secret in secrets {
        let cipher_key = derive_key(secret.password, &salt)?;
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&cipher_key));

        let body_nonce = random_bytes(NONCE_LEN);
        let header_nonce = random_bytes(NONCE_LEN);

        let body_ciphertext = cipher.encrypt(
            GenericArray::from_slice(&body_nonce),
            secret.value.as_bytes(),
        )?;
        let header = Header {
            length: body_ciphertext.len() + body_nonce.len(),
            position: body_buffer.len() + body_offset,
        };
        body_buffer.extend_from_slice(&body_nonce);
        body_buffer.extend_from_slice(&body_ciphertext);

        let header_plaintext = header.to_bytes();
        let header_ciphertext = cipher.encrypt(
            GenericArray::from_slice(&header_nonce),
            &header_plaintext[..],
        )?;
        header_buffer.extend_from_slice(&header_nonce);
        header_buffer.extend_from_slice(&header_ciphertext);
    }

    let mut ciphertext = Vec::<u8>::new();
    ciphertext.extend_from_slice(&salt);
    ciphertext.extend_from_slice(&header_buffer);
    ciphertext.extend_from_slice(&body_buffer);

    let padding_size = rng.gen_range(1..=11);
    let padding = random_bytes(padding_size);
    ciphertext.extend_from_slice(&padding);

    Ok(ciphertext)
}

pub fn decrypt_secret(ciphertext: &[u8], password: &str) -> anyhow::Result<String> {
    let mut ciphertext_buf = BytesMut::from(ciphertext);

    let salt = ciphertext_buf.split_to(SALT_LEN);
    let cipher_key = derive_key(password, &salt[..])?;
    let cipher = Aes256Gcm::new(GenericArray::from_slice(&cipher_key));

    let mut found_headers = Vec::<Header>::new();

    while ciphertext_buf.remaining() > HEADER_LEN {
        let mut header_candidate = ciphertext_buf.split_to(HEADER_LEN);
        let header_nonce = header_candidate.split_to(NONCE_LEN);
        let header_ciphertext = header_candidate;

        let header_plaintext = cipher.decrypt(
            GenericArray::from_slice(&header_nonce),
            &header_ciphertext[..],
        );

        if let Ok(header_plaintext) = header_plaintext {
            let mut header_bytes = BytesMut::from(&header_plaintext[..]);
            let header = Header::from_bytes(&mut header_bytes);
            found_headers.push(header);
        }
    }

    if found_headers.len() != 1 {
        panic!("Invalid number of headers found: {}", found_headers.len())
    }

    let header = &found_headers[0];

    let mut body_ciphertext =
        BytesMut::from(&ciphertext[header.position..header.position + header.length]);

    let nonce = body_ciphertext.split_to(NONCE_LEN);
    let decrypted = cipher.decrypt(GenericArray::from_slice(&nonce), &body_ciphertext[..])?;

    Ok(String::from_utf8(decrypted)?)
}

#[cfg(test)]
mod tests {
    use crate::{crypto::decrypt_secret, passphrase::gen_passphrase};

    use super::{encrypt_secrets, Secret};

    #[test]
    fn round_trip_test() {
        let pass1 = gen_passphrase(8);
        let pass2 = gen_passphrase(8);
        let pass3 = gen_passphrase(8);

        const VALUE1: &'static str = "This is my real secret";
        const VALUE2: &'static str = "This is a fake secret";
        const VALUE3: &'static str = "This is another fake secret";

        let secrets = vec![
            Secret {
                value: &VALUE1,
                password: &pass1,
            },
            Secret {
                value: &VALUE2,
                password: &pass2,
            },
            Secret {
                value: &VALUE3,
                password: &pass3,
            },
        ];

        let ciphertext = encrypt_secrets(secrets).expect("Failed to encrypt");

        let decrypted1 =
            decrypt_secret(&ciphertext, &pass1).expect("Failed to decrypt first secret");
        let decrypted2 =
            decrypt_secret(&ciphertext, &pass2).expect("Failed to decrypt second secret");
        let decrypted3 =
            decrypt_secret(&ciphertext, &pass3).expect("Failed to decrypt third secret");

        assert_eq!(decrypted1, VALUE1);
        assert_eq!(decrypted2, VALUE2);
        assert_eq!(decrypted3, VALUE3);
    }
}
