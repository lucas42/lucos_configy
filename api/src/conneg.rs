use axum::{
	response::{IntoResponse, Response},
	Json,
	http::StatusCode,
	http::header,
	http::header::HeaderMap,
	extract::Query,
};
use axum_yaml::Yaml;
use std::str::FromStr;
use std::collections::HashSet;
use mime::Mime;
use serde_json::{Value, Map};
use serde::Deserialize;

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

#[derive(Deserialize)]
pub struct Params {
	pub fields: Option<String>,
}

fn filter_fields(value: Value, allowed: &Option<HashSet<String>>) -> Value {
	if let Some(fields) = allowed {
		match value {
			Value::Object(map) => {
				let filtered = map
					.into_iter()
					.filter(|(k, _)| fields.contains(k))
					.map(|(k, v)| (k, filter_fields(v, allowed)))
					.collect::<Map<_, _>>();
				Value::Object(filtered)
			}
			Value::Array(vec) => Value::Array(
				vec.into_iter()
					.map(|v| filter_fields(v, allowed))
					.collect(),
			),
			other => other,
		}
	} else {
		// No filtering requested — return unchanged
		value
	}
}

/// Filter a serde_json::Value object and produce a `Vec<String>` suitable for CSV rows.
fn filter_fields_csv(value: Value, allowed: &Option<HashSet<String>>, order: &Option<Vec<String>>) -> Vec<String> {
	match value {
		Value::Object(map) => {
			let keys: Vec<String> = if let Some(order) = order {
				order.clone()
			} else if let Some(allowed) = allowed {
				map.keys().filter(|k| allowed.contains(*k)).cloned().collect()
			} else {
				map.keys().cloned().collect()
			};

			keys.iter().map(|k| {
				map.get(k).map(|v| match v {
					Value::String(s) => s.clone(),
					_ => v.to_string(),
				}).unwrap_or_default()
			}).collect()
		}
		Value::Array(arr) => arr.iter().map(|v| match v {
			Value::String(s) => s.clone(),
			_ => v.to_string(),
		}).collect(),
		other => vec![match other {
			Value::String(s) => s,
			_ => other.to_string(),
		}],
	}
}

pub fn negotiate_response<T>(
	headers: &HeaderMap,
	Query(params): Query<Params>,
	data: T,
) -> Response
where
	T: serde::Serialize + Clone + std::iter::IntoIterator<Item: serde::Serialize>,
{
	// Parse "fields" query param into a HashSet
	let fields: Option<HashSet<String>> = params.fields.as_ref().map(|s| {
		s.split(',')
			.map(|s| s.trim().to_string())
			.filter(|s| !s.is_empty())
			.collect()
	});

	let available_mimes = vec![
		mime::APPLICATION_JSON,
		Mime::from_str("application/x-yaml").unwrap(),
		Mime::from_str("text/csv").unwrap(),
	];

	let mime = negotiate(headers, available_mimes);

	match mime.essence_str() {
		"application/x-yaml" => {
			let value = serde_json::to_value(data.clone()).unwrap();
			let filtered = filter_fields(value, &fields);
			let yaml_value: serde_yaml_ng::Value = serde_yaml_ng::from_str(&serde_json::to_string(&filtered).unwrap()).unwrap();
			Yaml(yaml_value).into_response()
		},
		"text/csv" => {
			let print_csv_header = mime.get_param("header").map(|n| n.as_str()).unwrap_or("present") != "absent";
			let mut w = csv::WriterBuilder::new().has_headers(false).from_writer(vec![]);

			// If a list of fields has been given, use them for field_order
			let given_field_order: Option<Vec<String>> = params.fields.as_ref().map(|s| {
				s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
			});
			let field_order: Vec<String> = if let Some(ref order) = given_field_order {
				order.clone()
			// Otherwise, try to infer fields from the first record
			} else if let Some(record) = data.clone().into_iter().next() {
				let value = serde_json::to_value(record).unwrap();
				if let Value::Object(map) = value {
					map.iter()
						.filter(|(_, v)| !matches!(v, Value::Array(_)))
						.map(|(k, _)| k.clone())
						.collect()
				} else {
					vec!["value".to_string()]
				}
			// If there's no records, then the fields names are kinda moot
			} else {
				vec![]
			};

			if print_csv_header {
				w.write_record(&field_order).unwrap();
			}

			for record in data {
				let value = serde_json::to_value(record).unwrap();
				let row = filter_fields_csv(value, &fields, &Some(field_order.clone()));
				w.write_record(row).unwrap();
			}

			let csv_output = String::from_utf8(w.into_inner().unwrap()).unwrap();
			Response::builder()
				.status(StatusCode::OK)
				.header(header::CONTENT_TYPE, "text/csv")
				.body(csv_output.into())
				.unwrap()
		},
		"application/json"|_ => {
			let value = serde_json::to_value(data.clone()).unwrap();
			let filtered = filter_fields(value, &fields);
			Json(filtered).into_response()
		},
	}
}


#[cfg(test)]
mod negotiate_tests {
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

#[cfg(test)]
mod negotiate_response_tests {
	use super::*;
	use axum::http::HeaderMap;
	use axum::extract::Query;
	use serde_json::Value;
	use axum::body::to_bytes;

	#[derive(Clone, serde::Serialize)]
	struct TestRecord {
		a: i32,
		b: String,
		c: bool,
	}

	fn make_data() -> Vec<TestRecord> {
		vec![
			TestRecord { a: 1, b: "x".to_string(), c: true },
			TestRecord { a: 2, b: "y".to_string(), c: false },
		]
	}

	async fn body_string(resp: Response) -> String {
		// set a maximum limit of 1 MB
		let bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
		String::from_utf8(bytes.to_vec()).unwrap()
	}


	#[tokio::test]
	async fn json_no_fields() {
		let mut headers = HeaderMap::new();
		headers.insert(http::header::ACCEPT, "application/json".parse().unwrap());

		let resp = negotiate_response(&headers, Query(Params { fields: None }), make_data());
		let body = body_string(resp).await;
		let parsed: Value = serde_json::from_str(&body).unwrap();
		assert_eq!(parsed.as_array().unwrap().len(), 2);
		assert!(parsed.as_array().unwrap()[0].get("a").is_some());
		assert!(parsed.as_array().unwrap()[0].get("b").is_some());
		assert!(parsed.as_array().unwrap()[0].get("c").is_some());
	}

	#[tokio::test]
	async fn json_with_fields() {
		let mut headers = HeaderMap::new();
		headers.insert(http::header::ACCEPT, "application/json".parse().unwrap());

		let resp = negotiate_response(
			&headers,
			Query(Params { fields: Some("a,c".to_string()) }),
			make_data()
		);
		let body = body_string(resp).await;
		let parsed: Value = serde_json::from_str(&body).unwrap();
		let first = parsed.as_array().unwrap()[0].as_object().unwrap();
		assert!(first.contains_key("a"));
		assert!(first.contains_key("c"));
		assert!(!first.contains_key("b"));
	}

	#[tokio::test]
	async fn yaml_no_fields() {
		let mut headers = HeaderMap::new();
		headers.insert(http::header::ACCEPT, "application/x-yaml".parse().unwrap());

		let resp = negotiate_response(&headers, Query(Params { fields: None }), make_data());
		let body = body_string(resp).await;
		assert!(body.contains("a: 1"));
		assert!(body.contains("b: x"));
		assert!(body.contains("c: true"));
	}

	#[tokio::test]
	async fn yaml_with_fields() {
		let mut headers = HeaderMap::new();
		headers.insert(http::header::ACCEPT, "application/x-yaml".parse().unwrap());

		let resp = negotiate_response(
			&headers,
			Query(Params { fields: Some("b,c".to_string()) }),
			make_data()
		);
		let body = body_string(resp).await;
		assert!(!body.contains("a:"));
		assert!(body.contains("b: x"));
		assert!(body.contains("c: true"));
	}

	#[tokio::test]
	async fn csv_with_header() {
		let mut headers = HeaderMap::new();
		headers.insert(http::header::ACCEPT, "text/csv".parse().unwrap());

		let resp = negotiate_response(
			&headers,
			Query(Params { fields: Some("b,a".to_string()) }),
			make_data()
		);
		let body = body_string(resp).await;
		let mut lines = body.lines();
		assert_eq!(lines.next().unwrap(), "b,a"); // header
		assert!(lines.next().unwrap().contains("x,1"));
		assert!(lines.next().unwrap().contains("y,2"));
	}

	#[tokio::test]
	async fn csv_without_header() {
		let mut headers = HeaderMap::new();
		headers.insert(http::header::ACCEPT, "text/csv;header=absent".parse().unwrap());

		let resp = negotiate_response(
			&headers,
			Query(Params { fields: Some("b,a".to_string()) }),
			make_data()
		);
		let body = body_string(resp).await;
		let mut lines = body.lines();
		let first_line = lines.next().unwrap();
		assert!(!first_line.contains("b,a")); // no header
		assert!(first_line.contains("x,1") || first_line.contains("y,2")); // first row
	}

	#[tokio::test]
	async fn csv_no_fields() {
		let mut headers = HeaderMap::new();
		headers.insert(http::header::ACCEPT, "text/csv".parse().unwrap());

		let resp = negotiate_response(&headers, Query(Params { fields: None }), make_data());
		let body = body_string(resp).await;
		let mut lines = body.lines();
		let header = lines.next().unwrap();
		assert!(header.contains("a"));
		assert!(header.contains("b"));
		assert!(header.contains("c"));
		let first_data = lines.next().unwrap();
		assert!(first_data.contains("1"));
		assert!(first_data.contains("x"));
		assert!(first_data.contains("true"));
	}
}
