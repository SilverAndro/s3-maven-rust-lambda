use std::cell::RefCell;
use std::sync::Arc;
use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use aws_lambda_events::encodings::Body;
use aws_sdk_s3::Client;
use http::{HeaderMap, Method};
use lambda_runtime::Error;
use crate::cfg::MavenConfig;
use crate::responses::http_templates;
use crate::storage;
use crate::storage::layers::Layer;
use crate::util::mime_type;

pub struct ErrorResponseBuilder {}
pub struct ResponseBuilder {}

impl ErrorResponseBuilder {
	pub fn invalid_request_method(http_method: Method) -> Result<ApiGatewayProxyResponse, Error> {
		let mut headers = HeaderMap::new();
		let error_message = format!("Method {http_method} not supported.");
		headers.insert("content-type", "text/html".parse().unwrap());
		let resp = ApiGatewayProxyResponse {
			status_code: 400,
			multi_value_headers: headers.clone(),
			is_base64_encoded: false,
			body: Some(error_message.into()),
			headers,
		};
		Ok(resp)
	}

	pub fn no_index_allowed() -> Result<ApiGatewayProxyResponse, Error> {
		let mut headers = HeaderMap::new();
		headers.insert("content-type", "text/html".parse().unwrap());
		headers.insert("Allow", "".parse().unwrap());
		let resp = ApiGatewayProxyResponse {
			status_code: 405,
			multi_value_headers: headers.clone(),
			is_base64_encoded: false,
			body: Some("Indexing is not enabled for this repository.".into()),
			headers,
		};
		return Ok(resp)
	}

	pub fn no_content() -> Result<ApiGatewayProxyResponse, Error> {
		let mut headers = HeaderMap::new();
		headers.insert("content-type", "text/html".parse().unwrap());
		let resp = ApiGatewayProxyResponse {
			status_code: 404,
			multi_value_headers: headers.clone(),
			is_base64_encoded: false,
			body: Some("No content found.".into()),
			headers,
		};
		return Ok(resp)
	}

	pub fn no_auth() -> Result<ApiGatewayProxyResponse, Error> {
		let mut headers = HeaderMap::new();
		headers.insert("content-type", "text/html".parse().unwrap());
		headers.insert("WWW-Authenticate", "Basic realm=\"Upload Artifact\"".parse().unwrap());
		let resp = ApiGatewayProxyResponse {
			status_code: 401,
			multi_value_headers: headers.clone(),
			is_base64_encoded: false,
			body: Some("No authorization provided.".into()),
			headers,
		};
		return Ok(resp)
	}

	pub fn invalid_auth() -> Result<ApiGatewayProxyResponse, Error> {
		let mut headers = HeaderMap::new();
		headers.insert("content-type", "text/html".parse().unwrap());
		let resp = ApiGatewayProxyResponse {
			status_code: 404,
			multi_value_headers: headers.clone(),
			is_base64_encoded: false,
			body: Some("Invalid authorization provided.".into()),
			headers,
		};
		return Ok(resp)
	}
}

impl ResponseBuilder {
	pub fn resource_head(request_path: &String) -> Result<ApiGatewayProxyResponse, Error> {
		todo!()
	}

	pub async fn resource(s3_client: &Client, maven_config: MavenConfig, request_path: &String) -> Result<ApiGatewayProxyResponse, Error> {
		let resource = storage::get_resource(s3_client, maven_config, request_path).await;
		return match resource {
			None => {
				ErrorResponseBuilder::no_content()
			}

			Some(bytes) => {
				let mut headers = HeaderMap::new();
				let content_type = mime_type(request_path);
				headers.insert("content-type", content_type.parse().unwrap());
				headers.insert("Cache-Control", "public, max-age=86400".parse().unwrap());
				let resp = ApiGatewayProxyResponse {
					status_code: 200,
					multi_value_headers: headers.clone(),
					is_base64_encoded: false,
					body: Some(Body::Binary(bytes.to_vec())),
					headers,
				};
				Ok(resp)
			}
		}
	}

	pub async fn index<'a>(s3_client: &Client, maven_config: MavenConfig, root_layer: &'a Arc<RefCell<Layer>>, request_path: &String) -> Result<ApiGatewayProxyResponse, Error> {
		let root = &mut root_layer.borrow_mut();
		let layer = storage::get_index(s3_client, maven_config, root, request_path).await;

		if layer.is_none() {
			return ErrorResponseBuilder::no_content()
		}

		let mut headers = HeaderMap::new();
		headers.insert("content-type", "text/html".parse().unwrap());
		headers.insert("Cache-Control", "public, max-age=28800".parse().unwrap());
		let resp = ApiGatewayProxyResponse {
			status_code: 200,
			multi_value_headers: headers.clone(),
			is_base64_encoded: false,
			body: Some(http_templates::build_index(
				&request_path.split('/').filter(|it| { !it.is_empty() }).collect(),
				layer.unwrap()
			).into()),
			headers,
		};
		return Ok(resp)
	}
}