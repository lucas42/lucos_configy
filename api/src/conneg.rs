use axum::{
	response::{IntoResponse, Response},
	Json,
	http::StatusCode,
	http::header,
	http::header::HeaderMap,
};
use axum_yaml::Yaml;
use std::str::FromStr;
use mime::Mime;

fn negotiate(headers: &HeaderMap, available_mimes: Vec<Mime>) -> Mime {
	let accept_string = headers.get(http::header::ACCEPT)
		.and_then(|h| h.to_str().ok())
		.unwrap_or("");

	let mut mime_types: Vec<Mime> = accept_string.split(",") // HACK: Technically splitting on comma isn't fully standards-compliant, but it covers the vast majority of cases.
		.filter_map(|mime_str| Mime::from_str(mime_str.trim()).ok())
		.collect();

	mime_types.sort_by(|a, b| {
		let weight_a: f32 = a.get_param("q").and_then(|n| n.as_str().trim().parse::<f32>().ok()).unwrap_or(1.0);
		let weight_b: f32 = b.get_param("q").and_then(|n| n.as_str().trim().parse::<f32>().ok()).unwrap_or(1.0);
		weight_b.partial_cmp(&weight_a).unwrap()
	});

	for target_mime in mime_types {
		for available_mime in available_mimes.clone() {
			match (target_mime.type_(), target_mime.subtype()) {
				(available_type, available_subtype) if available_type == available_mime.type_() && available_subtype == available_mime.subtype() => return target_mime.clone(),
				(mime::STAR, mime::STAR) => return available_mime.clone(),
				(mime::STAR, available_subtype) if available_subtype == available_mime.subtype() => return available_mime.clone(),
				(available_type, mime::STAR) if available_type == available_mime.type_() => return available_mime.clone(),
				(_, _) => continue,
			}
		}
	}

	return mime::APPLICATION_JSON
}

pub fn negotiate_response<T>(headers: &HeaderMap, data: T) -> Response 
	where
		T: std::iter::IntoIterator<Item: serde::Serialize> + serde::Serialize
{
	let available_mimes = vec![
		mime::APPLICATION_JSON,
		Mime::from_str("application/x-yaml").unwrap(),
		Mime::from_str("text/csv").unwrap(),
	];
	let mime = negotiate(headers, available_mimes);
	match mime.essence_str() {
		"application/json" => Json(data).into_response(),
		"application/x-yaml" => Yaml(data).into_response(),
		"text/csv" => {
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
		},
		_ => Json(data).into_response(),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn content_negotiation_should_work() {
		let mut headers = HeaderMap::new();
		headers.insert(http::header::ACCEPT, "application/json, text/html;q=0.9, text/plain;q=0.8, */*;q=0.7".parse().unwrap());

		let available = vec![
			Mime::from_str("text/html").unwrap(),
			Mime::from_str("application/json").unwrap(),
		];
		let negotiated = negotiate(&headers, available);
		assert_eq!(negotiated.essence_str(), "application/json");

		let available = vec![Mime::from_str("application/xml").unwrap()];
		let negotiated = negotiate(&headers, available);
		assert_eq!(negotiated.essence_str(), "application/xml");
	}

	#[test]
	fn mime_parameters_should_be_parsed() {
		let mut headers = HeaderMap::new();
		headers.insert(http::header::ACCEPT, "application/json;q=0.6, text/csv;header=absent;q=0.9, text/plain;q=0.8, */*;q=0.7".parse().unwrap());

		let available = vec![
			Mime::from_str("text/html").unwrap(),
			Mime::from_str("text/csv").unwrap(),
		];
		let negotiated = negotiate(&headers, available);
		assert_eq!(negotiated.essence_str(), "text/csv");

		let header_parameter = negotiated.get_param("header").unwrap().as_str();
		assert_eq!(header_parameter, "absent");
	}

	#[test]
	fn ignore_invalid_mimetypes() {
		let mut headers = HeaderMap::new();
		headers.insert(http::header::ACCEPT, "oneinvalid, text/html;q=0.9, twoinvalid, text/plain;q=0.8, */*;q=0.7".parse().unwrap());

		let available = vec![
			Mime::from_str("text/html").unwrap(),
			Mime::from_str("application/json").unwrap(),
		];
		let negotiated = negotiate(&headers, available);
		assert_eq!(negotiated.essence_str(), "text/html");
	}
	#[test]
	fn no_accept_header_should_return_default() {
		let headers = HeaderMap::new();

		let available = vec![
			Mime::from_str("text/html").unwrap(),
			Mime::from_str("application/json").unwrap(),
		];
		let negotiated = negotiate(&headers, available);
		assert_eq!(negotiated.essence_str(), "application/json");
	}
}