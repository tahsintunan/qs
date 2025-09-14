use clap::Parser;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

mod command;
mod config;
mod util;

use config::Config;

use crate::command::Commands;
use crate::config::Profile;
use crate::util::{
    check_dependencies, copy_ssh_key_manual, ensure_ssh_key, remove_alias, setup_multiplex,
    ssh_target, validate_alias,
};

#[derive(Parser)]
#[command(name = "qs")]
#[command(about = "Quick SSH - Dead simple, zero-friction SSH wrapper")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();
    let mut config = Config::load().unwrap_or_else(|err| {
        eprintln!("Error loading config: {err}");
        eprintln!("Please fix the config file or remove it to start fresh.");
        std::process::exit(1);
    });

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
                    eprintln!("{msg}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Init => {
            if let Err(msg) = check_dependencies() {
                eprintln!("{msg}");
                eprintln!("Run 'qs check' after installing missing tools");
                std::process::exit(1);
            }

            let key_path = ensure_ssh_key();
            println!("✓ SSH key ready at {}", key_path.display());
            println!("✓ Public key at {}.pub", key_path.display());
            println!("\nNext: Add a host with 'qs add <alias> --host <host> --user <user>'");
        }

        Commands::Add {
            alias,
            host,
            user,
            port,
            skip_key,
            is_default,
            overwrite,
        } => {
            if let Err(msg) = check_dependencies() {
                eprintln!("{msg}");
                std::process::exit(1);
            }

            if let Err(error) = validate_alias(&alias) {
                eprintln!("Error: {error}");
                std::process::exit(1);
            }

            if let Some(existing_host) = config.profiles.get(&alias) {
                if !overwrite {
                    eprintln!("Error: Alias '{alias}' already exists");
                    eprintln!("  Host: {}", existing_host.host);
                    eprintln!("  User: {}", existing_host.user);
                    eprintln!("  Port: {}", existing_host.port);
                    eprintln!("\nUse --overwrite to replace the existing alias");
                    std::process::exit(1);
                }
            }

            let profile = Profile { host, user, port };

            if !skip_key {
                copy_ssh_key_manual(&profile);
            }

            config.profiles.insert(alias.clone(), profile);

            if is_default || config.default.is_none() {
                config.default = Some(alias.clone());
                println!("✓ Set {alias} as default");
            }
            if let Err(e) = config.save() {
                eprintln!("Error saving config: {e}");
                std::process::exit(1);
            }
            println!("✓ Added alias: {alias}");
        }

        Commands::Remove { alias, yes } => {
            if !yes {
                print!("Are you sure you want to remove '{alias}'? [y/N]: ");
                io::stdout().flush().unwrap_or(());

                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap_or_else(|_| {
                    eprintln!("Failed to read input");
                    std::process::exit(1);
                });

                let input = input.trim().to_lowercase();
                if input != "y" && input != "yes" {
                    println!("Removal cancelled");
                    return;
                }
            }

            match remove_alias(&mut config, &alias) {
                Ok(messages) => {
                    for message in messages {
                        println!("{message}");
                    }

                    if let Err(e) = config.save() {
                        eprintln!("Error saving config: {e}");
                        std::process::exit(1);
                    }
                }
                Err(error) => {
                    eprintln!("{error}");
                    eprintln!("\nAvailable aliases:");
                    for alias_name in config.profiles.keys() {
                        eprintln!("  - {alias_name}");
                    }
                    std::process::exit(1);
                }
            }
        }

        Commands::List => {
            if config.profiles.is_empty() {
                println!(
                    "No hosts configured. Use 'qs add <alias> --host <host> --user <user>' to add one."
                );
                return;
            }

            println!("Configured hosts:\n");
            for (alias, profile) in &config.profiles {
                let default = if Some(alias) == config.default.as_ref() {
                    " [default]"
                } else {
                    ""
                };
                let port_info = if profile.port != 22 {
                    format!(":{}", profile.port)
                } else {
                    String::new()
                };
                println!(
                    "  {}{}: {}@{}{}",
                    alias, default, profile.user, profile.host, port_info
                );
            }
        }

        Commands::Connect { alias } => {
            let profile = config.get_profile(&alias).unwrap_or_else(|err| {
                eprintln!("{err}");
                std::process::exit(1);
            });

            let mut cmd = Command::new("ssh");
            cmd.args(setup_multiplex());
            if profile.port != 22 {
                cmd.arg("-p").arg(profile.port.to_string());
            }
            cmd.arg(ssh_target(profile));
            cmd.status().ok();
        }

        Commands::Send { source, dest } => {
            let (alias_name, remote_path) = if dest.contains(':') {
                let parts: Vec<_> = dest.splitn(2, ':').collect();
                (parts[0], parts[1].to_string())
            } else {
                ("default", dest.clone())
            };

            let profile = config.get_profile(alias_name).unwrap_or_else(|err| {
                eprintln!("{err}");
                std::process::exit(1);
            });

            let absolute_source = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(&source);

            if !absolute_source.exists() {
                eprintln!("Error: Source file '{source}' does not exist");
                eprintln!("Looking for: {}", absolute_source.display());
                std::process::exit(1);
            }

            println!("Sending {source} → {alias_name}:{remote_path}");

            let mut cmd = Command::new("rsync");
            cmd.arg("-az");
            cmd.arg("--progress");
            cmd.arg("-e");
            let ssh_opts = if profile.port != 22 {
                format!("ssh -p {} {}", profile.port, setup_multiplex().join(" "))
            } else {
                format!("ssh {}", setup_multiplex().join(" "))
            };
            cmd.arg(ssh_opts);
            cmd.arg(absolute_source.to_string_lossy().to_string());
            cmd.arg(format!("{}:{}", ssh_target(profile), remote_path));

            if !cmd.status().map(|s| s.success()).unwrap_or(false) {
                eprintln!("\n✗ Transfer failed");
                std::process::exit(1);
            }
        }

        Commands::Get { source, dest } => {
            let (alias_name, remote_path) = if source.contains(':') {
                let parts: Vec<_> = source.splitn(2, ':').collect();
                (parts[0], parts[1].to_string())
            } else {
                ("default", source.clone())
            };

            let profile = config.get_profile(alias_name).unwrap_or_else(|err| {
                eprintln!("{err}");
                std::process::exit(1);
            });

            let absolute_dest = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(&dest);

            println!("Getting {alias_name}:{remote_path} → {dest}");

            let mut cmd = Command::new("rsync");
            cmd.arg("-az");
            cmd.arg("--progress");
            cmd.arg("-e");
            let ssh_opts = if profile.port != 22 {
                format!("ssh -p {} {}", profile.port, setup_multiplex().join(" "))
            } else {
                format!("ssh {}", setup_multiplex().join(" "))
            };
            cmd.arg(ssh_opts);
            cmd.arg(format!("{}:{}", ssh_target(profile), remote_path));
            cmd.arg(absolute_dest.to_string_lossy().to_string());

            if !cmd.status().map(|s| s.success()).unwrap_or(false) {
                eprintln!("\n✗ Transfer failed");
                std::process::exit(1);
            }
        }

        Commands::Exec {
            alias: alias_name,
            cmd,
        } => {
            if cmd.is_empty() {
                eprintln!("No command specified");
                std::process::exit(1);
            }

            let profile = config.get_profile(&alias_name).unwrap_or_else(|err| {
                eprintln!("{err}");
                std::process::exit(1);
            });

            let mut ssh_cmd = Command::new("ssh");
            ssh_cmd.args(setup_multiplex());
            if profile.port != 22 {
                ssh_cmd.arg("-p").arg(profile.port.to_string());
            }
            ssh_cmd.arg(ssh_target(profile));
            ssh_cmd.arg(cmd.join(" "));
            ssh_cmd.status().ok();
        }

        Commands::Status { alias } => {
            let profile = config.get_profile(&alias).unwrap_or_else(|err| {
                eprintln!("{err}");
                std::process::exit(1);
            });

            let display_alias = if alias == "default" {
                match config.default.as_ref() {
                    Some(actual_alias) => format!("{actual_alias} [default]"),
                    None => alias,
                }
            } else {
                alias
            };

            print!("Checking connection to {display_alias}... ");
            io::stdout().flush().unwrap_or(());

            let mut cmd = Command::new("ssh");
            cmd.args(setup_multiplex());
            if profile.port != 22 {
                cmd.arg("-p").arg(profile.port.to_string());
            }
            cmd.arg("-O");
            cmd.arg("check");
            cmd.arg(ssh_target(profile));
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());

            match cmd.status() {
                Ok(status) if status.success() => println!("✓ Active"),
                _ => println!("✗ No active connection"),
            }
        }

        Commands::SetDefault { alias } => {
            if !config.profiles.contains_key(&alias) {
                eprintln!("Alias '{alias}' not found");
                eprintln!("\nAvailable aliases:");
                for alias_name in config.profiles.keys() {
                    eprintln!("  - {alias_name}");
                }
                std::process::exit(1);
            }

            config.default = Some(alias.clone());
            if let Err(e) = config.save() {
                eprintln!("Error saving config: {e}");
                std::process::exit(1);
            }
            println!("✓ Set {alias} as default");
        }
    }
}
