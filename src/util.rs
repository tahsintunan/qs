use std::fs;

use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::config::Host;

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

fn missing_tools_msg(missing: Vec<&str>) -> String {
    let mut msg = String::from("Missing required tools:\n\n");

    let os = std::env::consts::OS;

    let (install_cmd, ssh_package) = match os {
        "macos" => ("brew install", "openssh"),
        "linux" => {
            if check_command("apt") {
                ("sudo apt install", "openssh-client")
            } else if check_command("yum") {
                ("sudo yum install", "openssh-clients")
            } else if check_command("dnf") {
                ("sudo dnf install", "openssh-clients")
            } else if check_command("pacman") {
                ("sudo pacman -S", "openssh")
            } else if check_command("zypper") {
                ("sudo zypper install", "openssh")
            } else {
                ("check your distribution's package manager for", "openssh")
            }
        }
        "windows" => ("install via WSL or Windows OpenSSH:", "openssh"),
        _ => ("check your package manager for", "openssh"),
    };

    for cmd in missing {
        let package = match cmd {
            "ssh" | "ssh-keygen" => ssh_package,
            "rsync" => "rsync",
            _ => cmd,
        };
        msg.push_str(&format!("  {} - Install with:\n", cmd));
        msg.push_str(&format!("    {} {}\n\n", install_cmd, package));
    }

    msg
}

fn check_command(cmd: &str) -> bool {
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

pub fn ssh_target(host: &Host) -> String {
    if let Some(user) = &host.user {
        format!("{}@{}", user, host.host)
    } else {
        host.host.clone()
    }
}

pub fn ensure_ssh_key() -> PathBuf {
    let ssh_dir = dirs::home_dir().unwrap().join(".ssh");
    fs::create_dir_all(&ssh_dir).ok();

    let key_path = ssh_dir.join("id_ed25519");

    if !key_path.exists() {
        println!("No SSH key found. Creating one...");

        let output = Command::new("ssh-keygen")
            .args(&["-t", "ed25519", "-f"])
            .arg(&key_path)
            .args(&["-N", ""]) // Empty passphrase
            .args(&["-C", "qs-tool"])
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

pub fn copy_ssh_key_manual(host: &Host) {
    let key_path = ensure_ssh_key();
    let pub_key_path = format!("{}.pub", key_path.display());

    // Read the public key
    let pubkey = fs::read_to_string(&pub_key_path).expect("Failed to read public key");

    println!("Copying SSH key to {}...", host.host);
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
    cmd.arg(ssh_target(host));
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
                ssh_target(host)
            );
        }
    }
}
