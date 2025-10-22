use serde_yaml_ng;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct System {

}

#[derive(Serialize, Deserialize)]
struct Volume {

}

#[derive(Serialize, Deserialize)]
struct Host {

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
}