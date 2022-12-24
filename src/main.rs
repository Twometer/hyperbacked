use std::fs;

use passphrase::gen_passphrase;

use crate::crypto::{decrypt_secret, encrypt_secrets, Secret};

mod crypto;
mod passphrase;

fn main() -> anyhow::Result<()> {
    let pass1 = gen_passphrase(8);
    let pass2 = gen_passphrase(8);
    let pass3 = gen_passphrase(8);

    let secrets = vec![
        Secret {
            value: "Hello world this is my personal secret number one",
            password: &pass1,
        },
        Secret {
            value: "This is a decoy secret",
            password: &pass2,
        },
        Secret {
            value: "This is yet another decoy secret",
            password: &pass3,
        },
    ];

    println!("Passphrase 1: {}", pass1);
    println!("Passphrase 2: {}", pass2);
    println!("Passphrase 3: {}", pass3);

    let secret = encrypt_secrets(secrets)?;
    fs::write("./test.bin", &secret)?;

    println!("Decoded 1: {}", decrypt_secret(&secret, &pass1)?);
    println!("Decoded 2: {}", decrypt_secret(&secret, &pass2)?);
    println!("Decoded 3: {}", decrypt_secret(&secret, &pass3)?);

    Ok(())
}
