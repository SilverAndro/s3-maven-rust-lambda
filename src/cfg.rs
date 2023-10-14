pub struct MavenConfig {
	pub bucket_name: String,
	pub indexing_enabled: bool
}

impl MavenConfig {
	pub fn new() -> MavenConfig {
		return MavenConfig {
			bucket_name: std::env::var("BUCKET_NAME")
				.expect("A BUCKET_NAME must be set in this app's Lambda environment variables."),
			indexing_enabled: std::env::var("ENABLE_INDEXING")
				.unwrap_or_else(|_| { String::from("true") })
				.parse().expect("Failed to read boolean from environment variable ENABLE_INDEXING.")
		}
	}
}