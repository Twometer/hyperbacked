use sharks::{Share, Sharks};

use crate::crypto::{decrypt_secret, encrypt_secrets, Secret};

pub struct BackupConfig {
    pub num_shares: u8,
    pub required_shares: u8,
}

pub struct BackupShare {
    pub number: usize,
    pub data: Vec<u8>,
}

pub fn create_backup(
    secrets: Vec<Secret>,
    config: BackupConfig,
) -> anyhow::Result<Vec<BackupShare>> {
    let ciphertext = encrypt_secrets(secrets)?;

    // Shamir secret sharing
    let sharks = Sharks(config.required_shares);
    let dealer = sharks.dealer(&ciphertext);
    let mut shares = Vec::<BackupShare>::new();

    for (index, share) in dealer.take(config.num_shares as usize).enumerate() {
        let share_data = Vec::from(&share);
        shares.push(BackupShare {
            number: index + 1,
            data: share_data,
        })
    }

    Ok(shares)
}

pub fn recover_backup(shares: &[&[u8]], password: &str) -> anyhow::Result<String> {
    let sharks = Sharks(shares.len() as u8);

    let shares_decoded: Vec<Share> = shares
        .iter()
        .filter_map(|share| Share::try_from(&share[..]).ok())
        .collect();

    let ciphertext = sharks.recover(&shares_decoded[..]).unwrap();
    decrypt_secret(&ciphertext, password)
}
