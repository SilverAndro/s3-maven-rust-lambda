pub mod layers;

use std::cell::RefMut;
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::AggregatedBytes;
use crate::cfg::MavenConfig;
use crate::storage::layers::Layer;
use crate::util::is_file_request;

pub async fn get_resource<'a>(s3_client: &Client, maven_config: MavenConfig, request_path: &String) -> Option<AggregatedBytes> {
	tracing::info!("Getting object \"{request_path}\"");
	let obj = s3_client.get_object()
		.bucket(maven_config.bucket_name)
		.key(request_path)
		.send().await;

	return match obj {
		Err(..) => {
			None
		}

		Ok(result) => {
			let collected = result.body.collect()
				.await.expect("Failed to collect object bytes");
			Some(collected)
		}
	}
}
pub async fn get_index<'a>(s3_client: &Client, maven_config: MavenConfig, root_layer: &'a mut RefMut<'_, Layer>, request_path: &String) -> Option<&'a Layer> {
	let path_prefix = request_path.rsplit_once('/').unwrap_or_else(|| { ("", "") }).0;
	let request_split: Vec<&str> = request_path.split('/').filter(|it| { !it.is_empty() }).collect();

	if !request_split.is_empty() && root_layer.has_children(&request_split, 0) {
		tracing::info!("Index for \"{path_prefix}\" already exists, returning our cache");
		return Some(root_layer.descend(&request_split, 0));
	}

	tracing::info!("Getting index for \"{path_prefix}\"");

	if path_prefix.is_empty() {
		let list = s3_client.list_objects_v2()
			.bucket(maven_config.bucket_name)
			.max_keys(maven_config.indexing_max_keys)
			.delimiter('/')
			.send().await
			.expect("Failed to get bucket contents, did you setup the permissions properly?");

		let prefixes = list.common_prefixes.expect("Did not receive common prefixes from bucket.");

		if prefixes.is_empty() {
			tracing::info!("Found no prefixes");
			return None
		}

		for obj in prefixes {
			let key = obj.prefix.unwrap();
			let splice: Vec<&str> = key.split('/').filter(|it| { !it.is_empty() }).collect();
			root_layer.populate(&splice, 0);
		}
	} else {
		let list = s3_client.list_objects_v2()
			.bucket(maven_config.bucket_name)
			.max_keys(maven_config.indexing_max_keys)
			.prefix(path_prefix)
			.send().await
			.expect("Failed to get bucket contents, did you setup the permissions properly?");

		let content = list.contents.expect("Did not receive contents from bucket");

		if content.is_empty() {
			tracing::info!("Found no content");
			return None
		}

		for obj in content {
			let key = obj.key.unwrap();
			let mut splice: Vec<&str> = key.split('/').filter(|it| { !it.is_empty() }).collect();
			let last = splice.remove(splice.len() - 1);
			let layer = root_layer.populate(&splice, 0);

			if is_file_request(last) {
				layer.files.push(String::from(last));
				layer.files.dedup()
			}
		}
	}

	if root_layer.has_children(&request_split, 0) {
		return Some(root_layer.descend(&request_split, 0));
	} else {
		return None
	}
}