use std::sync::Arc;
use axum::{
	extract::{Query, State},
	response::Response,
	http::header::HeaderMap,
};
use crate::conneg::negotiate_response;

pub async fn all(
	State(data): State<Arc<crate::data::Data>>,
	headers: HeaderMap,
	params: Query<crate::conneg::Params>,
) -> Response {
	negotiate_response(&headers, params, data.get_hosts())
}

pub async fn http(
	State(data): State<Arc<crate::data::Data>>,
	headers: HeaderMap,
	params: Query<crate::conneg::Params>,
) -> Response {
	let http_hosts = data.get_hosts_filtered(|host| host.serves_http);
	negotiate_response(&headers, params, http_hosts)
}