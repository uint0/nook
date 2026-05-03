use anyhow::Result;
use comfy_table::{Attribute, Cell, Table, presets::UTF8_FULL};

use crate::envoy::{client::EnvoyClient, types::ScreeningCard};
use crate::profile::Profile;
use crate::util::date::{DateFormat, format_date};
use crate::util::spinner;

pub async fn run(profile: Profile, start_date: &str, end_date: &str) -> Result<()> {
    tracing::debug!(location_id = %profile.location_id, start_date, end_date, "Running booking show");
    let mut client = EnvoyClient::new(profile.token_store)?;
    let sp = spinner::start("Fetching bookings...");
    let dates = client
        .get_registration_dates(&profile.location_id, start_date, end_date)
        .await?;
    sp.finish_and_clear();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Date", "Desk", "Invite ID", "Global Bookings"]);

    for reg in &dates {
        let date = format_date(&reg.date, DateFormat::DateWithDow);

        let (invite_id, location_name) = match &reg.screening_card {
            Some(ScreeningCard::Invite(inv)) => {
                let loc = inv.location.as_ref().map(|l| l.name.as_str()).unwrap_or("");
                (inv.id.clone(), loc.to_owned())
            }
            _ => (String::from("-"), String::new()),
        };

        let desk_cell = if let Some(desk) = reg.reservations.first().and_then(|r| r.desk.as_ref()) {
            let label = if location_name.is_empty() {
                format!("{} / {}", desk.floor.name, desk.name)
            } else {
                format!("{} / {}", location_name, desk.name)
            };
            Cell::new(label)
        } else {
            Cell::new("No Booking").add_attribute(Attribute::Italic)
        };

        table.add_row(vec![
            Cell::new(&date),
            desk_cell,
            Cell::new(&invite_id),
            Cell::new(reg.people_registered),
        ]);
    }

    println!("{table}");
    Ok(())
}
