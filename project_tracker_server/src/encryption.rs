use aes_gcm::{Aes256Gcm, Error, Key, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use pbkdf2::pbkdf2_hmac;
use rand::Rng;
use sha2::Sha256;

const PBKDF2_ITERATIONS: u32 = 100_000; // Increase for more security in production
const KEY_LENGTH: usize = 32;           // 32 bytes for AES-256
pub const NONCE_LENGTH: usize = 12;         // AES-GCM standard nonce length
pub const SALT_LENGTH: usize = 16;          // Salt length

fn derive_key(password: &str, salt: &[u8; SALT_LENGTH]) -> [u8; KEY_LENGTH] {
	let mut key = [0u8; KEY_LENGTH];
	pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
	key
}

fn generate_salt() -> [u8; SALT_LENGTH] {
	let mut salt = [0u8; SALT_LENGTH];
	rand::thread_rng().fill(&mut salt);
	salt
}

fn generate_nonce() -> [u8; NONCE_LENGTH] {
	let mut nonce = [0u8; NONCE_LENGTH];
	rand::thread_rng().fill(&mut nonce);
	nonce
}

#[allow(clippy::type_complexity)]
pub fn encrypt(content: &[u8], password: &str)
	-> Result<(Vec<u8>, [u8; SALT_LENGTH], [u8; NONCE_LENGTH]), Error>
{
	let salt = generate_salt();
	let key = derive_key(password, &salt);
	let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));

	let nonce = generate_nonce();
	let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), content)?;
	Ok((ciphertext, salt, nonce))
}

pub fn decrypt(encrypted: &[u8], password: &str, salt: &[u8; SALT_LENGTH], nonce: &[u8; NONCE_LENGTH])
	-> Result<Vec<u8>, Error>
{
	let key = derive_key(password, salt);
	let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
	let nonce = Nonce::from_slice(nonce);

	cipher.decrypt(nonce, encrypted)
}