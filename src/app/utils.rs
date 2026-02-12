pub fn to_train_case(text: &String) -> String {
	if text.trim().is_empty() {
		return text.to_owned();
	}

	let mut chars = text.chars();
	let mut new_chars = Vec::new();

	let first_char = chars.next().unwrap();
	new_chars.push(first_char.to_ascii_uppercase());

	while let Some(c) = chars.next() {
		if c == '-' {
			if let Some(next) = chars.next() {
				new_chars.push(c);
				new_chars.push(next.to_ascii_uppercase());
			}
		} else {
			new_chars.push(c);
		}
	}

	new_chars.iter().collect()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_to_train_case_empty_string() {
		let input = String::from("");
		assert_eq!(to_train_case(&input), "");
	}

	#[test]
	fn test_to_train_case_whitespace_only() {
		let input = String::from("   ");
		assert_eq!(to_train_case(&input), "   ");
	}

	#[test]
	fn test_to_train_case_single_word() {
		let input = String::from("hello");
		assert_eq!(to_train_case(&input), "Hello");
	}

	#[test]
	fn test_to_train_case_already_capitalized() {
		let input = String::from("Hello");
		assert_eq!(to_train_case(&input), "Hello");
	}

	#[test]
	fn test_to_train_case_single_hyphen() {
		let input = String::from("content-type");
		assert_eq!(to_train_case(&input), "Content-Type");
	}

	#[test]
	fn test_to_train_case_multiple_hyphens() {
		let input = String::from("x-custom-header");
		assert_eq!(to_train_case(&input), "X-Custom-Header");
	}

	#[test]
	fn test_to_train_case_hyphen_at_end() {
		// When hyphen is at end, there's no next char to capitalize
		// The hyphen is consumed but not added since there's no next char
		let input = String::from("test-");
		assert_eq!(to_train_case(&input), "Test");
	}

	#[test]
	fn test_to_train_case_consecutive_hyphens() {
		// First hyphen consumes the second hyphen as the "next char"
		// The second hyphen gets uppercased (no effect), then 'b' is regular
		let input = String::from("a--b");
		assert_eq!(to_train_case(&input), "A--b");
	}
}
