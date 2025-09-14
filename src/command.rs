use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize SSH keys
    Init,

    /// Check required dependencies
    Check,

    /// Add a new host
    Add {
        alias: String,
        #[arg(long)]
        host: String,
        #[arg(long)]
        user: String,
        #[arg(long, default_value = "22", help = "SSH port (default: 22)")]
        port: u16,
        #[arg(short, long, help = "Skip SSH key copy")]
        skip_key: bool,
        #[arg(short = 'd', long, help = "Make this host the default")]
        is_default: bool,
        #[arg(short = 'o', long, help = "Overwrite if alias already exists")]
        overwrite: bool,
    },

    /// Remove a host
    Remove {
        alias: String,
        #[arg(short = 'y', long = "yes", help = "Skip confirmation prompt")]
        yes: bool,
    },

    /// List all configured hosts
    List,

    /// Set the default host
    SetDefault { alias: String },

    /// Connect to a host via SSH
    Connect {
        #[arg(default_value = "default")]
        alias: String,
    },

    /// Execute a command on a host
    Exec {
        #[arg(default_value = "default")]
        alias: String,
        #[arg(last = true)]
        cmd: Vec<String>,
    },

    /// Send files to a host
    Send {
        source: String,
        #[arg(help = "Format: [alias:]destination")]
        dest: String,
    },

    /// Get files from a host
    Get {
        #[arg(help = "Format: [alias:]source")]
        source: String,
        dest: String,
    },

    /// Show connection status for a host
    Status {
        #[arg(default_value = "default")]
        alias: String,
    },
}
