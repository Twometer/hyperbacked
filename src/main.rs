use passphrase::gen_passphrase;

use crate::{
    backup::{create_backup, recover_backup},
    crypto::Secret,
};

mod backup;
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

    let backup = create_backup(
        secrets,
        backup::BackupConfig {
            num_shares: 7,
            required_shares: 4,
        },
    )?;

    let backup_shares: Vec<&[u8]> = backup.iter().map(|backup| &backup.data[..]).collect();

    // And let's see if we can decode it again
    println!("Decoded 1: {}", recover_backup(&backup_shares, &pass1)?);
    println!("Decoded 2: {}", recover_backup(&backup_shares, &pass2)?);
    println!("Decoded 3: {}", recover_backup(&backup_shares, &pass3)?);

    Ok(())
}
