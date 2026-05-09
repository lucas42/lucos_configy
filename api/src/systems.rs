use std::sync::Arc;
use axum::{
	extract::{Query, Path, State},
	response::Response,
	http::header::HeaderMap,
};
use serde::Serialize;
use crate::conneg::negotiate_response;

#[derive(Serialize, Clone)]
struct SystemWithSubdomainStart {
	#[serde(flatten)]
	system: crate::data::System,
	subdomain_start: Option<String>,
}

pub async fn all(
	State(data): State<Arc<crate::data::Data>>,
	headers: HeaderMap,
	params: Query<crate::conneg::Params>,
) -> Response {
	negotiate_response(&headers, params, data.get_systems())
}

pub async fn subdomain(
	Path(root_domain): Path<String>,
	State(data): State<Arc<crate::data::Data>>,
	headers: HeaderMap,
	params: Query<crate::conneg::Params>,
) -> Response {
	let systems = data.get_systems_filtered(|system| system.domain.as_ref().is_some_and(|domain| domain.ends_with(&root_domain)));
	let systems_with_start: Vec<SystemWithSubdomainStart> = systems
		.into_iter()
		.map(|system| {
			let subdomain_start = system.domain.as_ref()
				.and_then(|domain| domain.strip_suffix(root_domain.as_str()))
				.map(|prefix| prefix.trim_end_matches('.').to_string())
				.filter(|s| !s.is_empty());
			SystemWithSubdomainStart { system, subdomain_start }
		})
		.collect();
	negotiate_response(&headers, params, systems_with_start)
}

pub async fn http(
	State(data): State<Arc<crate::data::Data>>,
	headers: HeaderMap,
	params: Query<crate::conneg::Params>,
) -> Response {
	let http_systems = data.get_systems_filtered(|system| system.http_port.is_some());
	negotiate_response(&headers, params, http_systems)
}

pub async fn host(
	Path(host): Path<String>,
	State(data): State<Arc<crate::data::Data>>,
	headers: HeaderMap,
	params: Query<crate::conneg::Params>,
) -> Response {
	let systems_on_host = data.get_systems_filtered(|system| system.hosts.contains(&host));
	negotiate_response(&headers, params, systems_on_host)
}
