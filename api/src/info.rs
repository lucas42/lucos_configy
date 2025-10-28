use std::collections::HashMap;
use std::sync::Arc;
use axum::{
	extract::State,
	response::IntoResponse,
	Json,
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
	value: u8,
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
	let mut metrics = HashMap::new();
	metrics.insert("system-count", Metric {
		tech_detail: "The total number of systems configured",
		value: data.system_count() as u8,
	});
	metrics.insert("volume-count", Metric {
		tech_detail: "The total number of volumes configured",
		value: data.volume_count() as u8,
	});
	metrics.insert("host-count", Metric {
		tech_detail: "The total number of hosts configured",
		value: data.host_count() as u8,
	});
	Json(InfoResponse {
		system: "lucos_configy",
		title: "LucOS Configy",
		ci: InfoCI {
			circle: "gh/lucas42/lucos_configy",
		},
		network_only: true,
		show_on_homepage: false,
		metrics: metrics,
	})
}