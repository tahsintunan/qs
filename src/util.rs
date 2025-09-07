use std::fs;

use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::config::{Config, Profile};

pub fn check_dependencies() -> Result<(), String> {
    let mut missing = Vec::new();
    let required = vec!["ssh", "ssh-keygen", "rsync"];

    for cmd in required {
        if !check_command(cmd) {
            missing.push(cmd);
        }
    }

    if !missing.is_empty() {
        return Err(missing_tools_msg(missing));
    }

    Ok(())
}

pub fn missing_tools_msg(missing: Vec<&str>) -> String {
    let mut msg = String::from("Missing required tools:\n\n");

    for cmd in missing {
        msg.push_str(&format!("  • {cmd}\n"));
    }

    msg.push_str("\nPlease install these tools using your system's package manager.\n");
    msg.push_str("Common packages:\n");
    msg.push_str("  • ssh/ssh-keygen: Usually in 'openssh' or 'openssh-client'\n");
    msg.push_str("  • rsync: Usually in 'rsync'\n");

    msg
}

pub fn check_command(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn setup_multiplex() -> Vec<String> {
    let socket_dir = dirs::home_dir().unwrap().join(".ssh").join("sockets");
    fs::create_dir_all(&socket_dir).ok();

    vec![
        "-o".into(),
        "ControlMaster=auto".into(),
        "-o".into(),
        format!("ControlPath={}/%C", socket_dir.display()),
        "-o".into(),
        "ControlPersist=10m".into(),
    ]
}

pub fn ssh_target(profile: &Profile) -> String {
    format!("{}@{}", profile.user, profile.host)
}

pub fn ensure_ssh_key() -> PathBuf {
    let ssh_dir = dirs::home_dir().unwrap().join(".ssh");
    fs::create_dir_all(&ssh_dir).ok();

    let key_path = ssh_dir.join("id_ed25519");

    if !key_path.exists() {
        println!("No SSH key found. Creating one...");

        let output = Command::new("ssh-keygen")
            .args(["-t", "ed25519", "-f"])
            .arg(&key_path)
            .args(["-N", ""]) // Empty passphrase
            .args(["-C", "qs-tool"])
            .output()
            .expect("Failed to generate SSH key");

        if output.status.success() {
            println!("✓ SSH key created at {}", key_path.display());
        } else {
            eprintln!("Failed to create SSH key");
            std::process::exit(1);
        }
    }

    key_path
}

pub fn validate_alias(alias: &str) -> Result<(), String> {
    if alias.is_empty() {
        return Err("Alias cannot be empty".to_string());
    }

    if alias == "default" {
        return Err("'default' is a reserved alias name".to_string());
    }

    if alias.contains(':') {
        return Err("Alias cannot contain ':' character".to_string());
    }

    if alias.contains('/') {
        return Err("Alias cannot contain '/' character".to_string());
    }

    if alias.starts_with('-') {
        return Err("Alias cannot start with '-'".to_string());
    }

    Ok(())
}

pub fn remove_alias(config: &mut Config, alias: &str) -> Result<Vec<String>, String> {
    let mut messages = Vec::new();

    if !config.profiles.contains_key(alias) {
        return Err(format!("Alias '{alias}' not found"));
    }

    config.profiles.remove(alias);

    if config.default.as_ref() == Some(&alias.to_string()) {
        config.default = None;
        messages.push(format!("✓ Removed default alias '{alias}'"));

        match config.profiles.len() {
            0 => messages.push("  No aliases remaining".to_string()),
            1 => {
                let new_default = config.profiles.keys().next().unwrap().clone();
                config.default = Some(new_default.clone());
                messages.push(format!("✓ Set '{new_default}' as new default"));
            }
            _ => messages
                .push("  No default set. Use 'qs set-default <alias>' to set one.".to_string()),
        }
    } else {
        messages.push(format!("✓ Removed alias '{alias}'"));
    }

    Ok(messages)
}

pub fn copy_ssh_key_manual(profile: &Profile) {
    let key_path = ensure_ssh_key();
    let pub_key_path = format!("{}.pub", key_path.display());

    // Read the public key
    let pubkey = fs::read_to_string(&pub_key_path).expect("Failed to read public key");

    println!("Copying SSH key to {}...", profile.host);
    println!("You'll need to enter the password for this host:");

    // Create the SSH command to add the key
    let remote_cmd = format!(
        "mkdir -p ~/.ssh && \
         chmod 700 ~/.ssh && \
         echo '{}' >> ~/.ssh/authorized_keys && \
         chmod 600 ~/.ssh/authorized_keys && \
         echo 'Key added successfully'",
        pubkey.trim()
    );

    let mut cmd = Command::new("ssh");
    cmd.arg("-o").arg("StrictHostKeyChecking=accept-new");
    cmd.arg(ssh_target(profile));
    cmd.arg(remote_cmd);

    match cmd.status() {
        Ok(status) if status.success() => {
            println!("✓ SSH key copied successfully. No password needed from now on!");
        }
        _ => {
            eprintln!("⚠ Failed to copy SSH key. You can try manually with:");
            eprintln!(
                "  cat {} | ssh {} 'cat >> ~/.ssh/authorized_keys'",
                pub_key_path,
                ssh_target(profile)
            );
        }
    }
}
