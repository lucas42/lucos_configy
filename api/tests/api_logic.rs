use lucos_configy_api::routing::app;
use lucos_configy_api::data::Data;
use axum::{
	body::Body,
	http::{Request, StatusCode},
};
use std::sync::Arc;
use tower::ServiceExt;
use http_body_util::BodyExt;
use tempfile::tempdir;
use std::fs::File;
use std::io::Write;

async fn create_mock_data() -> Arc<Data> {
	let dir = tempdir().unwrap();
	
	let systems_path = dir.path().join("systems.yaml");
	let mut systems_file = File::create(systems_path).unwrap();
	writeln!(systems_file, "
system1:
  domain: s1.example.com
  http_port: 80
  hosts: [host1]
system2:
  domain: s2.test.com
  http_port: 8080
  hosts: [host1, host2]
").unwrap();

	let volumes_path = dir.path().join("volumes.yaml");
	let mut volumes_file = File::create(volumes_path).unwrap();
	writeln!(volumes_file, "
vol1:
  description: Volume 1
  recreate_effort: Low
vol2:
  description: Volume 2
  skip_backup: true
").unwrap();

	let hosts_path = dir.path().join("hosts.yaml");
	let mut hosts_file = File::create(hosts_path).unwrap();
	writeln!(hosts_file, "
host1:
  domain: h1.example.com
  ipv4: 1.1.1.1
  serves_http: true
host2:
  domain: h2.example.com
  ipv4: 1.1.1.2
  serves_http: false
").unwrap();

	let components_path = dir.path().join("components.yaml");
	let mut components_file = File::create(components_path).unwrap();
	writeln!(components_file, "
comp1: {{}}
comp2: {{}}
").unwrap();

	let data = Data::from_dir(dir.path()).unwrap();
	Arc::new(data)
}

#[tokio::test]
async fn test_systems_all() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/systems").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	assert_eq!(response.headers().get("content-type").unwrap(), "application/json");

	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_systems_subdomain() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/systems/subdomain/example.com").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	assert_eq!(body.as_array().unwrap().len(), 1);
	assert_eq!(body[0]["domain"], "s1.example.com");
}

#[tokio::test]
async fn test_systems_http() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/systems/http").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_systems_host() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/systems/host/host2").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	assert_eq!(body.as_array().unwrap().len(), 1);
	assert_eq!(body[0]["id"], "system2");
}

#[tokio::test]
async fn test_volumes_all() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/volumes").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_hosts_all() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/hosts").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_hosts_http() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/hosts/http").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	assert_eq!(body.as_array().unwrap().len(), 1);
	assert_eq!(body[0]["id"], "host1");
}

#[tokio::test]
async fn test_components_all() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/components").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_content_negotiation_yaml() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(
			Request::builder()
				.uri("/systems")
				.header("accept", "application/x-yaml")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	assert_eq!(response.headers().get("content-type").unwrap(), "application/yaml");
}

#[tokio::test]
async fn test_content_negotiation_csv() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(
			Request::builder()
				.uri("/systems")
				.header("accept", "text/csv")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	assert_eq!(response.headers().get("content-type").unwrap(), "text/csv");
}

#[tokio::test]
async fn test_query_params_fields() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(
			Request::builder()
				.uri("/systems?fields=id,domain")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	let first = &body[0];
	assert!(first.get("id").is_some());
	assert!(first.get("domain").is_some());
	assert!(first.get("http_port").is_none());
}
