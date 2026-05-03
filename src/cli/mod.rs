pub mod booking;
pub mod profile;

use clap::{Parser, Subcommand};

/// Nook — a tool for managing Envoy bookings and check-ins
#[derive(Parser)]
#[command(name = "nook", version, about, long_about = None)]
pub struct Cli {
    /// Profile to use
    #[arg(long, global = true, default_value = "default")]
    pub profile: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage profiles
    Profile {
        #[command(subcommand)]
        command: profile::ProfileCommands,
    },
    /// Manage bookings
    Booking {
        #[command(subcommand)]
        command: booking::BookingCommands,
    },
}
