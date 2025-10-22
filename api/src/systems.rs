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
	Json(data.get_systems_filtered(|system| system.domain.as_ref().is_some_and(|domain| domain.ends_with(&root_domain))))
}

pub async fn http(
	State(data): State<Arc<crate::data::Data>>,
) -> impl IntoResponse {
	Json(data.get_systems_filtered(|system| system.http_port.is_some()))
}

pub async fn host(
	Path(host): Path<String>,
	State(data): State<Arc<crate::data::Data>>,
) -> impl IntoResponse {
	Json(data.get_systems_filtered(|system| system.hosts.contains(&host)))
}