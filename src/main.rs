mod storage;
mod cfg;
mod responses;
mod util;

use std::sync::{Arc, Mutex};
use http::{Method, Response};
use aws_sdk_s3::Client;
use data_encoding::BASE64;
use lambda_http::request::RequestContext;
use lambda_http::{Request, RequestExt};
use lambda_runtime::{service_fn, Error};
use crate::responses::build_response::{ResponseBuilder, ErrorResponseBuilder};
use crate::cfg::MavenConfig;
use crate::storage::layers::Layer;
use crate::util::is_file_request;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let config = aws_config::load_from_env().await;
    let s3_client = Client::new(&config);
    let root_layer: Arc<Mutex<Layer>> = Arc::new(Mutex::new(Layer::new()));

    // need to curry together a proper invocation
    lambda_http::run(service_fn(|event| {
        Ok(handler(event, MavenConfig::new(), &s3_client, &root_layer))
    })).await
}

async fn handler(
    event: Request,
    maven_config: MavenConfig,
    s3_client: &Client,
    bucket_index: &Arc<Mutex<Layer>>
) -> Response<Vec<u8>> {
    let raw_context = event.request_context();

    match raw_context {
        RequestContext::ApiGatewayV1(_) => { panic!("Cannot handle api v1") }
        RequestContext::Alb(_) => { panic!("Cannot handle alb") }
        RequestContext::WebSocket(_) => { panic!("Cannot handle websocket") }
        RequestContext::ApiGatewayV2(context) => {
            // simple access
            let http_method = context.http.method;

            // get a simple string we can work with
            let mut request_path = String::from(event.raw_http_path());
            // remove first slash
            request_path.remove(0);

            // check if this is an index request
            let is_indexing_request = http_method == Method::GET && !is_file_request(&*request_path);

            tracing::info!("Handling a request for \"{request_path}\" with method {http_method}. Indexing: {is_indexing_request}");

            // return an error if we dont allow indexing
            if is_indexing_request && !maven_config.indexing_enabled {
                return ErrorResponseBuilder::no_index_allowed().unwrap()
            }

            // build and return an index
            if is_indexing_request {
                return ResponseBuilder::index(s3_client, maven_config, bucket_index, &request_path).await.unwrap()
            }

            // just generate the headers for the request
            // cloudflare converts these to GET requests but
            // no reason we cant add support here
            if http_method == Method::HEAD {
                return ResponseBuilder::resource_head(s3_client, maven_config, &request_path).await.unwrap()
            }

            // requesting an artifact
            if http_method == Method::GET {
                return ResponseBuilder::resource(s3_client, maven_config, &request_path).await.unwrap()
            }

            // uploading an artifact
            if http_method == Method::PUT {
                if request_path.is_empty() { return ErrorResponseBuilder::invalid_request().unwrap() }

                let auth_header = event.headers().get("Authorization");
                return match auth_header {
                    None => { ErrorResponseBuilder::no_auth().unwrap() }
                    Some(encoded) => {
                        let skip = "Basic ".len();
                        let extracted = &encoded.as_bytes()[skip..];
                        let decoded = BASE64.decode(extracted);
                        match decoded {
                            Err(err) => {
                                tracing::warn!("Failed to decode {err}");
                                ErrorResponseBuilder::invalid_auth().unwrap()
                            }
                            Ok(value) => {
                                let decoded_str = String::from_utf8(value)
                                    .unwrap_or(String::from("invalid:invalid"));
                                if !decoded_str.contains(':') { return ErrorResponseBuilder::invalid_auth().unwrap() }

                                let (username, password) = decoded_str.rsplit_once(':')
                                    .expect("Failed to split after checking delimiter exists");

                                if username != maven_config.username || password != maven_config.password {
                                    return ErrorResponseBuilder::invalid_auth().unwrap()
                                }

                                let size_header = event.headers().get("content-length");
                                let size: i64 = match size_header {
                                    None => { return ErrorResponseBuilder::invalid_content_length().unwrap() }
                                    Some(data) => {
                                        let length = data.to_str();
                                        match length {
                                            Err(_) => { return ErrorResponseBuilder::invalid_content_length().unwrap() }
                                            Ok(data) => { data.parse().unwrap() }
                                        }
                                    }
                                };

                                if size > maven_config.max_artifact_size { return ErrorResponseBuilder::too_large(&maven_config).unwrap() }

                                return storage::upload_artifact(s3_client, maven_config, &request_path).await.unwrap()
                            }
                        }
                    }
                }
            }

            // not an allowed method
            return ErrorResponseBuilder::invalid_request_method(http_method).unwrap()
        }
    }
}