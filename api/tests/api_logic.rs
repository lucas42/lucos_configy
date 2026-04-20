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
  unsupervisedAgentCode: true
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
comp1:
  unsupervisedAgentCode: true
comp2: {{}}
").unwrap();

	let scripts_path = dir.path().join("scripts.yaml");
	let mut scripts_file = File::create(scripts_path).unwrap();
	writeln!(scripts_file, "
script1:
  unsupervisedAgentCode: true
script2: {{}}
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

#[tokio::test]
async fn test_systems_unsupervised_agent_code_set() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/systems").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

	// system1 has unsupervisedAgentCode: true in the mock YAML
	let system1 = body.as_array().unwrap().iter().find(|s| s["id"] == "system1").unwrap();
	assert_eq!(system1["unsupervisedAgentCode"], true);

	// system2 does not set the field, so it should default to false
	let system2 = body.as_array().unwrap().iter().find(|s| s["id"] == "system2").unwrap();
	assert_eq!(system2["unsupervisedAgentCode"], false);
}

#[tokio::test]
async fn test_scripts_all() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/scripts").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_scripts_unsupervised_agent_code_set() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/scripts").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

	// script1 has unsupervisedAgentCode: true in the mock YAML
	let script1 = body.as_array().unwrap().iter().find(|s| s["id"] == "script1").unwrap();
	assert_eq!(script1["unsupervisedAgentCode"], true);

	// script2 does not set the field, so it should default to false
	let script2 = body.as_array().unwrap().iter().find(|s| s["id"] == "script2").unwrap();
	assert_eq!(script2["unsupervisedAgentCode"], false);
}

#[tokio::test]
async fn test_repositories_get_system() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/repositories/system1").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	assert_eq!(body["id"], "system1");
	assert_eq!(body["type"], "system");
	assert_eq!(body["domain"], "s1.example.com");
	assert!(body.get("unsupervisedAgentCode").is_some());
	// Single object, not an array
	assert!(body.as_object().is_some());
}

#[tokio::test]
async fn test_repositories_get_component() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/repositories/comp1").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	assert_eq!(body["id"], "comp1");
	assert_eq!(body["type"], "component");
	assert_eq!(body["unsupervisedAgentCode"], true);
}

#[tokio::test]
async fn test_repositories_get_script() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/repositories/script1").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	assert_eq!(body["id"], "script1");
	assert_eq!(body["type"], "script");
	assert_eq!(body["unsupervisedAgentCode"], true);
}

#[tokio::test]
async fn test_repositories_get_not_found() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/repositories/nonexistent").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_repositories_get_yaml() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(
			Request::builder()
				.uri("/repositories/system1")
				.header("accept", "application/x-yaml")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	assert_eq!(response.headers().get("content-type").unwrap(), "application/yaml");
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body_str = std::str::from_utf8(&body).unwrap();
	assert!(body_str.contains("id: system1"));
	assert!(body_str.contains("type: system"));
}

#[tokio::test]
async fn test_repositories_get_fields_filter() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(
			Request::builder()
				.uri("/repositories/system1?fields=id,type")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	assert_eq!(body["id"], "system1");
	assert_eq!(body["type"], "system");
	assert!(body.get("domain").is_none());
	assert!(body.get("http_port").is_none());
}

#[tokio::test]
async fn test_components_unsupervised_agent_code_set() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/components").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

	// comp1 has unsupervisedAgentCode: true in the mock YAML
	let comp1 = body.as_array().unwrap().iter().find(|c| c["id"] == "comp1").unwrap();
	assert_eq!(comp1["unsupervisedAgentCode"], true);

	// comp2 does not set the field, so it should default to false
	let comp2 = body.as_array().unwrap().iter().find(|c| c["id"] == "comp2").unwrap();
	assert_eq!(comp2["unsupervisedAgentCode"], false);
}

#[tokio::test]
async fn test_all_turtle_content_type() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(
			Request::builder()
				.uri("/all")
				.header("Accept", "text/turtle")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	assert!(response.headers().get("content-type").unwrap().to_str().unwrap().contains("text/turtle"));
}

#[tokio::test]
async fn test_all_turtle_contains_prefixes() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(
			Request::builder()
				.uri("/all")
				.header("Accept", "text/turtle")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body = std::str::from_utf8(&body).unwrap();

	assert!(body.contains("@prefix configy: <https://configy.l42.eu/ontology#>"));
	assert!(body.contains("@prefix skos:"));
	assert!(body.contains("@prefix owl:"));
}

#[tokio::test]
async fn test_all_turtle_contains_ontology_classes() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(
			Request::builder()
				.uri("/all")
				.header("Accept", "text/turtle")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body = std::str::from_utf8(&body).unwrap();

	assert!(body.contains("configy:System"));
	assert!(body.contains("configy:Host"));
	assert!(body.contains("configy:Volume"));
	assert!(body.contains("configy:Component"));
	assert!(body.contains("configy:Script"));
	assert!(body.contains("owl:Class"));
	assert!(body.contains("eolas:hasCategory eolas:Technological"));
}

#[tokio::test]
async fn test_all_turtle_contains_systems() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(
			Request::builder()
				.uri("/all")
				.header("Accept", "text/turtle")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body = std::str::from_utf8(&body).unwrap();

	assert!(body.contains("<https://configy.l42.eu/systems/system1>"));
	assert!(body.contains("a configy:System"));
	assert!(body.contains("configy:domain \"s1.example.com\""));
	assert!(body.contains("configy:httpPort 80"));
	assert!(body.contains("configy:hostedOn <https://configy.l42.eu/hosts/host1>"));
	assert!(body.contains("configy:unsupervisedAgentCode true"));
}

#[tokio::test]
async fn test_all_turtle_contains_hosts() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(
			Request::builder()
				.uri("/all")
				.header("Accept", "text/turtle")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body = std::str::from_utf8(&body).unwrap();

	assert!(body.contains("<https://configy.l42.eu/hosts/host1>"));
	assert!(body.contains("a configy:Host"));
	assert!(body.contains("configy:ipv4 \"1.1.1.1\""));
	assert!(body.contains("configy:servesHttp true"));
}

#[tokio::test]
async fn test_all_turtle_contains_volumes() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(
			Request::builder()
				.uri("/all")
				.header("Accept", "text/turtle")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body = std::str::from_utf8(&body).unwrap();

	assert!(body.contains("<https://configy.l42.eu/volumes/vol1>"));
	assert!(body.contains("a configy:Volume"));
	assert!(body.contains("configy:recreateEffort \"Low\""));
	assert!(body.contains("configy:skipBackup true"));
}

#[tokio::test]
async fn test_all_json_fallback() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(
			Request::builder()
				.uri("/all")
				.header("Accept", "application/json")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	assert!(body.get("systems").is_some());
	assert!(body.get("hosts").is_some());
	assert!(body.get("volumes").is_some());
	assert!(body.get("components").is_some());
	assert!(body.get("scripts").is_some());
}
