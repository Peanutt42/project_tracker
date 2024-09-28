use std::{path::Path, process::{Child, Command, Stdio}};

fn spawn_detatched_process(cmd: &mut Command) -> std::io::Result<Child> {
	#[cfg(unix)]
	{
		use std::os::unix::process::CommandExt;
		unsafe {
			cmd.pre_exec(|| {
				libc::setsid();
				Ok(())
			});
		}
	}

	// untested on windows!
	#[cfg(windows)]
	{
		use std::os::windows::process::CommandExt;
		cmd.creation_flags(0x00000200); // CREATE_NEW_PROCESS_GROUP
	}

	cmd.stdout(Stdio::null())
		.stderr(Stdio::null())
		.stdin(Stdio::null());

	cmd.spawn()
}

trait Terminal {
	fn binary(&self) -> &'static str;

	fn args(&self) -> Vec<&'static str>;

	fn to_command(&self, work_dir: &str, mut command_args: Vec<&'static str>) -> Command {
		let mut cmd = Command::new(self.binary());
		let mut args = self.args();
		args.append(&mut command_args);
		cmd.args(&args)
			.current_dir(work_dir);
		cmd
	}
}

struct KonsoleTerminal;

impl Terminal for KonsoleTerminal {
	fn binary(&self) -> &'static str { "konsole" }

	fn args(&self) -> Vec<&'static str> { vec!["-e"] }
}

// not tested
struct XTermTerminal;

impl Terminal for XTermTerminal {
	fn binary(&self) -> &'static str { "xterm" }
	fn args(&self) -> Vec<&'static str> { vec!["-e"] }
}

// not tested
struct GnomeTerminal;

impl Terminal for GnomeTerminal {
	fn binary(&self) -> &'static str { "gnome-terminal" }
	fn args(&self) -> Vec<&'static str> { vec!["--"] }
}

// not tested
struct CmdTerminal;
impl Terminal for CmdTerminal {
	fn binary(&self) -> &'static str { "cmd.exe" }
	fn args(&self) -> Vec<&'static str> { vec!["/C", "start", "cmd", "/K"] }
}

// returns wheter the compilation was successfull and wheter the old main process should close for the new one
pub fn spawn_cargo_run_process() -> bool {
	let work_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
					.parent()
					.and_then(|path| path.to_str())
					.unwrap_or(env!("CARGO_MANIFEST_DIR"));
	let build_args = vec!["cargo", "build", "--release", "--features", "hot_reload"];
	let terminals: [Box<dyn Terminal>; 4] = [
		Box::new(KonsoleTerminal),
		Box::new(XTermTerminal),
		Box::new(GnomeTerminal),
		Box::new(CmdTerminal),
	];

	for terminal in terminals {
		let mut build_cmd = terminal.to_command(work_dir, build_args.clone());
		if let Ok(mut build_process) = spawn_detatched_process(&mut build_cmd) {
			match build_process.wait() {
				Ok(build_status) => {
					if !build_status.success() {
						return false;
					}
				},
				Err(e) => {
					eprintln!("Failed to wait for cargo build command to finish: {e}");
					return false;
				}
			}

			cargo_run(work_dir);

			return true;
		}
	}

	cargo_run(work_dir)
}

fn cargo_run(work_dir: &str) -> bool {
	let mut run_cmd = Command::new("cargo");
	run_cmd.args(["run", "--release", "--features", "hot_reload"])
		.current_dir(work_dir);

	spawn_detatched_process(&mut run_cmd).is_ok()
}