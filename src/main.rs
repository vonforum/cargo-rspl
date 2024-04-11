use std::error::Error;
use std::path::Path;
use std::process::{Command, Output};
use std::{env, fs, io, path::PathBuf};

use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

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

fn build_file(cmd: &mut Command) -> io::Result<Output> {
	cmd.output()
}

struct Args {
	bin_name: Option<String>,
	data_dir: Option<String>,
	manifest_path: Option<String>,
}

struct SourceData {
	bin_existed: bool,
	bin_name: String,
	in_crate: bool,
	src_existed: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
	let args = parse_args().expect("Failed to parse args");
	let mut cmd = cargo_metadata::MetadataCommand::new();

	if let Some(path) = args.manifest_path {
		cmd.manifest_path(path);
	}

	let mut manifest_path = None;
	let mut crate_name = None;
	let metadata = cmd.exec();

	if let Ok(metadata) = metadata {
		let package = metadata.root_package();
		if let Some(package) = package {
			manifest_path = Some(package.manifest_path.clone());
			crate_name = Some(package.name.clone());
		}
	}

	if let Some(p) = &mut manifest_path {
		p.pop();
	}

	let mut source_data = SourceData {
		bin_existed: false,
		bin_name: args.bin_name.unwrap_or_else(|| {
			if manifest_path.is_some() {
				"_rspl_main_"
			} else {
				"main"
			}
			.to_string()
		}),
		in_crate: manifest_path.is_some(),
		src_existed: false,
	};

	let data_dir = args
		.data_dir
		.map(PathBuf::from)
		.unwrap_or(dirs::data_dir().expect(
			"No data dir on this system. Run again by providing a writable data directory with --data-dir",
		))
		.join("rspl");

	if !data_dir.exists() {
		fs::create_dir_all(&data_dir).expect("Failed to create data directory");
	}

	let mut dir = manifest_path.map(PathBuf::from).unwrap_or(data_dir);
	let crate_path = dir.clone();

	if !source_data.in_crate {
		fs::write(
			&dir.join("Cargo.toml"),
			r#"[package]
name = "_"
version = "0.1.0"
edition = "2021"
"#,
		)
		.expect("Failed to write Cargo.toml to data directory");
	}

	dir.push("src");
	if dir.exists() {
		source_data.src_existed = true;
	} else {
		fs::create_dir_all(&dir).expect("Failed to create src directory");
	}

	dir.push("bin");
	if dir.exists() {
		source_data.bin_existed = true;
	} else {
		fs::create_dir_all(&dir).expect("Failed to create bin directory");
	}

	let mut bin_path = dir.join(&source_data.bin_name);
	bin_path.set_extension("rs");

	if bin_path.exists() {
		return Err(format!(
			"File already exists: {}. Specify a different name with --bin-name",
			bin_path.display()
		)
		.into());
	}

	let crate_prompt = crate_name.unwrap_or("(global)".to_string());

	let mut line_editor = Reedline::create();
	let prompt = DefaultPrompt::new(
		DefaultPromptSegment::Basic(crate_prompt),
		DefaultPromptSegment::CurrentDateTime,
	);

	let mut cmd = build_command(&crate_path, &source_data.bin_name);

	let mut buffer: Vec<String> = vec![];
	loop {
		let sig = line_editor.read_line(&prompt);
		match sig {
			Ok(Signal::Success(line)) => {
				// TODO: Handle let etc. differently
				// TODO: Add commands: help, clear etc.
				buffer.push(line);

				fs::write(
					&bin_path,
					format!(
						r#"
#![allow(warnings)]
fn main() {{
    print!("{{:?}}", {{
        {}
    }});
}}
"#,
						buffer.join(";\n")
					),
				)
				.expect("Failed to write source file");

				match build_file(&mut cmd) {
					Ok(output) => {
						if output.status.success() {
							let output =
								String::from_utf8(output.stdout).expect("Failed to parse output");
							println!("{}", output);
						} else {
							let output =
								String::from_utf8(output.stderr).expect("Failed to parse output");
							eprintln!("{}", output);

							buffer.pop();
						}
					}
					Err(e) => eprintln!("Failed to build: {}", e),
				}
			}
			Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
				println!("\nExiting!");
				break;
			}
			_ => {}
		}
	}

	// Clean up
	if source_data.in_crate {
		let _ = fs::remove_file(&bin_path);

		if !source_data.bin_existed {
			let _ = fs::remove_dir(&dir);
		}

		dir.pop();
		if !source_data.src_existed {
			let _ = fs::remove_dir(&dir);
		}
	}

	Ok(())
}

fn parse_args() -> Result<Args, pico_args::Error> {
	let mut args = pico_args::Arguments::from_env();

	// TODO: Help

	Ok(Args {
		bin_name: args.opt_value_from_str("--bin-name")?,
		data_dir: args.opt_value_from_str("--data-dir")?,
		manifest_path: args.opt_value_from_str("--manifest-path")?,
	})
}
