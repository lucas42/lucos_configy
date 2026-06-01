use std::sync::Arc;
use axum::{
	extract::{Query, Path, State},
	response::Response,
	http::header::HeaderMap,
};
use serde::Serialize;
use crate::conneg::negotiate_response;

/// Flat record returned by the `/systems/host/{host}/public-ports` endpoint.
/// Contains the owning system's id alongside the port details so consumers
/// (e.g. lucos_firewall) don't need to group by system themselves.
#[derive(Serialize, Clone)]
struct HostPublicPort {
	system: String,
	port: u16,
	protocol: crate::data::Protocol,
	purpose: String,
}

#[derive(Serialize, Clone)]
struct SystemWithSubdomain {
	#[serde(flatten)]
	system: crate::data::System,
	subdomain: Option<String>,
}

pub async fn all(
	State(data): State<Arc<crate::data::Data>>,
	headers: HeaderMap,
	params: Query<crate::conneg::Params>,
) -> Response {
	negotiate_response(&headers, params, data.get_systems())
}

pub async fn subdomain(
	Path(root_domain): Path<String>,
	State(data): State<Arc<crate::data::Data>>,
	headers: HeaderMap,
	params: Query<crate::conneg::Params>,
) -> Response {
	let systems = data.get_systems_filtered(|system| system.domain.as_ref().is_some_and(|domain| domain.ends_with(&root_domain)));
	let systems_with_subdomain: Vec<SystemWithSubdomain> = systems
		.into_iter()
		.map(|system| {
			let subdomain = system.domain.as_ref()
				.and_then(|domain| domain.strip_suffix(root_domain.as_str()))
				.map(|prefix| prefix.trim_end_matches('.').to_string())
				.filter(|s| !s.is_empty());
			SystemWithSubdomain { system, subdomain }
		})
		.collect();
	negotiate_response(&headers, params, systems_with_subdomain)
}

pub async fn http(
	State(data): State<Arc<crate::data::Data>>,
	headers: HeaderMap,
	params: Query<crate::conneg::Params>,
) -> Response {
	let http_systems = data.get_systems_filtered(|system| system.http_port.is_some());
	negotiate_response(&headers, params, http_systems)
}

pub async fn host(
	Path(host): Path<String>,
	State(data): State<Arc<crate::data::Data>>,
	headers: HeaderMap,
	params: Query<crate::conneg::Params>,
) -> Response {
	let systems_on_host = data.get_systems_filtered(|system| system.hosts.contains(&host));
	negotiate_response(&headers, params, systems_on_host)
}

/// Returns a flat list of all public ports for systems on the given host.
/// Each record includes the owning system's id alongside the port, protocol, and purpose.
/// Intended for consumption by lucos_firewall to generate iptables rules.
pub async fn host_public_ports(
	Path(host): Path<String>,
	State(data): State<Arc<crate::data::Data>>,
	headers: HeaderMap,
	params: Query<crate::conneg::Params>,
) -> Response {
	let flat_ports: Vec<HostPublicPort> = data
		.get_systems_filtered(|system| system.hosts.contains(&host))
		.into_iter()
		.flat_map(|system| {
			let system_id = system.id.clone().unwrap_or_default();
			system.public_ports.into_iter().map(move |port| HostPublicPort {
				system: system_id.clone(),
				port: port.port,
				protocol: port.protocol,
				purpose: port.purpose,
			})
		})
		.collect();
	negotiate_response(&headers, params, flat_ports)
}
