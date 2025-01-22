use std::marker::PhantomData;

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, Version};
use rand::Rng;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{ServerError, ServerResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encrypted<T: Serialize + DeserializeOwned> {
	encrypted_bytes: Vec<u8>,
	salt: [u8; SALT_LENGTH],
	nonce: [u8; NONCE_LENGTH],
	_phantom_data: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> Encrypted<T> {
	pub fn encrypt(message: &T, password: &str) -> ServerResult<Self> {
		let plaintext =
			bincode::serialize(message).map_err(|_| ServerError::FailedToSerializeToBinary)?;
		let (encrypted_bytes, salt, nonce) =
			encrypt(&plaintext, password).map_err(|_| ServerError::FailedToEncryptBinary)?;
		Ok(Self {
			encrypted_bytes,
			salt,
			nonce,
			_phantom_data: PhantomData,
		})
	}

	pub fn decrypt(&self, password: &str) -> ServerResult<T> {
		let bytes = decrypt(&self.encrypted_bytes, password, &self.salt, &self.nonce)
			.map_err(|_| ServerError::InvalidPassword)?;
		let message: T =
			bincode::deserialize(&bytes).map_err(|_| ServerError::EncryptedContentParseError)?;
		Ok(message)
	}
}

const KEY_LENGTH: usize = 32; // 32 bytes for AES-256
const NONCE_LENGTH: usize = 12; // AES-GCM standard nonce length
const SALT_LENGTH: usize = 16; // Salt length

#[derive(Error, Debug, Clone)]
pub enum EncryptionError {
	#[error("failed to encrypt/decrypt")]
	AesGcm(#[from] aes_gcm::Error),
	#[error("failed to hash password with argon2")]
	Argon2(#[from] argon2::Error),
	#[error("faield to encode salt")]
	FailedToEncodeSalt(#[from] argon2::password_hash::Error),
}

fn derive_key(
	password: &str,
	salt: &[u8; SALT_LENGTH],
) -> Result<[u8; KEY_LENGTH], EncryptionError> {
	let mut output_key = [0u8; KEY_LENGTH];

	// if this default value changes in the future --> rethink the 2 specified in the 'Params'
	static_assertions::const_assert_eq!(Params::DEFAULT_P_COST, 1);

	let argon2 = Argon2::new(
		Algorithm::Argon2id,
		Version::V0x13,
		Params::new(Params::DEFAULT_M_COST, Params::DEFAULT_T_COST, 2, None)?,
	);

	let salt = SaltString::encode_b64(salt)?;

	argon2.hash_password_into(
		password.as_bytes(),
		salt.as_str().as_bytes(),
		&mut output_key,
	)?;

	Ok(output_key)
}

/// returns: Result::Ok is (encrypted_bytes, salt, nonce)
#[allow(clippy::type_complexity)]
fn encrypt(
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

fn decrypt(
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
	use crate::DEFAULT_PASSWORD;

	use super::{decrypt, encrypt, Encrypted};

	const TEST_PASSWORD: &str = DEFAULT_PASSWORD;

	#[test]
	fn test_encrypted_struct() {
		let content = 42_i32;
		let encrypted_content: Encrypted<i32> =
			Encrypted::encrypt(&content, TEST_PASSWORD).unwrap();
		let encrypted_as_binary = bincode::serialize(&encrypted_content).unwrap();
		assert_eq!(
			content,
			bincode::deserialize::<Encrypted<i32>>(&encrypted_as_binary)
				.unwrap()
				.decrypt(TEST_PASSWORD)
				.unwrap()
		);
	}

	#[test]
	fn test_raw_encryption_functions() {
		let plaintext = "This is plaintext. No encryption here.";
		let (encrypted_bytes, salt, nonce) = encrypt(plaintext.as_bytes(), TEST_PASSWORD).unwrap();
		let decrypted_plaintext = decrypt(&encrypted_bytes, TEST_PASSWORD, &salt, &nonce).unwrap();
		assert_eq!(plaintext.as_bytes(), decrypted_plaintext);
	}
}
