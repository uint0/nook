use clap::Subcommand;

#[derive(Subcommand)]
pub enum BookingCommands {
    /// Create a new booking
    Create {
        /// Backfill all available dates in the next 14 days
        #[arg(long, default_value_t = false)]
        backfill: bool,

        /// Date of the booking (e.g. "latest" or "2026-05-02")
        #[arg(long)]
        date: String,

        /// Specific desk to book, e.g. "raw:<desk_id>".
        #[arg(long)]
        desk: Option<String>,
    },
    /// Check in to an existing booking
    #[command(name = "check-in")]
    CheckIn {
        /// Date of the booking to check in to (e.g. "latest" or "2026-05-02")
        #[arg(long)]
        date: String,
    },
    /// Show bookings for a profile
    Show {
        /// Start date for the range (default: local midnight today)
        #[arg(long)]
        start_date: Option<String>,

        /// End date for the range (default: 14 days from start)
        #[arg(long)]
        end_date: Option<String>,
    },
}
