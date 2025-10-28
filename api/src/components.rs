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
	negotiate_response(&headers, params, data.get_components())
}
