mod storage;
mod cfg;
mod responses;
mod util;

use std::cell::RefCell;
use std::sync::Arc;
use http::Method;
use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayV2httpRequest};
use aws_sdk_s3::Client;
use lambda_runtime::{service_fn, Error, LambdaEvent};
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
    let root_layer: Arc<RefCell<Layer>> = Arc::new(RefCell::new(Layer::new()));

    // need to curry together a proper invocation
    lambda_runtime::run(service_fn(|event| {
        handler(event, MavenConfig::new(), &s3_client, &root_layer)
    })).await
}

async fn handler(
    event: LambdaEvent<ApiGatewayV2httpRequest>,
    maven_config: MavenConfig,
    s3_client: &Client,
    bucket_index: &Arc<RefCell<Layer>>
) -> Result<ApiGatewayProxyResponse, Error> {
    // simple access
    let http_method = event.payload.http_method;

    // get a simple string we can work with
    let mut request_path = event.payload.raw_path.unwrap_or_else(|| { String::from("/") });
    // remove first slash
    request_path.remove(0);

    // check if this is an index request
    let is_indexing_request = http_method == Method::GET && !is_file_request(&*request_path);

    tracing::info!("Handling a request for \"{request_path}\" with method {http_method}. Indexing: {is_indexing_request}");

    // return an error if we dont allow indexing
    if is_indexing_request && !maven_config.indexing_enabled {
        return ErrorResponseBuilder::no_index_allowed()
    }

    // build and return an index
    if is_indexing_request {
        return ResponseBuilder::index(s3_client, maven_config, bucket_index, &request_path).await
    }

    // just generate the headers for the request
    // cloudflare converts these to GET requests but
    // no reason we cant add support here
    if http_method == Method::HEAD {
        return ResponseBuilder::resource_head(&request_path)
    }

    // requesting an artifact
    if http_method == Method::GET {
        return ResponseBuilder::resource(&request_path)
    }

    // uploading an artifact
    if http_method == Method::PUT {
        event.payload.headers.get("Authorization");
    }

    // not an allowed method
    return ErrorResponseBuilder::invalid_request_method(http_method)
}