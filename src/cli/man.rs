use std::env;
use std::fs;

use clap::CommandFactory;

use crate::cli::args::{ARGS, Args};

pub fn generate_man_pages() -> anyhow::Result<()> {
	let man = clap_mangen::Man::new(Args::command());
	let mut buffer: Vec<u8> = vec![];

	man.render(&mut buffer)?;

	let path = match &ARGS.directory {
		None => &env::current_dir()?,
		Some(path) => path,
	};

	fs::write(path.join("squrl.1"), buffer)?;

	println!("Man page generated into \"{}\"", path.display());

	Ok(())
}
