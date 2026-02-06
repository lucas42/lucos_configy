use lucos_configy_api::data::Data;
use std::path::Path;

#[test]
fn validate_config_files() {
	// The test is expected to be run from the `api` directory
	// The config files are in `../config` relative to the `api` directory
	let config_path = Path::new("..").join("config");
	
	// If that doesn't exist (e.g. running from root), try "config"
	let final_path = if config_path.exists() {
		config_path
	} else {
		Path::new("config").to_path_buf()
	};

	println!("Validating config in: {:?}", final_path);
	
	let result = Data::from_dir(final_path);
	
	match result {
		Ok(data) => {
			println!("Successfully loaded config:");
			println!("  Systems: {}", data.system_count());
			println!("  Volumes: {}", data.volume_count());
			println!("  Hosts: {}", data.host_count());
			println!("  Components: {}", data.component_count());
			
			// Basic sanity checks
			assert!(data.system_count() > 0, "No systems found in config");
			assert!(data.host_count() > 0, "No hosts found in config");
		}
		Err(e) => {
			panic!("Failed to validate config: {:?}", e);
		}
	}
}
