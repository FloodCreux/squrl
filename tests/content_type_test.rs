use squrl::models::protocol::http::body::find_file_format_in_content_type;

fn make_headers(pairs: Vec<(&str, &str)>) -> Vec<(String, String)> {
	pairs
		.into_iter()
		.map(|(k, v)| (k.to_string(), v.to_string()))
		.collect()
}

#[test]
fn detects_json_content_type() {
	let headers = make_headers(vec![("content-type", "application/json")]);
	assert_eq!(
		find_file_format_in_content_type(&headers),
		Some("json".to_string())
	);
}

#[test]
fn detects_xml_content_type() {
	let headers = make_headers(vec![("content-type", "application/xml")]);
	assert_eq!(
		find_file_format_in_content_type(&headers),
		Some("xml".to_string())
	);
}

#[test]
fn detects_png_content_type() {
	let headers = make_headers(vec![("content-type", "image/png")]);
	assert_eq!(
		find_file_format_in_content_type(&headers),
		Some("png".to_string())
	);
}

#[test]
fn detects_html_content_type() {
	let headers = make_headers(vec![("content-type", "text/html")]);
	assert_eq!(
		find_file_format_in_content_type(&headers),
		Some("html".to_string())
	);
}

#[test]
fn detects_plain_text_content_type() {
	let headers = make_headers(vec![("content-type", "text/plain")]);
	assert_eq!(
		find_file_format_in_content_type(&headers),
		Some("plain".to_string())
	);
}

#[test]
fn returns_none_when_no_content_type_header() {
	let headers = make_headers(vec![("accept", "application/json")]);
	assert_eq!(find_file_format_in_content_type(&headers), None);
}

#[test]
fn returns_none_for_empty_headers() {
	let headers: Vec<(String, String)> = vec![];
	assert_eq!(find_file_format_in_content_type(&headers), None);
}

#[test]
fn finds_content_type_among_multiple_headers() {
	let headers = make_headers(vec![
		("accept", "text/html"),
		("content-type", "application/json"),
		("authorization", "Bearer token"),
	]);
	assert_eq!(
		find_file_format_in_content_type(&headers),
		Some("json".to_string())
	);
}
