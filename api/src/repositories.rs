use std::sync::Arc;
use axum::{
	extract::{Query, Path, State},
	response::Response,
	http::header::HeaderMap,
};
use crate::conneg::negotiate_response_single;

pub async fn get(
	Path(id): Path<String>,
	State(data): State<Arc<crate::data::Data>>,
	headers: HeaderMap,
	params: Query<crate::conneg::Params>,
) -> Response {
	let item = data.get_repository(&id);
	negotiate_response_single(&headers, params, item)
}
