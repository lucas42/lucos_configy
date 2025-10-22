use axum::{
	response::IntoResponse,
	Json,
	http::{HeaderMap, HeaderValue, StatusCode},
};
use serde::Serialize;

#[derive(Serialize)]
struct InfoCI {
	circle: &'static str,
}

#[derive(Serialize)]
struct InfoResponse {
	system: &'static str,
	title: &'static str,
	network_only: bool,
	show_on_homepage: bool,
	ci: InfoCI,
}


pub async fn controller() -> impl IntoResponse {
	let mut headers = HeaderMap::new();
	headers.insert("X-App-Version", HeaderValue::from_static("1.0"));
	let json = Json(InfoResponse {
		system: "lucos_configy",
		title: "LucOS Configy",
		ci: InfoCI {
			circle: "gh/lucas42/lucos_configy",
		},
		network_only: true,
		show_on_homepage: false,
	});
	(StatusCode::OK, headers, json)
}