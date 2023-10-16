use once_cell::sync::Lazy;
use regex::Regex;

pub fn is_file_request(haystack: &str) -> bool {
	static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r".+\.(pom|jar|\w+)").unwrap());
	RE.is_match(haystack)
}