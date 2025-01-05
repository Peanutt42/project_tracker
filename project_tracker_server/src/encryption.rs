use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, Version};
use rand::Rng;
use thiserror::Error;

const KEY_LENGTH: usize = 32; // 32 bytes for AES-256
pub const NONCE_LENGTH: usize = 12; // AES-GCM standard nonce length
pub const SALT_LENGTH: usize = 16; // Salt length

#[derive(Error, Debug, Clone)]
pub enum EncryptionError {
	#[error("failed to encrypt/decrypt")]
	AesGcm(#[from] aes_gcm::Error),
	#[error("failed to hash password with argon2")]
	Argon2(#[from] argon2::Error),
}

fn derive_key(password: &str, salt: &[u8; SALT_LENGTH]) -> argon2::Result<[u8; KEY_LENGTH]> {
	let mut output_key = [0u8; KEY_LENGTH];

	// if this default value changes in the future --> rethink the 2 specified in the 'Params'
	static_assertions::const_assert_eq!(Params::DEFAULT_P_COST, 1);

	let argon2 = Argon2::new(
		Algorithm::Argon2id,
		Version::V0x13,
		Params::new(Params::DEFAULT_M_COST, Params::DEFAULT_T_COST, 2, None)?,
	);

	let salt = SaltString::encode_b64(salt).unwrap();

	argon2.hash_password_into(
		password.as_bytes(),
		salt.as_str().as_bytes(),
		&mut output_key,
	)?;

	Ok(output_key)
}

#[allow(clippy::type_complexity)]
pub fn encrypt(
	content: &[u8],
	password: &str,
) -> Result<(Vec<u8>, [u8; SALT_LENGTH], [u8; NONCE_LENGTH]), EncryptionError> {
	let mut salt = [0u8; SALT_LENGTH];
	rand::thread_rng().fill(&mut salt);
	let mut nonce = [0u8; NONCE_LENGTH];
	rand::thread_rng().fill(&mut nonce);

	let key = derive_key(password, &salt)?;
	let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));

	let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), content)?;
	Ok((ciphertext, salt, nonce))
}

pub fn decrypt(
	encrypted: &[u8],
	password: &str,
	salt: &[u8; SALT_LENGTH],
	nonce: &[u8; NONCE_LENGTH],
) -> Result<Vec<u8>, EncryptionError> {
	let key = derive_key(password, salt)?;
	let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
	let nonce = Nonce::from_slice(nonce);

	cipher.decrypt(nonce, encrypted).map_err(|e| e.into())
}
