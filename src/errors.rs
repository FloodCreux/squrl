use ratatui::prelude::Stylize;
use std::fmt::Display;
use std::process::exit;

pub fn panic_error<T>(message: T) -> !
where
	T: Display,
{
	eprintln!("{error}:\n\t{message}", error = "Error".red().bold());
	exit(1);
}
