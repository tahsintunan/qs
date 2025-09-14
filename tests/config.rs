use qs::config::{Config, Profile};
use std::{collections::HashMap, fs};
use tempfile::TempDir;

#[test]
fn get_profile_with_empty_config() {
    let config = Config {
        default: None,
        profiles: HashMap::new(),
    };

    let result = config.get_profile("default");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "No host found. Add a host: 'qs add <alias> --host <host> --user <user>'"
    );

    let result = config.get_profile("nonexistent");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Alias 'nonexistent' doesn't exist");
}

#[test]
fn get_profile_with_no_default_set() {
    let mut profiles = HashMap::new();
    profiles.insert(
        "server1".to_string(),
        Profile {
            host: "192.168.1.100".to_string(),
            user: "testuser".to_string(),
            port: 22,
        },
    );

    let config = Config {
        default: None,
        profiles,
    };

    let result = config.get_profile("default");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "No default set. Use 'qs set-default <alias>'"
    );

    let result = config.get_profile("server1");
    assert!(result.is_ok());
    let profile = result.unwrap();
    assert_eq!(profile.host, "192.168.1.100");
    assert_eq!(profile.user, "testuser");
}

#[test]
fn get_profile_with_valid_default() {
    let mut profiles = HashMap::new();
    profiles.insert(
        "server1".to_string(),
        Profile {
            host: "192.168.1.100".to_string(),
            user: "testuser".to_string(),
            port: 22,
        },
    );

    let config = Config {
        default: Some("server1".to_string()),
        profiles,
    };

    let result = config.get_profile("default");
    assert!(result.is_ok());
    let profile = result.unwrap();
    assert_eq!(profile.host, "192.168.1.100");
    assert_eq!(profile.user, "testuser");
}

#[test]
fn get_profile_with_invalid_default() {
    let mut profiles = HashMap::new();
    profiles.insert(
        "server1".to_string(),
        Profile {
            host: "192.168.1.100".to_string(),
            user: "testuser".to_string(),
            port: 22,
        },
    );

    let config = Config {
        default: Some("nonexistent".to_string()),
        profiles,
    };

    let result = config.get_profile("default");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Default alias 'nonexistent' not found");
}

#[test]
fn get_profile_direct_alias() {
    let mut profiles = HashMap::new();
    profiles.insert(
        "server1".to_string(),
        Profile {
            host: "192.168.1.100".to_string(),
            user: "user1".to_string(),
            port: 22,
        },
    );
    profiles.insert(
        "server2".to_string(),
        Profile {
            host: "192.168.1.101".to_string(),
            user: "user2".to_string(),
            port: 22,
        },
    );

    let config = Config {
        default: Some("server1".to_string()),
        profiles,
    };

    let result = config.get_profile("server2");
    assert!(result.is_ok());
    let profile = result.unwrap();
    assert_eq!(profile.host, "192.168.1.101");
    assert_eq!(profile.user, "user2");

    let result = config.get_profile("server3");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Alias 'server3' doesn't exist");
}

fn create_test_config() -> Config {
    let mut profiles = HashMap::new();
    profiles.insert(
        "test".to_string(),
        Profile {
            host: "example.com".to_string(),
            user: "testuser".to_string(),
            port: 22,
        },
    );

    Config {
        default: Some("test".to_string()),
        profiles,
    }
}

#[test]
fn save_and_load_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config = create_test_config();
    config.save_to(config_path.clone()).unwrap();

    let loaded_config = Config::load_from(config_path).unwrap();

    assert_eq!(loaded_config.default, config.default);
    assert_eq!(loaded_config.profiles.len(), config.profiles.len());

    let profile = loaded_config.profiles.get("test").unwrap();
    assert_eq!(profile.host, "example.com");
    assert_eq!(profile.user, "testuser");
}

#[test]
fn load_nonexistent_file_returns_default() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("nonexistent.toml");

    assert!(!config_path.exists());

    let loaded_config = Config::load_from(config_path).unwrap();
    assert!(loaded_config.default.is_none());
    assert!(loaded_config.profiles.is_empty());
}

#[test]
fn load_invalid_toml_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");

    // Write invalid TOML
    fs::write(&config_path, "invalid toml [").unwrap();

    let result = Config::load_from(config_path);
    assert!(result.is_err());
}

#[test]
fn save_creates_directory_if_not_exists() {
    let temp_dir = TempDir::new().unwrap();
    let nested_path = temp_dir
        .path()
        .join("nested")
        .join("dir")
        .join("config.toml");

    // Ensure parent directories don't exist
    assert!(!nested_path.parent().unwrap().exists());

    let config = create_test_config();
    config.save_to(nested_path.clone()).unwrap();

    assert!(nested_path.exists());

    // Verify content is correct
    let loaded_config = Config::load_from(nested_path).unwrap();
    assert_eq!(loaded_config.default, Some("test".to_string()));
}

#[test]
fn round_trip_with_multiple_profiles() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("multi.toml");

    let mut profiles = HashMap::new();
    profiles.insert(
        "server1".to_string(),
        Profile {
            host: "192.168.1.100".to_string(),
            user: "admin".to_string(),
            port: 22,
        },
    );
    profiles.insert(
        "server2".to_string(),
        Profile {
            host: "10.0.0.50".to_string(),
            user: "deploy".to_string(),
            port: 22,
        },
    );

    let config = Config {
        default: Some("server1".to_string()),
        profiles,
    };

    config.save_to(config_path.clone()).unwrap();
    let loaded_config = Config::load_from(config_path).unwrap();

    // Verify
    assert_eq!(loaded_config.default, Some("server1".to_string()));
    assert_eq!(loaded_config.profiles.len(), 2);

    let server1 = loaded_config.profiles.get("server1").unwrap();
    assert_eq!(server1.host, "192.168.1.100");
    assert_eq!(server1.user, "admin");

    let server2 = loaded_config.profiles.get("server2").unwrap();
    assert_eq!(server2.host, "10.0.0.50");
    assert_eq!(server2.user, "deploy");
}
