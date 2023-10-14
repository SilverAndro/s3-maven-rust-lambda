pub mod layers;

use std::cell::RefMut;
use aws_sdk_s3::Client;
use crate::cfg::MavenConfig;
use crate::storage::layers::Layer;
use crate::util::is_file_request;

pub async fn get_index<'a>(s3_client: &Client, maven_config: MavenConfig, root_layer: &'a mut RefMut<'_, Layer>, request_path: &String) -> Option<&'a Layer> {
	let path_prefix = request_path.rsplit_once('/').unwrap_or_else(|| { ("", "") }).0;
	let request_split = request_path.split('/').filter(|it| { !it.is_empty() }).collect();

	if root_layer.has_children(&request_split, 0) {
		return Some(root_layer.descend(&request_split, 0));
	}

	let list = s3_client.list_objects_v2()
		.bucket(maven_config.bucket_name)
		.max_keys(i32::MAX)
		.prefix(path_prefix)
		.send().await
		.expect("Failed to get bucket contents, did you not setup the permissions properly?");

	let content = list.contents.expect("Did not receive contents from bucket");

	if content.is_empty() {
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

	return Some(root_layer.descend(&request_split, 0));
}