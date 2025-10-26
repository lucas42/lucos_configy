use axum::{
	response::{IntoResponse, Response},
	Json,
	http::StatusCode,
	http::header,
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
	let text_csv = mime::Mime::from_str("text/csv").unwrap();
	let available_mimes = vec![mime::APPLICATION_JSON, application_yaml.clone(), text_csv.clone()];
	let mime = negotiate_or_error(headers, available_mimes).unwrap_or(mime::APPLICATION_JSON);

	if mime == mime::APPLICATION_JSON {
		Json(data).into_response()
	} else if mime == application_yaml {
		Yaml(data).into_response()
	} else if mime == text_csv {
		// accept_header doesn't currently set parameters on mime objects, but if it did, this would handle the "header" parameter
		let print_csv_header = mime.get_param("header").map(|n| n.as_str()).unwrap_or("present") != "absent";
		let mut w = csv::WriterBuilder::new().has_headers(print_csv_header).from_writer(vec![]);
		for record in data {
			w.serialize(record).unwrap();
		}
		let csv_output = String::from_utf8(w.into_inner().unwrap()).unwrap();
		return Response::builder()
			.status(StatusCode::OK)
			.header(header::CONTENT_TYPE, "text/csv")
			.body(csv_output.into())
			.unwrap()
	} else {
		Json(data).into_response()
	}
}