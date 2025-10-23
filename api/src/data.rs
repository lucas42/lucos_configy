use serde_yaml_ng;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::vec::Vec;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone)]
pub struct System {
	id: Option<String>, // This is optional because the raw yaml specifies it as than key, rather than as an attribute
	#[serde(skip_serializing_if = "Option::is_none")]
	pub domain: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub http_port: Option<u16>, // TCP ports are 16-bit integers
	pub hosts: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Volume {
	id: Option<String>, // This is optional because the raw yaml specifies it as than key, rather than as an attribute
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub recreate_effort: Option<String>,
	#[serde(default)]
	pub skip_backup: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Host {
	id: Option<String>, // This is optional because the raw yaml specifies it as than key, rather than as an attribute
	#[serde(skip_serializing_if = "Option::is_none")]
	pub domain: Option<String>,
}

#[derive(Deserialize)]
pub struct Data {
	systems: HashMap<String, System>,
	volumes: HashMap<String, Volume>,
	hosts: HashMap<String, Host>,
}


impl Data {
	pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Data, Box<dyn std::error::Error>> {
		let file = std::fs::File::open(path)?;
		let data: Data = serde_yaml_ng::from_reader(file)?;
		Ok(data)
	}
	pub fn system_count(&self) -> usize {
		self.systems.keys().len()
	}
	pub fn volume_count(&self) -> usize {
		self.volumes.keys().len()
	}
	pub fn host_count(&self) -> usize {
		self.hosts.keys().len()
	}
	pub fn get_systems(&self) -> Vec<System> {
		let mut systems = vec![];
		for (id, orig_system) in (&self.systems).into_iter() {
			let mut system = orig_system.clone();
			system.id = Some(id.to_string());
			systems.push(system);
		}
		systems
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
		let mut volumes = vec![];
		for (id, orig_volume) in (&self.volumes).into_iter() {
			let mut volume = orig_volume.clone();
			volume.id = Some(id.to_string());
			volumes.push(volume);
		}
		volumes
	}
	pub fn get_hosts(&self) -> Vec<Host> {
		let mut hosts = vec![];
		for (id, orig_host) in (&self.hosts).into_iter() {
			let mut host = orig_host.clone();
			host.id = Some(id.to_string());
			hosts.push(host);
		}
		hosts
	}
}