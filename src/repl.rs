use std::env;
use std::fs::write;
use std::path::{Path, PathBuf};
use std::process::Command;

use reedline::Signal;

pub enum ReplResult {
	Unknown,
	Success,
	Failure,
	Exit,
}

fn build_command(dir: &Path, bin_name: &str) -> Command {
	let cargo = env::var("CARGO")
		.map(PathBuf::from)
		.ok()
		.unwrap_or_else(|| PathBuf::from("cargo"));

	let mut cmd = Command::new(cargo);
	cmd.current_dir(dir);
	cmd.args(["run", "-q", "--bin", bin_name]);

	cmd
}

pub struct Repl {
	pub bin_path: PathBuf,
	pub buffer: Vec<String>,
	pub command: Command,
}

impl Repl {
	pub fn new(dir: &Path, bin_name: &str, bin_path: PathBuf) -> Self {
		Self {
			bin_path,
			buffer: Vec::new(),
			command: build_command(dir, bin_name),
		}
	}

	pub fn process_signal(&mut self, sig: Signal) -> ReplResult {
		match sig {
			Signal::Success(line) => {
				if line.starts_with(':') {
				} else {
					self.buffer.push(line);
				}

				write(
					&self.bin_path,
					format!(
						r#"
#![allow(warnings)]
fn main() {{
    print!("{{:?}}", {{
        {}
    }});
}}
"#,
						self.buffer.join(";\n")
					),
				)
				.expect("Failed to write source file");

				match self.command.output() {
					Ok(output) => {
						if output.status.success() {
							let output =
								String::from_utf8(output.stdout).expect("Failed to parse output");
							println!("{}", output);

							ReplResult::Success
						} else {
							let output =
								String::from_utf8(output.stderr).expect("Failed to parse output");
							eprintln!("{}", output);

							self.buffer.pop();

							ReplResult::Failure
						}
					}
					Err(e) => {
						eprintln!("Failed to build: {}", e);

						ReplResult::Failure
					}
				}
			}
			Signal::CtrlC | Signal::CtrlD => ReplResult::Exit,
			_ => ReplResult::Unknown,
		}
	}
}
