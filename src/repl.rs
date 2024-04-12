use std::env;
use std::fs::write;
use std::path::{Path, PathBuf};
use std::process::Command;

use reedline::Signal;

const REPL_SOURCE: &'static str = include_str!("repl/_rspl_main_.rs");

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
			Signal::Success(mut line) => {
				if line.starts_with(':') {
					match line.as_str() {
						":q" | ":quit" | ":exit" => return ReplResult::Exit,
						":buffer" => {
							println!("{}", self.buffer.join("\n"));
						}
						":clear" => {
							self.buffer.clear();
						}
						":pop" => {
							self.buffer.pop();
						}
						":h" | ":help" => {
							println!(":q, :quit, :exit - Exit the REPL");
							println!(":buffer - Print the current buffer");
							println!(":clear - Clear the current buffer");
							println!(":pop - Remove the last line from the buffer");
							println!(":h, :help - Print this help message");
						}
						_ => {
							eprintln!(
								"Unknown command: {}. Type :help to show the available commands",
								line
							);
						}
					}

					return ReplResult::Unknown;
				} else {
					if line.starts_with("let ") {
						// Can't return let statements
						line.push(';');
					}

					self.buffer.push(line);
				}

				let joined = self.buffer.join(";\n");
				write(
					&self.bin_path,
					REPL_SOURCE.replace(r#""::_rspl_main_::";"#, &joined),
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
			_ => ReplResult::Exit,
		}
	}
}
