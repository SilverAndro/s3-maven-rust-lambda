use std::sync::{Arc, Mutex};
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::DateTimeFormat;
use http::Method;
use lambda_http::{Body, Response};
use lambda_runtime::Error;
use crate::cfg::MavenConfig;
use crate::responses::http_templates;
use crate::storage;
use crate::storage::layers::Layer;
use crate::util::{mime_type, simple_response, simple_response_fmt};

pub struct ErrorResponseBuilder {}
pub struct ResponseBuilder {}

// I originally did this because it made more sense to my OOP brain but somehow it breaks the
// lambda_http::run in main if you try to move these out so they sit in an empty impl
impl ErrorResponseBuilder {
	pub fn server_error(message: &str) -> Result<Response<Body>, Error> {
		simple_response(500, message)
	}

	pub fn invalid_request() -> Result<Response<Body>, Error> {
		simple_response(400, "Invalid request.")
	}

	pub fn invalid_request_method(http_method: Method) -> Result<Response<Body>, Error> {
		simple_response_fmt(400, format!("Method {http_method} not supported."))
	}

	pub fn no_index_allowed() -> Result<Response<Body>, Error> {
		let resp = Response::builder()
			.status(405)
			.header("content-type", "text/html")
			.header("Allow", "")
			.body(Body::Text(String::from("Indexing is not enabled for this repository.")))
			.map_err(Box::new)?;
		Ok(resp)
	}

	pub fn no_content() -> Result<Response<Body>, Error> {
		simple_response(404, "No content found.")
	}

	pub fn no_content_bytes() -> Result<Response<Body>, Error> {
		let resp = Response::builder()
			.status(404)
			.header("content-type", "text/html")
			.body(Body::Empty)
			.map_err(Box::new)?;
		Ok(resp)
	}

	pub fn no_auth() -> Result<Response<Body>, Error> {
		let resp = Response::builder()
			.status(401)
			.header("content-type", "text/html")
			.header("WWW-Authenticate", "Basic realm=\"Upload Artifact\"")
			.body(Body::Text(String::from("No authorization provided.")))
			.map_err(Box::new)?;
		Ok(resp)
	}

	pub fn invalid_auth() -> Result<Response<Body>, Error> {
		simple_response(403, "Invalid authorization provided.")
	}

	pub fn invalid_content_length() -> Result<Response<Body>, Error> {
		simple_response(411, "No content-length provided.")
	}

	pub fn too_large(maven_config: &MavenConfig) -> Result<Response<Body>, Error> {
		simple_response_fmt(413, format!("Artifact too large. Max size: {}", maven_config.max_artifact_size))
	}
}

impl ResponseBuilder {
	pub async fn resource_head(s3_client: &Client, maven_config: MavenConfig, request_path: &String) -> Result<Response<Body>, Error> {
		tracing::info!("Getting object head \"{request_path}\"");
		let obj = s3_client.head_object()
			.bucket(maven_config.bucket_name)
			.key(request_path)
			.send().await;

		match obj {
			Err(_) => { ErrorResponseBuilder::no_content() }
			Ok(data) => {
				let content_type = mime_type(request_path);

				let resp = Response::builder()
					.status(200)
					.header("content-type", content_type)
					.header("Cache-Control", "public, max-age=259200")
					.header("Last-Modified", data.last_modified.unwrap().fmt(DateTimeFormat::HttpDate).unwrap())
					.header("Content-Length", data.content_length)
					.body(Body::Empty)
					.map_err(Box::new)?;
				Ok(resp)
			}
		}
	}

	pub async fn resource(s3_client: &Client, maven_config: MavenConfig, request_path: &String) -> Result<Response<Body>, Error> {
		let resource = storage::get_resource(s3_client, maven_config, request_path).await;
		match resource {
			None => {
				ErrorResponseBuilder::no_content_bytes()
			}

			Some(data) => {
				let bytes = data.body.collect().await.expect("Failed to collect object bytes");
				let content_type = mime_type(request_path);

				let resp = Response::builder()
					.status(200)
					.header("content-type", content_type)
					.header("Cache-Control", "public, max-age=259200")
					.header("Last-Modified", data.last_modified.unwrap().fmt(DateTimeFormat::HttpDate).unwrap())
					.header("Content-Length", data.content_length)
					.body(Body::Binary(bytes.to_vec()))
					.map_err(Box::new)?;
				Ok(resp)
			}
		}
	}

	pub async fn index<'a>(s3_client: &Client, maven_config: MavenConfig, root_layer: &Arc<Mutex<Layer>>, request_path: &str) -> Result<Response<Body>, Error> {
		let layer = storage::get_index(s3_client, maven_config, root_layer, request_path).await;

		if layer.is_none() {
			return ErrorResponseBuilder::no_content()
		}

		let resp = Response::builder()
			.status(200)
			.header("content-type", "text/html")
			.header("Cache-Control", "public, max-age=43200")
			.body(Body::Text(http_templates::build_index(
				&request_path.split('/').filter(|it| { !it.is_empty() }).collect(),
				&layer.unwrap())))
			.map_err(Box::new)?;
		Ok(resp)
	}

	pub fn uploaded_artifact() -> Result<Response<Body>, Error> {
		let resp = Response::builder()
			.status(201)
			.body(Body::Empty)
			.map_err(Box::new)?;
		Ok(resp)
	}
}