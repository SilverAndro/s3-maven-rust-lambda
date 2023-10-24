use crate::storage::layers::Layer;

pub fn build_index(req_slice: &Vec<&str>, layer: &Layer) -> String {
	let full_path = req_slice.join("/") + "/";
	let mut builder = string_builder::Builder::new(512);
	builder.append("<!DOCTYPE html>\n");
	builder.append("<html>\n");
	builder.append("	<head>\n");
	builder.append("		<title>Silver's Silly Little Maven</title>\n");
	builder.append("		<meta property=\"og:title\" content=\"Silver's Silly Little Maven\">\n");
	builder.append(format!("		<meta property=\"og:description\" content=\"An index of {full_path} on the maven\">\n"));
	builder.append("		<meta property=\"og:image\" content=\"https://www.silverandro.dev/site_image.png\">\n");
	builder.append("		<meta name=\"theme-color\" content=\"#B00B69\">\n");
	builder.append("	</head>\n");
	builder.append("	<body>\n");
	builder.append(format!("		<h1>Index of {full_path}</h1>\n"));

	if !layer.packages.is_empty() {
		builder.append("		<h3>Packages:</h3>\n");
		builder.append("		<ul>\n");
		for package in &layer.packages {
			builder.append(format!("			<li><a href=\"./{package}/\">{package}/</a></li>\n"))
		}
		builder.append("		</ul>\n");
	}

	if !layer.files.is_empty() {
		builder.append("		<h3>Files:</h3>\n");
		builder.append("		<ul>\n");
		for file in &layer.files {
			builder.append(format!("			<li><a href=\"./{file}\" download>{file}</a></li>\n"))
		}
		builder.append("		</ul>\n");
	}

	builder.append("	  </body>\n");
	builder.append("</html>\n");
	return builder.string().unwrap()
}