use lucos_configy_api::data::Data;
use std::path::{Path, PathBuf};
use serde_yaml_ng::Value;

fn config_dir() -> PathBuf {
	let config_path = Path::new("..").join("config");
	if config_path.exists() {
		config_path
	} else {
		Path::new("config").to_path_buf()
	}
}

fn load_test_data() -> Data {
	Data::from_dir(&config_dir()).expect("Failed to load config")
}

#[test]
fn validate_config_files() {
	let data = load_test_data();

	println!("Successfully loaded config:");
	println!("  Systems: {}", data.system_count());
	println!("  Volumes: {}", data.volume_count());
	println!("  Hosts: {}", data.host_count());
	println!("  Components: {}", data.component_count());
	println!("  Scripts: {}", data.script_count());

	// Basic sanity checks
	assert!(data.system_count() > 0, "No systems found in config");
	assert!(data.host_count() > 0, "No hosts found in config");

	// A system with a domain must have at most one host, to prevent silent DNS misconfiguration
	for system in data.get_systems() {
		if system.domain.is_some() {
			assert!(
				system.hosts.len() <= 1,
				"System with domain must have at most one host, but has {}: {:?}",
				system.hosts.len(),
				system.domain
			);
		}
	}
}

#[test]
fn repository_ids_are_unique_across_types() {
	let data = load_test_data();
	let ids = data.get_all_repository_ids();

	let mut seen: std::collections::HashMap<String, &str> = std::collections::HashMap::new();
	for (id, repo_type) in &ids {
		if let Some(existing_type) = seen.get(id.as_str()) {
			panic!(
				"Repository id {:?} appears in both {:?} and {:?}",
				id, existing_type, repo_type
			);
		}
		seen.insert(id.clone(), repo_type);
	}
}

#[test]
fn config_files_are_sorted_alphabetically() {
	let config_files = ["systems.yaml", "volumes.yaml", "hosts.yaml", "components.yaml", "scripts.yaml"];

	for filename in &config_files {
		let file_path = config_dir().join(filename);
		let file = std::fs::File::open(&file_path)
			.unwrap_or_else(|e| panic!("Failed to open {:?}: {}", file_path, e));

		let value: Value = serde_yaml_ng::from_reader(file)
			.unwrap_or_else(|e| panic!("Failed to parse {:?}: {}", file_path, e));

		let mapping = value.as_mapping()
			.unwrap_or_else(|| panic!("{} is not a YAML mapping", filename));

		let keys: Vec<&str> = mapping.keys()
			.map(|k| k.as_str().unwrap_or_else(|| panic!("Non-string key in {}", filename)))
			.collect();

		let mut sorted_keys = keys.clone();
		sorted_keys.sort();

		assert_eq!(
			keys, sorted_keys,
			"{} keys are not in alphabetical order. Got: {:?}",
			filename, keys
		);
	}
}
