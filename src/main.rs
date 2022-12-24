use std::fs;

use passphrase::gen_passphrase;
use sharks::{Share, Sharks};

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

    let sharks = Sharks(4); // min required 4
    let dealer = sharks.dealer(&secret);

    // Create 7 shares
    let shares: Vec<Share> = dealer.take(7).collect();
    let mut ctr = 0;
    for share in &shares {
        let sharebin = Vec::from(share);
        ctr += 1;
        fs::write(format!("test-share-{}.bin", ctr), sharebin)?;
    }

    // Recover the secret from 4 shares
    let secret = sharks.recover(&shares[0..=3]).unwrap();

    // And let's see if we can decode it again
    println!("Decoded 1: {}", decrypt_secret(&secret, &pass1)?);
    println!("Decoded 2: {}", decrypt_secret(&secret, &pass2)?);
    println!("Decoded 3: {}", decrypt_secret(&secret, &pass3)?);

    Ok(())
}
