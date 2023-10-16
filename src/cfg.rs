pub struct MavenConfig {
	pub bucket_name: String,
	pub indexing_enabled: bool,
	pub indexing_max_keys: i32,
}

impl MavenConfig {
	pub fn new() -> MavenConfig {
		return MavenConfig {
			bucket_name: std::env::var("BUCKET_NAME")
				.expect("A BUCKET_NAME must be set in this app's Lambda environment variables."),
			indexing_enabled: std::env::var("INDEXING_ENABLED")
				.unwrap_or_else(|_| { String::from("true") })
				.parse().expect("Failed to read boolean from environment variable INDEXING_ENABLED."),
			indexing_max_keys: std::env::var("INDEXING_MAX_KEYS")
				.unwrap_or_else(|_| { String::from("1000") })
				.parse().expect("Failed to read i32 from environment variable INDEXING_MAX_KEYS.")
		}
	}
}