use std::sync::Arc;
use axum::{
	extract::State,
	response::Response,
	http::header::HeaderMap,
};
use crate::conneg::negotiate_response;

pub async fn all(
	State(data): State<Arc<crate::data::Data>>,
	headers: HeaderMap,
) -> Response {
	negotiate_response(&headers, data.get_volumes())
}
