use std::sync::Arc;
use axum::{
	extract::State,
	response::{IntoResponse, Response},
	http::{header, StatusCode},
	http::header::HeaderMap,
};
use std::str::FromStr;
use mime::Mime;
use crate::conneg::negotiate;
use crate::data::{Data, System, Host, Volume, Component, Script};

fn escape_turtle_literal(s: &str) -> String {
	s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r")
}

fn turtle_ontology() -> String {
	let mut out = String::new();

	out.push_str("# Ontology\n");

	// eolas:Technological must be defined here so the RDF is self-contained
	// (arachne ingestor requires type metadata inline — see lucos_arachne#371)
	out.push_str("\neolas:Technological\n    skos:prefLabel \"Technological\" .\n");

	for (class, label) in &[
		("System", "System"),
		("Host", "Host"),
		("Volume", "Volume"),
		("Component", "Component"),
		("Script", "Script"),
	] {
		out.push_str(&format!(
			"\nconfigy:{class}\n    a owl:Class ;\n    skos:prefLabel \"{label}\" ;\n    eolas:hasCategory eolas:Technological .\n"
		));
	}

	out.push_str("\n# Predicates\n");

	for (pred, label, domain, range) in &[
		("domain", "Domain", "configy:System", "xsd:string"),
		("httpPort", "HTTP Port", "configy:System", "xsd:integer"),
		("hostedOn", "Hosted On", "configy:System", "configy:Host"),
		("unsupervisedAgentCode", "Unsupervised Agent Code", "configy:System", "xsd:boolean"),
		("ipv4", "IPv4 Address", "configy:Host", "xsd:string"),
		("ipv6", "IPv6 Address", "configy:Host", "xsd:string"),
		("ipv4Nat", "IPv4 NAT Address", "configy:Host", "xsd:string"),
		("servesHttp", "Serves HTTP", "configy:Host", "xsd:boolean"),
		("recreateEffort", "Recreate Effort", "configy:Volume", "xsd:string"),
		("skipBackup", "Skip Backup", "configy:Volume", "xsd:boolean"),
		("skipBackupOnHost", "Skip Backup On Host", "configy:Volume", "configy:Host"),
	] {
		out.push_str(&format!(
			"\nconfigy:{pred}\n    a rdf:Property ;\n    rdfs:label \"{label}\" ;\n    rdfs:domain {domain} ;\n    rdfs:range {range} .\n"
		));
	}

	out
}

fn turtle_systems(systems: &[System], base: &str) -> String {
	let mut out = String::new();
	for system in systems {
		let id = match &system.id {
			Some(id) => id,
			None => continue,
		};
		out.push_str(&format!("\n<{base}/systems#{id}>\n    a configy:System ;\n    skos:prefLabel \"{}\"", escape_turtle_literal(id)));
		if let Some(domain) = &system.domain {
			out.push_str(&format!(" ;\n    configy:domain \"{}\"", escape_turtle_literal(domain)));
		}
		if let Some(port) = system.http_port {
			out.push_str(&format!(" ;\n    configy:httpPort {port}"));
		}
		for host in &system.hosts {
			out.push_str(&format!(" ;\n    configy:hostedOn <{base}/hosts#{host}>"));
		}
		if system.unsupervised_agent_code {
			out.push_str(" ;\n    configy:unsupervisedAgentCode true");
		}
		out.push_str(" .\n");
	}
	out
}

fn turtle_hosts(hosts: &[Host], base: &str) -> String {
	let mut out = String::new();
	for host in hosts {
		let id = match &host.id {
			Some(id) => id,
			None => continue,
		};
		out.push_str(&format!("\n<{base}/hosts#{id}>\n    a configy:Host ;\n    skos:prefLabel \"{}\"", escape_turtle_literal(id)));
		if let Some(domain) = &host.domain {
			out.push_str(&format!(" ;\n    configy:domain \"{}\"", escape_turtle_literal(domain)));
		}
		if let Some(ipv4) = &host.ipv4 {
			out.push_str(&format!(" ;\n    configy:ipv4 \"{}\"", escape_turtle_literal(ipv4)));
		}
		if let Some(ipv6) = &host.ipv6 {
			out.push_str(&format!(" ;\n    configy:ipv6 \"{}\"", escape_turtle_literal(ipv6)));
		}
		if let Some(nat) = &host.ipv4_nat {
			out.push_str(&format!(" ;\n    configy:ipv4Nat \"{}\"", escape_turtle_literal(nat)));
		}
		if host.serves_http {
			out.push_str(" ;\n    configy:servesHttp true");
		}
		out.push_str(" .\n");
	}
	out
}

fn turtle_volumes(volumes: &[Volume], base: &str) -> String {
	let mut out = String::new();
	for volume in volumes {
		let id = match &volume.id {
			Some(id) => id,
			None => continue,
		};
		out.push_str(&format!("\n<{base}/volumes#{id}>\n    a configy:Volume ;\n    skos:prefLabel \"{}\"", escape_turtle_literal(id)));
		if let Some(desc) = &volume.description {
			out.push_str(&format!(" ;\n    dc:description \"{}\"", escape_turtle_literal(desc)));
		}
		if let Some(effort) = &volume.recreate_effort {
			out.push_str(&format!(" ;\n    configy:recreateEffort \"{}\"", escape_turtle_literal(effort)));
		}
		if volume.skip_backup {
			out.push_str(" ;\n    configy:skipBackup true");
		}
		for host in &volume.skip_backup_on_hosts {
			out.push_str(&format!(" ;\n    configy:skipBackupOnHost <{base}/hosts#{host}>"));
		}
		out.push_str(" .\n");
	}
	out
}

fn turtle_components(components: &[Component], base: &str) -> String {
	let mut out = String::new();
	for component in components {
		let id = match &component.id {
			Some(id) => id,
			None => continue,
		};
		out.push_str(&format!("\n<{base}/components#{id}>\n    a configy:Component ;\n    skos:prefLabel \"{}\"", escape_turtle_literal(id)));
		if component.unsupervised_agent_code {
			out.push_str(" ;\n    configy:unsupervisedAgentCode true");
		}
		out.push_str(" .\n");
	}
	out
}

fn turtle_scripts(scripts: &[Script], base: &str) -> String {
	let mut out = String::new();
	for script in scripts {
		let id = match &script.id {
			Some(id) => id,
			None => continue,
		};
		out.push_str(&format!("\n<{base}/scripts#{id}>\n    a configy:Script ;\n    skos:prefLabel \"{}\"", escape_turtle_literal(id)));
		if script.unsupervised_agent_code {
			out.push_str(" ;\n    configy:unsupervisedAgentCode true");
		}
		out.push_str(" .\n");
	}
	out
}

pub fn to_turtle(data: &Data, base: &str) -> String {
	let mut out = String::new();
	out.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
	out.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
	out.push_str("@prefix owl: <http://www.w3.org/2002/07/owl#> .\n");
	out.push_str("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n");
	out.push_str("@prefix skos: <http://www.w3.org/2004/02/skos/core#> .\n");
	out.push_str("@prefix dc: <http://purl.org/dc/elements/1.1/> .\n");
	out.push_str("@prefix eolas: <https://eolas.l42.eu/ontology/> .\n");
	out.push_str(&format!("@prefix configy: <{base}/ontology#> .\n"));

	out.push_str("\n");
	out.push_str(&turtle_ontology());

	out.push_str("\n# Systems\n");
	out.push_str(&turtle_systems(&data.get_systems(), base));

	out.push_str("\n# Hosts\n");
	out.push_str(&turtle_hosts(&data.get_hosts(), base));

	out.push_str("\n# Volumes\n");
	out.push_str(&turtle_volumes(&data.get_volumes(), base));

	out.push_str("\n# Components\n");
	out.push_str(&turtle_components(&data.get_components(), base));

	out.push_str("\n# Scripts\n");
	out.push_str(&turtle_scripts(&data.get_scripts(), base));

	out
}

pub async fn all(
	State(data): State<Arc<Data>>,
	headers: HeaderMap,
) -> Response {
	let available_mimes = vec![
		Mime::from_str("text/turtle").unwrap(),
		mime::APPLICATION_JSON,
	];
	let mime = negotiate(&headers, available_mimes);

	if mime.essence_str() == "text/turtle" {
		let base = std::env::var("APP_ORIGIN").unwrap_or_else(|_| "https://configy.l42.eu".to_string());
		let turtle = to_turtle(&data, &base);
		return Response::builder()
			.status(StatusCode::OK)
			.header(header::CONTENT_TYPE, "text/turtle; charset=utf-8")
			.body(turtle.into())
			.unwrap();
	}

	// Fallback: combined JSON
	let combined = serde_json::json!({
		"systems": data.get_systems(),
		"hosts": data.get_hosts(),
		"volumes": data.get_volumes(),
		"components": data.get_components(),
		"scripts": data.get_scripts(),
	});
	axum::Json(combined).into_response()
}
