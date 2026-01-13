use serde_yaml_ng;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::vec::Vec;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone)]
pub struct System {
	id: Option<String>, // This is optional because the raw yaml specifies it as than key, rather than as an attribute
	pub domain: Option<String>,
	pub http_port: Option<u16>, // TCP ports are 16-bit integers
	#[serde(default)]
	pub hosts: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Volume {
	id: Option<String>, // This is optional because the raw yaml specifies it as than key, rather than as an attribute
	pub description: Option<String>,
	pub recreate_effort: Option<String>,
	#[serde(default)]
	pub skip_backup: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Host {
	id: Option<String>, // This is optional because the raw yaml specifies it as than key, rather than as an attribute
	pub domain: Option<String>,
	pub ipv4: Option<String>, // The primary IPv4 address for this host
	pub ipv6: Option<String>, // The primary IPv6 address for this host
	pub ipv4_nat: Option<String>, // An IPv4 address that may forward ports to the host.  For use from legacy networks which don't support IPv6.
	#[serde(default)]
	pub serves_http: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Component {
	id: Option<String>, // This is optional because the raw yaml specifies it as than key, rather than as an attribute
}

// The format the data appears in the YAML file
#[derive(Deserialize)]
struct RawData {
	systems: HashMap<String, System>,
	volumes: HashMap<String, Volume>,
	hosts: HashMap<String, Host>,
	components: HashMap<String, Component>,
}

// The format of data to expose publically
pub struct Data {
	systems: Vec<System>,
	volumes: Vec<Volume>,
	hosts: Vec<Host>,
	components: Vec<Component>,
}


impl Data {
	pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Data, Box<dyn std::error::Error>> {
		let file = std::fs::File::open(path)?;
		let mut raw_data: RawData = serde_yaml_ng::from_reader(file)?;
		let mut data = Data {
			systems: vec![],
			volumes: vec![],
			hosts: vec![],
			components: vec![],
		};
		for (id, system) in raw_data.systems.iter_mut() {
			system.id = Some(id.to_string());
			data.systems.push(system.clone());
		}
		for (id, volume) in raw_data.volumes.iter_mut() {
			volume.id = Some(id.to_string());
			data.volumes.push(volume.clone());
		}
		for (id, host) in raw_data.hosts.iter_mut() {
			host.id = Some(id.to_string());
			data.hosts.push(host.clone());
		}
		for (id, component) in raw_data.components.iter_mut() {
			component.id = Some(id.to_string());
			data.components.push(component.clone());
		}
		Ok(data)
	}
	pub fn system_count(&self) -> usize {
		self.systems.len()
	}
	pub fn volume_count(&self) -> usize {
		self.volumes.len()
	}
	pub fn host_count(&self) -> usize {
		self.hosts.len()
	}
	pub fn component_count(&self) -> usize {
		self.components.len()
	}
	pub fn get_systems(&self) -> Vec<System> {
		self.systems.clone()
	}
	pub fn get_systems_filtered<P>(&self, predicate: P) -> Vec<System>
	where
		P: Fn(&System) -> bool,
	{
		self.get_systems()
			.into_iter()
			.filter(predicate)
			.collect()
	}
	pub fn get_volumes(&self) -> Vec<Volume> {
		self.volumes.clone()
	}
	pub fn get_hosts(&self) -> Vec<Host> {
		self.hosts.clone()
	}
	pub fn get_hosts_filtered<P>(&self, predicate: P) -> Vec<Host>
	where
		P: Fn(&Host) -> bool,
	{
		self.get_hosts()
			.into_iter()
			.filter(predicate)
			.collect()
	}
	pub fn get_components(&self) -> Vec<Component> {
		self.components.clone()
	}
}