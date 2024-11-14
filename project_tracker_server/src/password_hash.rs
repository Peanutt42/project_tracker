use serde::{Deserialize, Serialize};
use sha2::Digest;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PasswordHash(Vec<u8>);

impl PasswordHash {
	pub fn new(password: String) -> Self {
		let mut hasher = sha2::Sha512::new();
		hasher.update(password.as_bytes());
		Self(hasher.finalize().to_vec())
	}
}