use std::path::PathBuf;
use std::process::Command;

fn main() {
	if std::env::var("CARGO_FEATURE_WEB_SERVER").is_err() {
		return;
	}

	let certificates_output_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
		.join("src")
		.join("web_server")
		.join("self_signed_certificates");

	let key_path = certificates_output_dir.join("key.pem");
	let cert_path = certificates_output_dir.join("cert.pem");

	if !key_path.exists() && !cert_path.exists() {
		std::fs::create_dir(&certificates_output_dir).unwrap();

		let output = Command::new("openssl")
			.args([
				"req",
				"-x509",
				"-newkey", "rsa:4096",
				"-keyout", &key_path.to_string_lossy(),
				"-out", &cert_path.to_string_lossy(),
				"-days", "365",
				"-nodes",
				"-subj", "/CN=localhost",
			])
			.output()
			.expect("Failed to execute 'openssl'");

		if !output.status.success() {
			panic!(
				"OpenSSL command failed: {}",
				String::from_utf8_lossy(&output.stderr)
			);
		}
	}

	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed={}", certificates_output_dir.display());
}
