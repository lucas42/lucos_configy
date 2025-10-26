use axum::{
	response::{IntoResponse, Response},
	Json,
	http::header::HeaderMap,
};
use axum_yaml::Yaml;
use std::{error::Error, str::FromStr};

fn negotiate_or_error(headers: &HeaderMap, available_mimes: Vec<mime::Mime>) -> Result<mime::Mime, Box<dyn Error>> {
	headers.get(http::header::ACCEPT)
		.ok_or_else(|| Box::<dyn Error>::from("Missing Accept header"))?
		.to_str()?
		.parse::<accept_header::Accept>()?
		.negotiate(&available_mimes)
		.map_err(|_| "No Match".into())
}

pub fn negotiate_response<T>(headers: &HeaderMap, data: T) -> Response 
	where
		T: std::iter::IntoIterator<Item: serde::Serialize> + serde::Serialize
{
	let application_yaml = mime::Mime::from_str("application/x-yaml").unwrap();
	let available_mimes = vec![mime::APPLICATION_JSON, application_yaml.clone()];
	let mime = negotiate_or_error(headers, available_mimes).unwrap_or(mime::APPLICATION_JSON);

	if mime == mime::APPLICATION_JSON {
		Json(data).into_response()
	} else if mime == application_yaml {
		Yaml(data).into_response()
	} else {
		Json(data).into_response()
	}
}