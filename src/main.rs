use clap::Parser;
use std::io::{self, Write};
use std::process::{Command, Stdio};

mod command;
mod config;
mod util;

use config::Config;

use crate::command::Commands;
use crate::config::Host;
use crate::util::{
    check_dependencies, copy_ssh_key_manual, ensure_ssh_key, setup_multiplex, ssh_target,
};

#[derive(Parser)]
#[command(name = "qs")]
#[command(about = "Quick SSH - Dead simple, zero-friction SSH wrapper")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();
    let mut config = Config::load();

    match cli.command {
        Commands::Check => {
            println!("Checking dependencies...\n");
            match check_dependencies() {
                Ok(_) => {
                    println!("✓ All required tools are installed");
                    println!("✓ ssh");
                    println!("✓ ssh-keygen");
                    println!("✓ rsync");
                }
                Err(msg) => {
                    eprintln!("{}", msg);
                    std::process::exit(1);
                }
            }
        }

        Commands::Init => {
            if let Err(msg) = check_dependencies() {
                eprintln!("{}", msg);
                eprintln!("Run 'qs check' after installing missing tools");
                std::process::exit(1);
            }

            let key_path = ensure_ssh_key();
            println!("✓ SSH key ready at {}", key_path.display());
            println!("✓ Public key at {}.pub", key_path.display());
            println!("\nNext: Add a host with 'qs add <name> <host>'");
        }

        Commands::Add {
            name,
            host,
            user,
            skip_key,
            is_default,
        } => {
            if let Err(msg) = check_dependencies() {
                eprintln!("{}", msg);
                std::process::exit(1);
            }

            let host_obj = Host { host, user };

            if !skip_key {
                copy_ssh_key_manual(&host_obj);
            }

            config.hosts.insert(name.clone(), host_obj);

            if is_default || config.default.is_none() {
                config.default = Some(name.clone());
                println!("✓ Set {} as default host", name);
            }
            config.save();
            println!("✓ Added host: {}", name);
        }

        Commands::Remove { name } => {
            if !config.hosts.contains_key(&name) {
                eprintln!("Host '{}' not found", name);
                eprintln!("\nAvailable hosts:");
                for host_name in config.hosts.keys() {
                    eprintln!("  - {}", host_name);
                }
                std::process::exit(1);
            }

            config.hosts.remove(&name);

            if config.default.as_ref() == Some(&name) {
                config.default = None;
                println!("✓ Removed default host '{}'", name);

                if config.hosts.len() == 1 {
                    let new_default = config.hosts.keys().next().unwrap().clone();
                    config.default = Some(new_default.clone());
                    println!("✓ Set '{}' as new default host", new_default);
                }
            } else {
                println!("✓ Removed host '{}'", name);
            }

            config.save();
        }

        Commands::List => {
            if config.hosts.is_empty() {
                println!("No hosts configured. Use 'qs add <name> <host>' to add one.");
                return;
            }

            println!("Configured hosts:\n");
            for (name, host) in &config.hosts {
                let default = if Some(name) == config.default.as_ref() {
                    " [default]"
                } else {
                    ""
                };
                let user = host.user.as_deref().unwrap_or("$USER");
                println!("  {}{}: {}@{}", name, default, user, host.host);
            }
        }

        Commands::Connect { name } => {
            let host = config.get_host(&name).unwrap_or_else(|| {
                eprintln!("Host '{}' not found. Use 'qs add' to configure it.", name);
                std::process::exit(1);
            });

            let mut cmd = Command::new("ssh");
            cmd.args(&setup_multiplex());
            cmd.arg(ssh_target(host));
            cmd.status().ok();
        }

        Commands::Send { source, dest } => {
            let (host_name, remote_path) = if dest.contains(':') {
                let parts: Vec<_> = dest.splitn(2, ':').collect();
                (parts[0], parts[1])
            } else {
                ("default", dest.as_str())
            };

            let host = config.get_host(host_name).unwrap_or_else(|| {
                eprintln!(
                    "Host '{}' not found. Use 'qs add' to configure it.",
                    host_name
                );
                std::process::exit(1);
            });

            println!("Sending {} → {}:{}", source, host_name, remote_path);

            let mut cmd = Command::new("rsync");
            cmd.arg("-avz");
            cmd.arg("--progress");
            cmd.arg("-e");
            cmd.arg(format!("ssh {}", setup_multiplex().join(" ")));
            cmd.arg(&source);
            cmd.arg(format!("{}:{}", ssh_target(host), remote_path));

            if !cmd.status().map(|s| s.success()).unwrap_or(false) {
                eprintln!("\n✗ Transfer failed");
                std::process::exit(1);
            }
        }

        Commands::Get { source, dest } => {
            let (host_name, remote_path) = if source.contains(':') {
                let parts: Vec<_> = source.splitn(2, ':').collect();
                (parts[0], parts[1])
            } else {
                ("default", source.as_str())
            };

            let host = config.get_host(host_name).unwrap_or_else(|| {
                eprintln!(
                    "Host '{}' not found. Use 'qs add' to configure it.",
                    host_name
                );
                std::process::exit(1);
            });

            println!("Getting {}:{} → {}", host_name, remote_path, dest);

            let mut cmd = Command::new("rsync");
            cmd.arg("-avz");
            cmd.arg("--progress");
            cmd.arg("-e");
            cmd.arg(format!("ssh {}", setup_multiplex().join(" ")));
            cmd.arg(format!("{}:{}", ssh_target(host), remote_path));
            cmd.arg(&dest);

            if !cmd.status().map(|s| s.success()).unwrap_or(false) {
                eprintln!("\n✗ Transfer failed");
                std::process::exit(1);
            }
        }

        Commands::Exec {
            host: host_name,
            cmd,
        } => {
            if cmd.is_empty() {
                eprintln!("No command specified");
                std::process::exit(1);
            }

            let host = config.get_host(&host_name).unwrap_or_else(|| {
                eprintln!(
                    "Host '{}' not found. Use 'qs add' to configure it.",
                    host_name
                );
                std::process::exit(1);
            });

            let mut ssh_cmd = Command::new("ssh");
            ssh_cmd.args(&setup_multiplex());
            ssh_cmd.arg(ssh_target(host));
            ssh_cmd.arg(cmd.join(" "));
            ssh_cmd.status().ok();
        }

        Commands::Status { name } => {
            let host = config.get_host(&name).unwrap_or_else(|| {
                eprintln!("Host '{}' not found", name);
                std::process::exit(1);
            });

            print!("Checking connection to {}... ", name);
            io::stdout().flush().unwrap();

            let mut cmd = Command::new("ssh");
            cmd.args(&setup_multiplex());
            cmd.arg("-O");
            cmd.arg("check");
            cmd.arg(ssh_target(host));
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());

            match cmd.status() {
                Ok(status) if status.success() => println!("✓ Active"),
                _ => println!("✗ No active connection"),
            }
        }

        Commands::SetDefault { name } => {
            if !config.hosts.contains_key(&name) {
                eprintln!("Host '{}' not found", name);
                eprintln!("\nAvailable hosts:");
                for host_name in config.hosts.keys() {
                    eprintln!("  - {}", host_name);
                }
                std::process::exit(1);
            }

            config.default = Some(name.clone());
            config.save();
            println!("✓ Set {} as default host", name);
        }
    }
}
