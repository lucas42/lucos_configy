use std::sync::Arc;
use axum::{
	extract::State,
	response::IntoResponse,
	Json,
};

pub async fn all(
	State(data): State<Arc<crate::data::Data>>,
) -> impl IntoResponse {
	Json(data.get_volumes())
}