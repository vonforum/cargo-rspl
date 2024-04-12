use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub struct CrateData {
	pub bin_path: PathBuf,
	pub bin_existed: bool,
	pub bin_name: String,
	pub in_crate: bool,
	pub path: PathBuf,
	pub src_existed: bool,
}

impl CrateData {
	pub fn init(
		bin_name: Option<String>,
		manifest_path: Option<PathBuf>,
		data_dir: PathBuf,
	) -> Result<Self, Box<dyn Error>> {
		let in_crate = manifest_path.is_some();
		let dir = manifest_path.unwrap_or(data_dir);
		let path = dir.clone();

		let mut data = CrateData {
			bin_path: dir,
			bin_existed: false,
			bin_name: bin_name
				.unwrap_or_else(|| if in_crate { "_rspl_main_" } else { "main" }.to_string()),
			in_crate,
			path,
			src_existed: false,
		};

		if !data.in_crate {
			fs::write(
				&data.path.join("Cargo.toml"),
				r#"[package]
name = "_"
version = "0.1.0"
edition = "2021"
"#,
			)
			.expect("Failed to write Cargo.toml to data directory");
		}

		data.bin_path.push("src");
		if data.bin_path.exists() {
			data.src_existed = true;
		} else {
			fs::create_dir_all(&data.bin_path).expect("Failed to create src directory");
		}

		data.bin_path.push("bin");
		if data.bin_path.exists() {
			data.bin_existed = true;
		} else {
			fs::create_dir_all(&data.bin_path).expect("Failed to create bin directory");
		}

		data.bin_path.push(&data.bin_name);
		data.bin_path.set_extension("rs");

		if data.in_crate && data.bin_path.exists() {
			return Err(format!(
				"File already exists: {}. Specify a different name with --bin-name",
				data.bin_path.display()
			)
			.into());
		}

		Ok(data)
	}

	pub fn cleanup(&mut self) {
		if self.in_crate {
			let _ = fs::remove_file(&self.bin_path);

			self.bin_path.pop();
			if !self.bin_existed {
				let _ = fs::remove_dir(&self.bin_path);
			}

			self.bin_path.pop();
			if !self.src_existed {
				let _ = fs::remove_dir(&self.bin_path);
			}
		}
	}
}
