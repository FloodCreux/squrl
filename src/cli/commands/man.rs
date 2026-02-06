use std::path::PathBuf;

#[derive(clap::Args, Debug, Clone)]
pub struct ManCommand {
	#[clap(value_hint = clap::ValueHint::FilePath)]
	pub output_directory: Option<PathBuf>,
}
