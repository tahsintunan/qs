use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize SSH keys
    Init,

    /// Check required dependencies
    Check,

    /// Add a new host
    Add {
        name: String,
        host: String,
        #[arg(short, long)]
        user: Option<String>,
        #[arg(short, long, help = "Skip SSH key copy")]
        skip_key: bool,
        #[arg(short = 'd', long, help = "Make this host the default")]
        is_default: bool,
    },

    /// Remove a host
    Remove {
        name: String,
    },

    /// List all configured hosts
    List,

    /// Set the default host
    SetDefault {
        name: String,
    },

    /// Connect to a host via SSH
    Connect {
        #[arg(default_value = "default")]
        name: String,
    },

    /// Execute a command on a host
    Exec {
        #[arg(default_value = "default")]
        host: String,
        #[arg(last = true)]
        cmd: Vec<String>,
    },

    /// Send files to a host
    Send {
        source: String,
        #[arg(help = "Format: [host:]destination")]
        dest: String,
    },

    /// Get files from a host
    Get {
        #[arg(help = "Format: [host:]source")]
        source: String,
        dest: String,
    },

    /// Show connection status for a host
    Status {
        #[arg(default_value = "default")]
        name: String,
    },
}
