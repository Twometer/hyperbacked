use sharks::{Share, Sharks};

use crate::{
    crypto::{decrypt_secret, encrypt_secrets, Secret},
    errors::BackupError,
};

pub struct BackupConfig {
    pub total_shards: u8,
    pub min_shards: u8,
}

#[derive(Clone, Debug)]
pub struct BackupShard {
    pub number: usize,
    pub data: Vec<u8>,
}

pub fn create_backup(
    secrets: Vec<Secret>,
    config: BackupConfig,
) -> anyhow::Result<Vec<BackupShard>> {
    let ciphertext = encrypt_secrets(secrets)?;

    // Split ciphertext into shards using Shamir's secret sharing (Sharks)
    let sharks = Sharks(config.min_shards);
    let dealer = sharks.dealer(&ciphertext);
    let mut shards = Vec::<BackupShard>::new();

    for (index, share) in dealer.take(config.total_shards as usize).enumerate() {
        shards.push(BackupShard {
            number: index + 1,
            data: Vec::from(&share),
        });
    }

    Ok(shards)
}

pub fn recover_backup(shards: &[Vec<u8>], password: &str) -> anyhow::Result<String> {
    let sharks = Sharks(shards.len() as u8);

    let shares_decoded: Vec<Share> = shards
        .iter()
        .filter_map(|shard| Share::try_from(&shard[..]).ok())
        .collect();

    let ciphertext = sharks
        .recover(&shares_decoded[..])
        .map_err(|e| BackupError::SharksError(e.to_owned()))?;
    decrypt_secret(&ciphertext, password)
}
