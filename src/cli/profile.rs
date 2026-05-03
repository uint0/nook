use clap::Subcommand;

#[derive(Subcommand)]
pub enum ProfileCommands {
    /// Create a new profile
    Create,
    /// Force a token refresh for a profile
    Refresh,
    /// Show profile info and token expiry times
    Info,
}
