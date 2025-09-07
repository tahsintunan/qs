use qs::config::{Config, Profile};
use qs::util::remove_alias;
use std::collections::HashMap;

#[test]
fn remove_non_existent_alias() {
    let mut config = Config {
        default: Some("server1".to_string()),
        profiles: {
            let mut profiles = HashMap::new();
            profiles.insert(
                "server1".to_string(),
                Profile {
                    host: "192.168.1.1".to_string(),
                    user: "admin".to_string(),
                },
            );
            profiles
        },
    };

    let result = remove_alias(&mut config, "nonexistent");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Alias 'nonexistent' not found");
    assert_eq!(config.profiles.len(), 1);
    assert_eq!(config.default, Some("server1".to_string()));
}

#[test]
fn remove_non_default_alias() {
    let mut config = Config {
        default: Some("server1".to_string()),
        profiles: {
            let mut profiles = HashMap::new();
            profiles.insert(
                "server1".to_string(),
                Profile {
                    host: "192.168.1.1".to_string(),
                    user: "admin".to_string(),
                },
            );
            profiles.insert(
                "server2".to_string(),
                Profile {
                    host: "192.168.1.2".to_string(),
                    user: "admin".to_string(),
                },
            );
            profiles
        },
    };

    let result = remove_alias(&mut config, "server2");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["✓ Removed alias 'server2'"]);
    assert_eq!(config.profiles.len(), 1);
    assert_eq!(config.default, Some("server1".to_string()));
}

#[test]
fn remove_default_with_no_aliases_remaining() {
    let mut config = Config {
        default: Some("server1".to_string()),
        profiles: {
            let mut profiles = HashMap::new();
            profiles.insert(
                "server1".to_string(),
                Profile {
                    host: "192.168.1.1".to_string(),
                    user: "admin".to_string(),
                },
            );
            profiles
        },
    };

    let result = remove_alias(&mut config, "server1");
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![
            "✓ Removed default alias 'server1'",
            "  No aliases remaining"
        ]
    );
    assert_eq!(config.profiles.len(), 0);
    assert_eq!(config.default, None);
}

#[test]
fn remove_default_with_one_alias_remaining() {
    let mut config = Config {
        default: Some("server1".to_string()),
        profiles: {
            let mut profiles = HashMap::new();
            profiles.insert(
                "server1".to_string(),
                Profile {
                    host: "192.168.1.1".to_string(),
                    user: "admin".to_string(),
                },
            );
            profiles.insert(
                "server2".to_string(),
                Profile {
                    host: "192.168.1.2".to_string(),
                    user: "admin".to_string(),
                },
            );
            profiles
        },
    };

    let result = remove_alias(&mut config, "server1");
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![
            "✓ Removed default alias 'server1'",
            "✓ Set 'server2' as new default"
        ]
    );
    assert_eq!(config.profiles.len(), 1);
    assert_eq!(config.default, Some("server2".to_string()));
    assert!(config.profiles.contains_key("server2"));
}

#[test]
fn remove_default_with_multiple_aliases_remaining() {
    let mut config = Config {
        default: Some("server1".to_string()),
        profiles: {
            let mut profiles = HashMap::new();
            profiles.insert(
                "server1".to_string(),
                Profile {
                    host: "192.168.1.1".to_string(),
                    user: "admin".to_string(),
                },
            );
            profiles.insert(
                "server2".to_string(),
                Profile {
                    host: "192.168.1.2".to_string(),
                    user: "admin".to_string(),
                },
            );
            profiles.insert(
                "server3".to_string(),
                Profile {
                    host: "192.168.1.3".to_string(),
                    user: "admin".to_string(),
                },
            );
            profiles
        },
    };

    let result = remove_alias(&mut config, "server1");
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![
            "✓ Removed default alias 'server1'",
            "  No default set. Use 'qs set-default <alias>' to set one."
        ]
    );
    assert_eq!(config.profiles.len(), 2);
    assert_eq!(config.default, None);
    assert!(config.profiles.contains_key("server2"));
    assert!(config.profiles.contains_key("server3"));
}

#[test]
fn remove_sequence_leaves_one_default() {
    let mut config = Config {
        default: Some("server1".to_string()),
        profiles: {
            let mut profiles = HashMap::new();
            profiles.insert(
                "server1".to_string(),
                Profile {
                    host: "192.168.1.1".to_string(),
                    user: "admin".to_string(),
                },
            );
            profiles.insert(
                "server2".to_string(),
                Profile {
                    host: "192.168.1.2".to_string(),
                    user: "admin".to_string(),
                },
            );
            profiles.insert(
                "server3".to_string(),
                Profile {
                    host: "192.168.1.3".to_string(),
                    user: "admin".to_string(),
                },
            );
            profiles
        },
    };

    let result = remove_alias(&mut config, "server3");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["✓ Removed alias 'server3'"]);
    assert_eq!(config.default, Some("server1".to_string()));

    let result = remove_alias(&mut config, "server1");
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![
            "✓ Removed default alias 'server1'",
            "✓ Set 'server2' as new default"
        ]
    );
    assert_eq!(config.default, Some("server2".to_string()));
    assert_eq!(config.profiles.len(), 1);
}
