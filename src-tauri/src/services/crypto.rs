use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, Generate, Key, KeyInit}
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

#[cfg(test)]
mod tests {
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
}