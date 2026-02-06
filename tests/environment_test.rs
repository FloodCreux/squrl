use std::path::PathBuf;

use indexmap::IndexMap;
use squrl::models::environment::Environment;

#[test]
fn default_environment_has_empty_name() {
	let env = Environment::default();
	assert_eq!(env.name, "");
}

#[test]
fn default_environment_has_empty_values() {
	let env = Environment::default();
	assert!(env.values.is_empty());
}

#[test]
fn default_environment_has_empty_path() {
	let env = Environment::default();
	assert_eq!(env.path, PathBuf::new());
}

#[test]
fn environment_values_preserve_insertion_order() {
	let mut env = Environment::default();
	env.values.insert("third".to_string(), "3".to_string());
	env.values.insert("first".to_string(), "1".to_string());
	env.values.insert("second".to_string(), "2".to_string());

	let keys: Vec<&String> = env.values.keys().collect();
	assert_eq!(keys, vec!["third", "first", "second"]);
}

#[test]
fn environment_serializes_to_json() {
	let mut env = Environment {
		name: "test".to_string(),
		values: IndexMap::new(),
		path: PathBuf::from("/tmp/test.env"),
	};
	env.values
		.insert("API_KEY".to_string(), "secret".to_string());

	let json = serde_json::to_string(&env).unwrap();
	assert!(json.contains("\"name\":\"test\""));
	assert!(json.contains("\"API_KEY\":\"secret\""));
}

#[test]
fn environment_deserializes_from_json() {
	let json = r#"{"name":"prod","values":{"HOST":"localhost","PORT":"8080"},"path":"/tmp/prod.env"}"#;
	let env: Environment = serde_json::from_str(json).unwrap();

	assert_eq!(env.name, "prod");
	assert_eq!(env.values.len(), 2);
	assert_eq!(env.values["HOST"], "localhost");
	assert_eq!(env.values["PORT"], "8080");
}

#[test]
fn environment_clone_is_independent() {
	let mut env = Environment {
		name: "original".to_string(),
		values: IndexMap::new(),
		path: PathBuf::from("/tmp/test.env"),
	};
	env.values.insert("key".to_string(), "value".to_string());

	let mut cloned = env.clone();
	cloned.name = "cloned".to_string();
	cloned
		.values
		.insert("new_key".to_string(), "new_value".to_string());

	assert_eq!(env.name, "original");
	assert_eq!(env.values.len(), 1);
	assert_eq!(cloned.name, "cloned");
	assert_eq!(cloned.values.len(), 2);
}
