use std::sync::Arc;
use axum::{
	extract::{Path, State},
	response::IntoResponse,
	Json,
};

pub async fn all(
	State(data): State<Arc<crate::data::Data>>,
) -> impl IntoResponse {
	Json(data.get_systems())
}

pub async fn subdomain(
	Path(root_domain): Path<String>,
	State(data): State<Arc<crate::data::Data>>,
) -> impl IntoResponse {
	let filtered_systems: Vec<crate::data::System> = data.get_systems()
		.iter()
		.filter(|system| system.domain.as_ref().is_some_and(|domain| domain.ends_with(&root_domain)))
		.cloned()
		.collect();
	Json(filtered_systems)
}

pub async fn http(
	State(data): State<Arc<crate::data::Data>>,
) -> impl IntoResponse {
	let filtered_systems: Vec<crate::data::System> = data.get_systems()
		.iter()
		.filter(|system| system.http_port.is_some())
		.cloned()
		.collect();
	Json(filtered_systems)
}

pub async fn host(
	Path(host): Path<String>,
	State(data): State<Arc<crate::data::Data>>,
) -> impl IntoResponse {
	let filtered_systems: Vec<crate::data::System> = data.get_systems()
		.iter()
		.filter(|system| system.hosts.contains(&host))
		.cloned()
		.collect();
	Json(filtered_systems)
}