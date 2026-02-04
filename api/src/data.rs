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

// The format of data to expose publically
pub struct Data {
	systems: Vec<System>,
	volumes: Vec<Volume>,
	hosts: Vec<Host>,
	components: Vec<Component>,
}


impl Data {
	pub fn from_dir<P: AsRef<Path>>(path: P) -> Result<Data, Box<dyn std::error::Error>> {
		let mut data = Data {
			systems: vec![],
			volumes: vec![],
			hosts: vec![],
			components: vec![],
		};
		let systems_file = std::fs::File::open(path.as_ref().join("systems.yaml"))?;
		let mut raw_systems: HashMap<String, System> = serde_yaml_ng::from_reader(systems_file)?;
		for (id, system) in raw_systems.iter_mut() {
			system.id = Some(id.to_string());
			data.systems.push(system.clone());
		}
		data.systems.sort_by(|d1, d2| d1.id.cmp(&d2.id));

		let volumes_file = std::fs::File::open(path.as_ref().join("volumes.yaml"))?;
		let mut raw_volumes: HashMap<String, Volume> = serde_yaml_ng::from_reader(volumes_file)?;
		for (id, volume) in raw_volumes.iter_mut() {
			volume.id = Some(id.to_string());
			data.volumes.push(volume.clone());
		}
		data.volumes.sort_by(|d1, d2| d1.id.cmp(&d2.id));

		let hosts_file = std::fs::File::open(path.as_ref().join("hosts.yaml"))?;
		let mut raw_hosts: HashMap<String, Host> = serde_yaml_ng::from_reader(hosts_file)?;
		for (id, host) in raw_hosts.iter_mut() {
			host.id = Some(id.to_string());
			data.hosts.push(host.clone());
		}
		data.hosts.sort_by(|d1, d2| d1.id.cmp(&d2.id));

		let components_file = std::fs::File::open(path.as_ref().join("components.yaml"))?;
		let mut raw_components: HashMap<String, Component> = serde_yaml_ng::from_reader(components_file)?;
		for (id, component) in raw_components.iter_mut() {
			component.id = Some(id.to_string());
			data.components.push(component.clone());
		}
		data.components.sort_by(|d1, d2| d1.id.cmp(&d2.id));

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