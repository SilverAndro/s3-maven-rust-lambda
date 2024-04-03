pub struct MavenConfig {
	pub bucket_name: String,

	pub indexing_enabled: bool,
	pub indexing_max_keys: i32,

	pub max_artifact_size: i64,

	pub username: String,
	pub password: String
}

impl MavenConfig {
	pub fn new() -> MavenConfig {
		MavenConfig {
			bucket_name: std::env::var("BUCKET_NAME")
				.expect("A BUCKET_NAME must be set in this app's Lambda environment variables."),

			indexing_enabled: std::env::var("INDEXING_ENABLED")
				.unwrap_or_else(|_| { String::from("true") })
				.parse().expect("Failed to read boolean from environment variable INDEXING_ENABLED."),
			indexing_max_keys: std::env::var("INDEXING_MAX_KEYS")
				.unwrap_or_else(|_| { String::from("1000") })
				.parse().expect("Failed to read i32 from environment variable INDEXING_MAX_KEYS."),

			max_artifact_size: std::env::var("MAX_ARTIFACT_SIZE")
				.unwrap_or_else(|_| { String::from("5900000") })
				.parse().expect("Failed to read i64 from environment variable MAX_ARTIFACT_SIZE."),

			username: std::env::var("UPLOAD_USERNAME")
				.expect("Failed to get username from UPLOAD_USERNAME for uploading artifacts."),
			password: std::env::var("UPLOAD_PASSWORD")
				.expect("Failed to get password from UPLOAD_PASSWORD for uploading artifacts.")
		}
	}
}
