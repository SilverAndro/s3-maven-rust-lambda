use lambda_http::Response;
use lambda_runtime::Error;
use once_cell::sync::Lazy;
use regex::Regex;

pub fn is_file_request(haystack: &str) -> bool {
	static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r".+\.(pom|jar|\w+)").unwrap());
	RE.is_match(haystack)
}

pub fn mime_type(resource_path: &str) -> String {
	let split = resource_path.rsplit_once('.');
	match split {
		None => { return String::from("text/plain") }
		Some(splice) => {
			let postfix = splice.1;

			match postfix {
				"jar" => { String::from("application/java-archive") }
				"pom" => { String::from("application/xml") }
				_ => { String::from("text/plain") }
			}
		}
	}
}

pub fn simple_response(status_code: u16, msg: &str) -> Result<Response<Vec<u8>>, Error> {
	let resp = Response::builder()
		.status(status_code)
		.header("content-type", "text/html")
		.body(msg.as_bytes().to_vec())
		.map_err(Box::new)?;
	return Ok(resp)
}

pub fn simple_response_fmt(status_code: u16, msg: String) -> Result<Response<Vec<u8>>, Error> {
	let resp = Response::builder()
		.status(status_code)
		.header("content-type", "text/html")
		.body(msg.as_bytes().to_vec())
		.map_err(Box::new)?;
	return Ok(resp)
}