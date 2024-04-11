use std::error::Error;
use std::{fs, path::PathBuf};

use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline};
use rsepl::{crate_data::CrateData, Repl, ReplResult};

// TODO: Global flag
struct Args {
	bin_name: Option<String>,
	data_dir: Option<String>,
	manifest_path: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
	let args = parse_args().expect("Failed to parse args");

	let data_dir = args
		.data_dir
		.map(PathBuf::from)
		.unwrap_or(dirs::data_dir().expect(
			"No data dir on this system. Run again by providing a writable data directory with --data-dir",
		))
		.join("rspl");

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

	if !data_dir.exists() {
		fs::create_dir_all(&data_dir).expect("Failed to create data directory");
	}

	let mut crate_data =
		CrateData::init(args.bin_name, manifest_path.map(PathBuf::from), data_dir)?;

	let crate_prompt = crate_name.unwrap_or("(global)".to_string());

	let mut line_editor = Reedline::create();
	let prompt = DefaultPrompt::new(
		DefaultPromptSegment::Basic(crate_prompt),
		DefaultPromptSegment::Empty,
	);

	let mut rspl = Repl::new(
		&crate_data.path,
		&crate_data.bin_name,
		crate_data.bin_path.clone(),
	);

	loop {
		let sig = line_editor.read_line(&prompt);
		match sig {
			Ok(sig) => match rspl.process_signal(sig) {
				ReplResult::Exit => break,
				_ => {}
			},
			_ => {}
		}
	}

	crate_data.cleanup();

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
