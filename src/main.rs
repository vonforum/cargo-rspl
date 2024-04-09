use std::path::Path;
use std::process::{Command, Output};
use std::{fs, io, path::PathBuf};

use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

#[cfg(target_family = "windows")]
fn build_file(dir: &Path, bin_name: &str) -> Result<Output, io::Error> {
	Command::new("cmd")
		.current_dir(dir)
		.arg("/C")
		.arg(format!(
			"cargo run -q --bin {} -- --cap-lints allow",
			bin_name
		))
		.output()
}

#[cfg(target_family = "unix")]
fn build_file(dir: &Path) -> Result<Output, io::Error> {
	Command::new("sh")
		.current_dir(dir)
		.arg("-c")
		.arg("cargo run -q --bin")
		.arg(bin_name)
		.arg("-- --cap-lints allow")
		.output()
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

fn main() {
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
				"_rspl_bin"
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

	let crate_prompt = crate_name.unwrap_or("(global)".to_string());

	let mut line_editor = Reedline::create();
	let prompt = DefaultPrompt::new(
		DefaultPromptSegment::Basic(crate_prompt),
		DefaultPromptSegment::CurrentDateTime,
	);

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

				match build_file(&crate_path, &source_data.bin_name) {
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
