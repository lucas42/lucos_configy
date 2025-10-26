use std::sync::Arc;
use axum::{
	extract::State,
};
use axum_codec::{
	response::IntoCodecResponse,
	Codec,
};

pub async fn all(
	State(data): State<Arc<crate::data::Data>>,
) -> impl IntoCodecResponse {
	Codec(data.get_volumes())
}