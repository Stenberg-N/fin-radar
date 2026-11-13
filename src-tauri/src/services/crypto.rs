use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, Generate, Key, KeyInit, Payload}
};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngExt;
use sqlx::FromRow;
use log::error;

pub(crate) const PARAMS_LEN: usize = 12;
pub(crate) const KEY_LEN: usize = 32;
pub(crate) const NONCE_LEN: usize = 24;
pub(crate) const SALT_LEN: usize = 16;
pub(crate) const TAG_LEN: usize = 16;
pub(crate) const WRAPPED_DEK_LEN: usize = NONCE_LEN + KEY_LEN + TAG_LEN;

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub(crate) enum FormatVersion {
    /// Xchacha20poly1305
    V1 = 1,
}

impl FormatVersion {
    fn to_vec(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

impl std::fmt::Display for FormatVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, FromRow, PartialEq, Debug)]
pub(crate) struct KdfParams {
    /// memory size in 1 KiB blocks.
    pub(crate) m_cost: u32,
    /// number of iterations.
    pub(crate) t_cost: u32,
    /// degree of parallelism.
    pub(crate) p_cost: u32,
}

impl KdfParams {
    pub(crate) const DEFAULT: Self = Self { m_cost: 64 * 1024, t_cost: 3, p_cost: 1 };

    fn validate(&self) -> bool {
        (8..=1024 * 1024).contains(&self.m_cost) &&
        (1..=20).contains(&self.t_cost) &&
        (1..=16).contains(&self.p_cost)
    }

    /// Converts the KDF parameters into raw bytes.
    /// - First four bytes is the memory cost.
    /// - Bytes from 4 to 8 are the iterations.
    /// - Bytes from 8 to 12 is the degree of parallelism.
    pub(crate) fn to_bytes(&self) -> [u8; PARAMS_LEN] {
        let mut b = [0u8; PARAMS_LEN];
        b[0..4].copy_from_slice(&self.m_cost.to_le_bytes());
        b[4..8].copy_from_slice(&self.t_cost.to_le_bytes());
        b[8..12].copy_from_slice(&self.p_cost.to_le_bytes());
        b
    }

    pub(crate) fn from_bytes(b: [u8; PARAMS_LEN]) -> Self {
        Self {
            m_cost: u32::from_le_bytes(b[0..4].try_into().unwrap()),
            t_cost: u32::from_le_bytes(b[4..8].try_into().unwrap()),
            p_cost: u32::from_le_bytes(b[8..12].try_into().unwrap())
        }
    }

    pub(crate) fn argon2(&self) -> Result<Argon2<'static>, argon2::Error> {
        let params = Params::new(self.m_cost, self.t_cost, self.p_cost, Some(32))?;
        Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
    }
}

pub(crate) fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    rand::rng().fill(&mut salt);
    salt
}

/// Derives the KEK from the user's password to encrypt the DEK
pub(crate) fn derive_kek(password: &[u8], kek_salt: &[u8; SALT_LEN], p: &KdfParams) -> Result<[u8; KEY_LEN], String> {
    if !p.validate() {
        return Err("Too little memory".to_string());
    }

    let mut kek = [0u8; KEY_LEN];
    p.argon2().map_err(|e| {
        error!("Argon2 initialization failed during KEK derivation: {:#?}", e);
        "An error occurred".to_string()
    })?
    .hash_password_into(password, kek_salt, &mut kek).map_err(|e| {
        error!("Failed to create KEK hash: {:#?}", e);    
        "An error occurred".to_string()
    })?;

    Ok(kek)
}

pub(crate) fn generate_dek() -> [u8; KEY_LEN] {
    Key::<XChaCha20Poly1305>::generate().into()
}

/// Encrypts the DEK and returns it as a BLOB. 24-byte nonce + 32-byte DEK + 16-byte tag
pub(crate) fn wrap_dek(kek: &[u8; KEY_LEN], dek: &[u8; KEY_LEN]) -> Result<[u8; WRAPPED_DEK_LEN], String> {
    let nonce = XNonce::generate();
    let cipher = XChaCha20Poly1305::new(&Key::<XChaCha20Poly1305>::from(*kek));
    let wrapped_dek = cipher.encrypt(&nonce, dek.as_ref()).map_err(|e| {
        error!("DEK encryption failed: {:#?}", e);
        "An error occurred".to_string()
    })?;

    let mut out = [0u8; WRAPPED_DEK_LEN];
    out[..NONCE_LEN].copy_from_slice(&nonce);
    out[NONCE_LEN..].copy_from_slice(&wrapped_dek);

    Ok(out)
}

/// Decrypts and returns the DEK
pub(crate) fn unwrap_dek(kek: &[u8; KEY_LEN], wrapped_dek: &[u8; WRAPPED_DEK_LEN]) -> Result<[u8; KEY_LEN], String> {
    let (nonce_bytes, ciphertext) = wrapped_dek.split_at(NONCE_LEN);
    let nonce = XNonce::try_from(nonce_bytes).map_err(|e| {
        error!("Failed to parse nonce during DEK unwrapping: {:#?}", e);
        "An error occurred".to_string()
    })?;
    let cipher = XChaCha20Poly1305::new(&Key::<XChaCha20Poly1305>::from(*kek));

    let dek = cipher.decrypt(&nonce, ciphertext).map_err(|_| {
        error!("DEK decryption failed");
        "An error occurred".to_string()
    })?;
    dek.try_into().map_err(|_| {
        error!("Failed to parse DEK during decryption");
        "An error occurred".to_string()
    })
}

/// Splits the BLOB to its parts and returns them as a tuple: `(version, nonce, ciphertext)`
fn split_blob(blob: &[u8]) -> Result<(u8, &[u8], &[u8]), String> {
    if blob.len() < 25 { return Err("BLOB corrupt".to_string()) }

    let version = blob[0];
    let nonce = &blob[1..25];
    let ciphertext = &blob[25..];

    Ok((version, nonce, ciphertext))
}

/// Generates an AAD for usage in `encrypt` and `decrypt`.
/// # Arguments
/// - `version`: The FormatVersion enum. Determines which cipher was used on the data.
/// - `label`: A string identifier to add to the AAD to prevent pasting data to database columns they do not belong to.
/// - `user_id`: The user's id.
/// - `data_id`: The primary key of the data.
fn generate_aad(version: FormatVersion, label: &str, user_id: i64, data_id: i64) -> Vec<u8> {
    let mut aad = version.to_vec();
    aad.extend_from_slice(label.as_bytes());
    aad.extend_from_slice(&user_id.to_le_bytes());
    aad.extend_from_slice(&data_id.to_le_bytes());
    aad
}

/// Encrypt data into a BLOB. BLOB schema: `[1 byte version][24 byte nonce][ciphertext + 16 byte tag]`
/// # Arguments
/// - `dek`: The user's data encryption key.
/// - `content`: The content to be encrypted.
/// - `label`: An identifier to be supplied to AAD to prevent pasting the encrypted data to database columns to which it doesn't belong to.
///   - Example identifiers: "notes.title" or "notes.content"
/// - `user_id`: The user's id.
/// - `data_id`: The primary key used for the data row in the database.
pub(crate) fn encrypt(dek: &[u8; KEY_LEN], content: &[u8], label: &str, user_id: i64, data_id: i64) -> Result<Vec<u8>, String> {
    let aad = generate_aad(FormatVersion::V1, label, user_id, data_id);
    let cipher = XChaCha20Poly1305::new(dek.into());
    let nonce = XNonce::generate();
    let ciphertext = cipher.encrypt(&nonce, Payload { msg: content, aad: &aad }).map_err(|e| {
        error!("Failed to encrypt content: {:#?}", e);
        "An error occurred".to_string()
    })?;

    let mut out = FormatVersion::V1.to_vec();
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);

    Ok(out)
}

/// Decrypt data from a BLOB. BLOB schema: `[1 byte version][24 byte nonce][ciphertext + 16 byte tag]`
/// # Arguments
/// - `dek`: The user's data encryption key.
/// - `content`: The content to be encrypted.
/// - `label`: An identifier to be supplied to AAD to prevent pasting the encrypted data to database columns to which it doesn't belong to.
///   - Example identifiers: "notes.title" or "notes.content"
/// - `user_id`: The user's id.
/// - `data_id`: The primary key used for the data row in the database.
pub(crate) fn decrypt(dek: &[u8; KEY_LEN], blob: &[u8], label: &str, user_id: i64, data_id: i64) -> Result<Vec<u8>, String> {
    let (version, nonce, ciphertext) = split_blob(blob)?;
    if version != 1 { return Err("Invalid format version".to_string()) }

    let aad = generate_aad(FormatVersion::V1, label, user_id, data_id);
    let cipher = XChaCha20Poly1305::new(dek.into());

    let data = cipher.decrypt(
        nonce.try_into().map_err(|_| "Failed to parse nonce".to_string())?,
        Payload { msg: ciphertext, aad: &aad }
    ).map_err(|e| {
        error!("Failed to decrypt content: {:#?}", e);
        "An error occurred".to_string()
    })?;

    Ok(data)
}

#[cfg(test)]
mod tests {
    use crate::models::transaction::{Transaction, TransactionPayload, TransactionPayloadRef};
    use super::*;

    /// Verifies the wrapped DEK layout: 24-byte nonce + 32-byte DEK + 16-byte tag = 72 bytes.
    #[test]
    fn wrapped_dek_length() -> Result<(), Box<dyn std::error::Error>> {
        let pw = "test password".as_bytes();
        let kek_salt = generate_salt();
        let params = KdfParams::DEFAULT;
        let kek = derive_kek(pw, &kek_salt, &params)?;
        let dek = generate_dek();
        let wrapped_dek = wrap_dek(&kek, &dek)?;

        assert_eq!(wrapped_dek.len(), WRAPPED_DEK_LEN);
        Ok(())
    }

    /// Unwrapping a wrapped DEK recovers the original.
    #[test]
    fn test_unwrap_dek() -> Result<(), Box<dyn std::error::Error>> {
        let pw = "test password".as_bytes();
        let kek_salt = generate_salt();
        let params = KdfParams::DEFAULT;
        let kek = derive_kek(pw, &kek_salt, &params)?;
        let dek = generate_dek();
        let wrapped_dek = wrap_dek(&kek, &dek)?;
        let unwrapped_dek = unwrap_dek(&kek, &wrapped_dek)?;

        assert_eq!(dek, unwrapped_dek);
        Ok(())
    }

    /// Round-trip test: serializing the KDF parameters to bytes and deserializing back
    /// should yield the original parameters.
    #[test]
    fn test_to_and_from_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let params = KdfParams::DEFAULT;
        let params_bytes = params.to_bytes();
        let params_orig = KdfParams::from_bytes(params_bytes);

        assert_eq!(params, params_orig);
        Ok(())
    }

    /// Decrypting encrypted data recovers the original data.
    #[test]
    fn test_encrypt_decrypt() -> Result<(), Box<dyn std::error::Error>> {
        let pw = "test password".as_bytes();
        let kek_salt = generate_salt();
        let params = KdfParams::DEFAULT;
        let kek = derive_kek(pw, &kek_salt, &params)?;
        let dek = generate_dek();
        let wrapped_dek = wrap_dek(&kek, &dek)?;
        let unwrapped_dek = unwrap_dek(&kek, &wrapped_dek)?;

        let transaction = Transaction {
            id: 1,
            user_id: 1,
            category: "rent".into(),
            date: "2026-01-01".into(),
            description: "test".into(),
            amount: 100.00,
            _type: "expense".into()
        };
        let payload = serde_json::to_vec(&TransactionPayloadRef {
            category: &transaction.category,
            description: &transaction.description,
            amount: transaction.amount,
            _type: &transaction._type,
        })?;

        let wrapped_blob = encrypt(&unwrapped_dek, &payload, "transactions.payload", transaction.user_id, transaction.id)?;
        let unwrapped_blob = decrypt(&unwrapped_dek, &wrapped_blob, "transactions.payload", transaction.user_id, transaction.id)?;

        let reconstructed_payload: TransactionPayload = serde_json::from_slice(&unwrapped_blob)?;
        let reconstructed_transaction = Transaction {
            id: transaction.id,
            user_id: transaction.user_id,
            date: transaction.date.clone(),
            category: reconstructed_payload.category,
            description: reconstructed_payload.description,
            amount: reconstructed_payload.amount,
            _type: reconstructed_payload._type
        };

        assert_eq!(transaction, reconstructed_transaction);

        Ok(())
    }

    /// Decrypting encrypted data with an invalid DEK errors.
    #[test]
    fn test_invalid_dek_decryption() -> Result<(), Box<dyn std::error::Error>> {
        let pw = "test password".as_bytes();
        let kek_salt = generate_salt();
        let params = KdfParams::DEFAULT;
        let kek = derive_kek(pw, &kek_salt, &params)?;
        let dek = generate_dek();
        let wrapped_dek = wrap_dek(&kek, &dek)?;
        let unwrapped_dek = unwrap_dek(&kek, &wrapped_dek)?;

        let transaction = Transaction {
            id: 1,
            user_id: 1,
            category: "rent".into(),
            date: "2026-01-01".into(),
            description: "test".into(),
            amount: 100.00,
            _type: "expense".into()
        };
        let payload = serde_json::to_vec(&TransactionPayloadRef {
            category: &transaction.category,
            description: &transaction.description,
            amount: transaction.amount,
            _type: &transaction._type,
        })?;

        let wrapped_blob = encrypt(&unwrapped_dek, &payload, "transactions.payload", transaction.user_id, transaction.id)?;
        let unwrapped_blob = decrypt(&[1; 32], &wrapped_blob, "transactions.payload", transaction.user_id, transaction.id);
        
        assert!(unwrapped_blob.is_err());

        Ok(())
    }
}