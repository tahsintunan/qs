use clap::Parser;
use std::io::{self, Write};
use std::process::{Command, Stdio};

mod command;
mod config;
mod util;

use config::Config;

use crate::command::Commands;
use crate::config::Profile;
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
            println!("\nNext: Add a host with 'qs add <alias> -h <host> -u <user>'");
        }

        Commands::Add {
            alias,
            host,
            user,
            skip_key,
            is_default,
            overwrite,
        } => {
            if let Err(msg) = check_dependencies() {
                eprintln!("{}", msg);
                std::process::exit(1);
            }

            if alias == "default" {
                eprintln!("Error: 'default' is a reserved alias name");
                eprintln!("Please choose a different name for your host profile");
                std::process::exit(1);
            }

            if let Some(existing_host) = config.profiles.get(&alias) {
                if !overwrite {
                    eprintln!("Error: Alias '{}' already exists", alias);
                    eprintln!("  Host: {}", existing_host.host);
                    eprintln!("  User: {}", existing_host.user);
                    eprintln!("\nUse --overwrite to replace the existing alias");
                    std::process::exit(1);
                }
            }

            let profile = Profile { host, user };

            if !skip_key {
                copy_ssh_key_manual(&profile);
            }

            config.profiles.insert(alias.clone(), profile);

            if is_default || config.default.is_none() {
                config.default = Some(alias.clone());
                println!("✓ Set {} as default", alias);
            }
            config.save();
            println!("✓ Added alias: {}", alias);
        }

        Commands::Remove { alias } => {
            if !config.profiles.contains_key(&alias) {
                eprintln!("Alias '{}' not found", alias);
                eprintln!("\nAvailable aliases:");
                for alias_name in config.profiles.keys() {
                    eprintln!("  - {}", alias_name);
                }
                std::process::exit(1);
            }

            config.profiles.remove(&alias);

            if config.default.as_ref() == Some(&alias) {
                config.default = None;
                println!("✓ Removed default alias '{}'", alias);

                if config.profiles.len() == 1 {
                    let new_default = config.profiles.keys().next().unwrap().clone();
                    config.default = Some(new_default.clone());
                    println!("✓ Set '{}' as new default", new_default);
                }
            } else {
                println!("✓ Removed alias '{}'", alias);
            }

            config.save();
        }

        Commands::List => {
            if config.profiles.is_empty() {
                println!(
                    "No hosts configured. Use 'qs add <alias> -h <host> -u <user>' to add one."
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
                println!("  {}{}: {}@{}", alias, default, profile.user, profile.host);
            }
        }

        Commands::Connect { alias } => {
            let profile = config.get_profile(&alias).unwrap_or_else(|| {
                eprintln!("Alias '{}' not found. Use 'qs add' to configure it.", alias);
                std::process::exit(1);
            });

            let mut cmd = Command::new("ssh");
            cmd.args(&setup_multiplex());
            cmd.arg(ssh_target(profile));
            cmd.status().ok();
        }

        Commands::Send { source, dest } => {
            let (alias_name, remote_path) = if dest.contains(':') {
                let parts: Vec<_> = dest.splitn(2, ':').collect();
                (parts[0], parts[1])
            } else {
                ("default", dest.as_str())
            };

            let profile = config.get_profile(alias_name).unwrap_or_else(|| {
                eprintln!(
                    "Alias '{}' not found. Use 'qs add' to configure it.",
                    alias_name
                );
                std::process::exit(1);
            });

            println!("Sending {} → {}:{}", source, alias_name, remote_path);

            let mut cmd = Command::new("rsync");
            cmd.arg("-avz");
            cmd.arg("--progress");
            cmd.arg("-e");
            cmd.arg(format!("ssh {}", setup_multiplex().join(" ")));
            cmd.arg(&source);
            cmd.arg(format!("{}:{}", ssh_target(profile), remote_path));

            if !cmd.status().map(|s| s.success()).unwrap_or(false) {
                eprintln!("\n✗ Transfer failed");
                std::process::exit(1);
            }
        }

        Commands::Get { source, dest } => {
            let (alias_name, remote_path) = if source.contains(':') {
                let parts: Vec<_> = source.splitn(2, ':').collect();
                (parts[0], parts[1])
            } else {
                ("default", source.as_str())
            };

            let profile = config.get_profile(alias_name).unwrap_or_else(|| {
                eprintln!(
                    "Alias '{}' not found. Use 'qs add' to configure it.",
                    alias_name
                );
                std::process::exit(1);
            });

            println!("Getting {}:{} → {}", alias_name, remote_path, dest);

            let mut cmd = Command::new("rsync");
            cmd.arg("-avz");
            cmd.arg("--progress");
            cmd.arg("-e");
            cmd.arg(format!("ssh {}", setup_multiplex().join(" ")));
            cmd.arg(format!("{}:{}", ssh_target(profile), remote_path));
            cmd.arg(&dest);

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

            let profile = config.get_profile(&alias_name).unwrap_or_else(|| {
                eprintln!(
                    "Alias '{}' not found. Use 'qs add' to configure it.",
                    alias_name
                );
                std::process::exit(1);
            });

            let mut ssh_cmd = Command::new("ssh");
            ssh_cmd.args(&setup_multiplex());
            ssh_cmd.arg(ssh_target(profile));
            ssh_cmd.arg(cmd.join(" "));
            ssh_cmd.status().ok();
        }

        Commands::Status { alias } => {
            let profile = config.get_profile(&alias).unwrap_or_else(|| {
                eprintln!("Alias '{}' not found", alias);
                std::process::exit(1);
            });

            print!("Checking connection to {}... ", alias);
            io::stdout().flush().unwrap();

            let mut cmd = Command::new("ssh");
            cmd.args(&setup_multiplex());
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
                eprintln!("Alias '{}' not found", alias);
                eprintln!("\nAvailable aliases:");
                for alias_name in config.profiles.keys() {
                    eprintln!("  - {}", alias_name);
                }
                std::process::exit(1);
            }

            config.default = Some(alias.clone());
            config.save();
            println!("✓ Set {} as default", alias);
        }
    }
}
