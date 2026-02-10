use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum UrlCommand {
	Get,
	Set {
		#[clap(value_hint = clap::ValueHint::Url)]
		new_url: String,
	},
}
