use std::sync::{Arc, LazyLock};

use ratatui::prelude::Color;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxDefinition, SyntaxReference, SyntaxSet, SyntaxSetBuilder};

#[derive(Default)]
pub struct SyntaxHighlighting {
	pub highlighted_body: Option<Vec<Line<'static>>>,
	pub highlighted_console_output: Vec<Line<'static>>,
}

pub static SYNTAX_SET: LazyLock<Arc<SyntaxSet>> =
	LazyLock::new(|| Arc::new(SyntaxSet::load_defaults_newlines()));
pub static ENV_VARIABLE_SYNTAX_SET: LazyLock<Arc<SyntaxSet>> =
	LazyLock::new(|| Arc::new(generate_env_variable_syntax_set()));
pub static ENV_VARIABLE_SYNTAX_REF: LazyLock<&'static SyntaxReference> = LazyLock::new(|| {
	ENV_VARIABLE_SYNTAX_SET
		.syntaxes()
		.first()
		.expect("env variable syntax set should have at least one syntax")
});
pub static JSON_SYNTAX_REF: LazyLock<&'static SyntaxReference> = LazyLock::new(|| {
	SYNTAX_SET
		.find_syntax_by_extension("json")
		.expect("json syntax should be in default syntax set")
});
pub static XML_SYNTAX_REF: LazyLock<&'static SyntaxReference> = LazyLock::new(|| {
	SYNTAX_SET
		.find_syntax_by_extension("xml")
		.expect("xml syntax should be in default syntax set")
});
pub static HTML_SYNTAX_REF: LazyLock<&'static SyntaxReference> = LazyLock::new(|| {
	SYNTAX_SET
		.find_syntax_by_extension("html")
		.expect("html syntax should be in default syntax set")
});
pub static JS_SYNTAX_REF: LazyLock<&'static SyntaxReference> = LazyLock::new(|| {
	SYNTAX_SET
		.find_syntax_by_extension("js")
		.expect("js syntax should be in default syntax set")
});
pub static THEME_SET: LazyLock<Arc<ThemeSet>> =
	LazyLock::new(|| Arc::new(ThemeSet::load_defaults()));
pub static SYNTAX_THEME: LazyLock<&'static Theme> =
	LazyLock::new(|| &THEME_SET.themes["base16-ocean.dark"]);

pub fn highlight(string: &str, extension: &str) -> Option<Vec<Line<'static>>> {
	let syntax = match extension {
		"json" => *JSON_SYNTAX_REF,
		"xml" => *HTML_SYNTAX_REF,
		"html" => *XML_SYNTAX_REF,
		"js" => *JS_SYNTAX_REF,
		_ => SYNTAX_SET.find_syntax_by_extension(extension)?,
	};

	let mut highlight = HighlightLines::new(syntax, &SYNTAX_THEME);

	let mut lines: Vec<Line> = vec![];

	for line in string.lines() {
		let result = highlight
			.highlight_line(line, &SYNTAX_SET)
			.expect("syntax highlighting should not fail for valid syntax");

		let mut highlighted_line: Vec<Span> = vec![];

		for &(ref style, text) in result.iter() {
			highlighted_line.push(Span::raw(text.to_string()).fg(Color::Rgb(
				style.foreground.r,
				style.foreground.g,
				style.foreground.b,
			)));
		}

		lines.push(Line::from(highlighted_line));
	}

	Some(lines)
}

fn generate_env_variable_syntax_set() -> SyntaxSet {
	let mut syntax_set_builder = SyntaxSetBuilder::new();

	let syntax_def = SyntaxDefinition::load_from_str(
		r#"%YAML 1.2
---
name: Double Brace Variables
file_extensions:
  - dblvars
scope: source.dblvars

contexts:
  main:
    - match: '\{\{[A-Za-z0-9_-]+\}\}'
      scope: variable"#,
		true,
		None,
	)
	.expect("env variable syntax definition should be valid YAML");

	syntax_set_builder.add(syntax_def);

	syntax_set_builder.build()
}
