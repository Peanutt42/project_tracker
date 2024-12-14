use std::path::PathBuf;
use std::process::Command;

fn main() {
	let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

	let key_path = out_dir.join("key.pem");
	let cert_path = out_dir.join("cert.pem");

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

	println!("cargo:rerun-if-changed=build.rs");
}
