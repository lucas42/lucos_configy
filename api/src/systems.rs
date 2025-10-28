use std::sync::Arc;
use axum::{
	extract::{Query, Path, State},
	response::Response,
	http::header::HeaderMap,
};
use crate::conneg::negotiate_response;

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
	let subdomains = data.get_systems_filtered(|system| system.domain.as_ref().is_some_and(|domain| domain.ends_with(&root_domain)));
	negotiate_response(&headers, params, subdomains)
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
