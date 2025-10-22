use std::collections::HashMap;
use std::sync::Arc;
use axum::{
	extract::State,
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
#[serde(rename_all = "camelCase")]
struct Metric {
	tech_detail: &'static str,
	value: f64,
}

#[derive(Serialize)]
struct InfoResponse {
	system: &'static str,
	title: &'static str,
	network_only: bool,
	show_on_homepage: bool,
	ci: InfoCI,
	metrics: HashMap<&'static str, Metric>,
}


pub async fn controller(
	State(data): State<Arc<crate::data::Data>>,
) -> impl IntoResponse {
	let mut headers = HeaderMap::new();
	headers.insert("X-App-Version", HeaderValue::from_static("1.0"));
	let mut metrics = HashMap::new();
	metrics.insert("system-count", Metric {
		tech_detail: "The total number of systems configured",
		value: data.systemCount() as f64,
	});
	metrics.insert("volume-count", Metric {
		tech_detail: "The total number of volumes configured",
		value: data.volumeCount() as f64,
	});
	metrics.insert("host-count", Metric {
		tech_detail: "The total number of hosts configured",
		value: data.hostCount() as f64,
	});
	let json = Json(InfoResponse {
		system: "lucos_configy",
		title: "LucOS Configy",
		ci: InfoCI {
			circle: "gh/lucas42/lucos_configy",
		},
		network_only: true,
		show_on_homepage: false,
		metrics: metrics,
	});
	(StatusCode::OK, headers, json)
}