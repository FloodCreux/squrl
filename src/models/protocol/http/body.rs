use rayon::prelude::*;
use regex::Regex;

pub fn find_file_format_in_content_type(headers: &Vec<(String, String)>) -> Option<String> {
    if let Some((_, content_type)) = headers
        .par_iter()
        .find_any(|(header, _)| *header == "content_type")
    {
        let regex = Regex::new(r"\w+/(?<file_format>\w+").unwrap();

        return match regex.captures(content_type) {
            None => None,
            Some(capture) => Some(capture["file_format"].to_string()),
        };
    } else {
        return None;
    }
}
