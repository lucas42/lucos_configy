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
  public_ports:
    - port: 25
      protocol: tcp
      purpose: SMTP inbound
    - port: 587
      protocol: tcp
      purpose: SMTP submission
system2:
  domain: s2.test.com
  http_port: 8080
  hosts: [host1, host2]
system3:
  domain: s3.other.net
  hosts: [host2]
  public_ports:
    - port: 53
      protocol: udp
      purpose: DNS
").unwrap();

	let volumes_path = dir.path().join("volumes.yaml");
	let mut volumes_file = File::create(volumes_path).unwrap();
	writeln!(volumes_file, "
vol1:
  description: Volume 1
  recreate_effort: Low
  backup_strategy: incremental
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
  can_reach_external_services: false
host3:
  domain: h3.example.com
  ipv4: 1.1.1.3
  firewall_enforce: true
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
	assert_eq!(body.as_array().unwrap().len(), 3);
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
	assert_eq!(body[0]["subdomain"], "s1");
}

#[tokio::test]
async fn test_systems_subdomain_multiple() {
	let data = create_mock_data().await;
	let app = app(data);

	// Both systems share the same root: "com" — so both should be returned with their starts
	let response = app
		.oneshot(Request::builder().uri("/systems/subdomain/com").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	let systems = body.as_array().unwrap();
	assert_eq!(systems.len(), 2);

	let system1 = systems.iter().find(|s| s["id"] == "system1").unwrap();
	assert_eq!(system1["subdomain"], "s1.example");

	let system2 = systems.iter().find(|s| s["id"] == "system2").unwrap();
	assert_eq!(system2["subdomain"], "s2.test");
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
	assert_eq!(body.as_array().unwrap().len(), 2);
	let ids: Vec<&str> = body.as_array().unwrap().iter()
		.map(|s| s["id"].as_str().unwrap())
		.collect();
	assert!(ids.contains(&"system2"));
	assert!(ids.contains(&"system3"));
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
	assert_eq!(body.as_array().unwrap().len(), 3);
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
async fn test_hosts_can_reach_external_services_default() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/hosts").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

	// host1 does not set can_reach_external_services — should default to true
	let host1 = body.as_array().unwrap().iter().find(|h| h["id"] == "host1").unwrap();
	assert_eq!(host1["can_reach_external_services"], true,
		"can_reach_external_services should default to true when absent from YAML");

	// host2 explicitly sets can_reach_external_services: false
	let host2 = body.as_array().unwrap().iter().find(|h| h["id"] == "host2").unwrap();
	assert_eq!(host2["can_reach_external_services"], false,
		"can_reach_external_services: false should be honoured");
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
	assert!(body.contains("eolas:Technological"));
	assert!(body.contains("skos:prefLabel \"Technological\""));
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

	assert!(body.contains("/systems#system1>"));
	assert!(body.contains("a configy:System"));
	assert!(body.contains("configy:domain \"s1.example.com\""));
	assert!(body.contains("configy:httpPort 80"));
	assert!(body.contains("/hosts#host1>"));
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

	assert!(body.contains("/hosts#host1>"));
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

	assert!(body.contains("/volumes#vol1>"));
	assert!(body.contains("a configy:Volume"));
	assert!(body.contains("configy:recreateEffort \"Low\""));
	assert!(body.contains("configy:backupStrategy \"incremental\""));
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

// ── public_ports field and endpoint tests ──────────────────────────────────────

#[tokio::test]
async fn test_systems_include_public_ports() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/systems").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

	let system1 = body.as_array().unwrap().iter().find(|s| s["id"] == "system1").unwrap();
	let ports = system1["public_ports"].as_array().unwrap();
	assert_eq!(ports.len(), 2);
	assert_eq!(ports[0]["port"], 25);
	assert_eq!(ports[0]["protocol"], "tcp");
	assert_eq!(ports[0]["purpose"], "SMTP inbound");
	assert_eq!(ports[1]["port"], 587);
	assert_eq!(ports[1]["protocol"], "tcp");

	// system2 has no public_ports — should be an empty array, not null or absent
	let system2 = body.as_array().unwrap().iter().find(|s| s["id"] == "system2").unwrap();
	assert_eq!(system2["public_ports"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_host_public_ports_endpoint() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/systems/host/host1/public-ports").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	let ports = body.as_array().unwrap();

	// host1 has system1 (2 ports) and system2 (0 ports) → 2 records total
	assert_eq!(ports.len(), 2);

	// All records for host1 come from system1
	for port in ports {
		assert_eq!(port["system"], "system1");
	}
	// The two expected ports
	let p25 = ports.iter().find(|p| p["port"] == 25).unwrap();
	assert_eq!(p25["protocol"], "tcp");
	assert_eq!(p25["purpose"], "SMTP inbound");

	let p587 = ports.iter().find(|p| p["port"] == 587).unwrap();
	assert_eq!(p587["protocol"], "tcp");
}

#[tokio::test]
async fn test_host_public_ports_endpoint_host2() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/systems/host/host2/public-ports").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	let ports = body.as_array().unwrap();

	// host2 has system2 (0 ports) and system3 (1 UDP port) → 1 record total
	assert_eq!(ports.len(), 1);
	assert_eq!(ports[0]["system"], "system3");
	assert_eq!(ports[0]["port"], 53);
	assert_eq!(ports[0]["protocol"], "udp");
	assert_eq!(ports[0]["purpose"], "DNS");
}

#[tokio::test]
async fn test_host_public_ports_empty_host() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/systems/host/nonexistent/public-ports").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
	assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_all_turtle_contains_public_ports_ontology() {
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

	// Ontology declares PublicPort class and its predicates
	assert!(body.contains("configy:PublicPort"));
	assert!(body.contains("configy:publicPort"));
	assert!(body.contains("configy:portNumber"));
	assert!(body.contains("configy:portProtocol"));
	assert!(body.contains("configy:portPurpose"));
}

#[tokio::test]
async fn test_all_turtle_contains_public_port_data() {
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

	// system1 has public ports — they should appear as blank nodes
	assert!(body.contains("configy:portNumber 25"));
	assert!(body.contains("configy:portProtocol \"tcp\""));
	assert!(body.contains("configy:portPurpose \"SMTP inbound\""));
	assert!(body.contains("configy:portNumber 53"));
	assert!(body.contains("configy:portProtocol \"udp\""));
	assert!(body.contains("configy:portPurpose \"DNS\""));
}

#[test]
fn test_public_ports_invalid_protocol_fails_load() {
	use lucos_configy_api::data::Data;
	use tempfile::tempdir;

	let dir = tempdir().unwrap();

	// Write a systems.yaml with an invalid protocol value
	let systems_path = dir.path().join("systems.yaml");
	std::fs::write(&systems_path, "
broken:
  hosts: [host1]
  public_ports:
    - port: 80
      protocol: ftp
      purpose: Should fail
").unwrap();

	// Write minimal stubs for the other YAML files so from_dir doesn't fail on file-not-found
	std::fs::write(dir.path().join("volumes.yaml"), "{}\n").unwrap();
	std::fs::write(dir.path().join("hosts.yaml"), "{}\n").unwrap();
	std::fs::write(dir.path().join("components.yaml"), "{}\n").unwrap();
	std::fs::write(dir.path().join("scripts.yaml"), "{}\n").unwrap();

	let result = Data::from_dir(dir.path());
	assert!(result.is_err(), "Expected config load to fail on invalid protocol 'ftp'");
}

#[test]
fn test_public_ports_invalid_port_zero_fails_load() {
	use lucos_configy_api::data::Data;
	use tempfile::tempdir;

	let dir = tempdir().unwrap();

	std::fs::write(dir.path().join("systems.yaml"), "
broken:
  hosts: [host1]
  public_ports:
    - port: 0
      protocol: tcp
      purpose: Port zero is invalid
").unwrap();
	std::fs::write(dir.path().join("volumes.yaml"), "{}\n").unwrap();
	std::fs::write(dir.path().join("hosts.yaml"), "{}\n").unwrap();
	std::fs::write(dir.path().join("components.yaml"), "{}\n").unwrap();
	std::fs::write(dir.path().join("scripts.yaml"), "{}\n").unwrap();

	let result = Data::from_dir(dir.path());
	assert!(result.is_err(), "Expected config load to fail on port 0");
}

// ── /hosts/{host} endpoint tests ─────────────────────────────────────────────

#[tokio::test]
async fn test_hosts_get_by_id() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/hosts/host1").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	assert_eq!(response.headers().get("content-type").unwrap(), "application/json");

	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

	// Single object, not an array
	assert!(body.as_object().is_some());
	assert_eq!(body["id"], "host1");
	assert_eq!(body["domain"], "h1.example.com");
	assert_eq!(body["ipv4"], "1.1.1.1");
	assert_eq!(body["serves_http"], true);
}

#[tokio::test]
async fn test_hosts_get_firewall_enforce_default_false() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/hosts/host1").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

	// host1 does not set firewall_enforce — should default to false
	assert_eq!(body["firewall_enforce"], false,
		"firewall_enforce should default to false when absent from YAML");
}

#[tokio::test]
async fn test_hosts_get_firewall_enforce_true() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/hosts/host3").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

	// host3 explicitly sets firewall_enforce: true
	assert_eq!(body["firewall_enforce"], true,
		"firewall_enforce: true should be honoured");
}

#[tokio::test]
async fn test_hosts_get_not_found() {
	let data = create_mock_data().await;
	let app = app(data);

	let response = app
		.oneshot(Request::builder().uri("/hosts/nonexistent").body(Body::empty()).unwrap())
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_all_turtle_contains_firewall_enforce_ontology() {
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

	assert!(body.contains("configy:firewallEnforce"),
		"Turtle output should declare firewallEnforce predicate in ontology");
}

#[tokio::test]
async fn test_all_turtle_contains_firewall_enforce_data() {
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

	// host3 has firewall_enforce: true — should appear in turtle output
	assert!(body.contains("configy:firewallEnforce true"),
		"Turtle output should include firewallEnforce true for host3");
}
