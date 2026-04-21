//! Calendar tool: list upcoming tax deadlines and scheduled events.

use chrono::Utc;
use serde::{Deserialize, Serialize};

use ledger_core::calendar::BusinessCalendar;

/// Request to list calendar events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCalendarEventsRequest {
    /// Filter by jurisdiction: "US" | "AU" | None (returns both).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction: Option<String>,
}

/// A calendar event row with computed next due date.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEventRow {
    pub id: String,
    pub description: String,
    /// ISO date YYYY-MM-DD
    pub next_due_date: String,
    pub jurisdiction: String,
    pub tags: Vec<String>,
}

/// List calendar events (tax deadlines and scheduled operations).
///
/// Calls `BusinessCalendar::us_tax_defaults()` and `BusinessCalendar::au_tax_defaults()`,
/// filters by jurisdiction if requested, and computes next due dates.
/// Results are sorted by next_due_date ascending.
pub fn list_calendar_events(req: ListCalendarEventsRequest) -> Vec<CalendarEventRow> {
    let mut calendar = BusinessCalendar::us_tax_defaults();
    calendar.merge(BusinessCalendar::au_tax_defaults());

    let today = Utc::now().naive_utc().date();

    // Filter by jurisdiction if specified.
    let mut events: Vec<_> = calendar
        .events
        .iter()
        .filter(|event| {
            if let Some(ref jurisdiction_filter) = req.jurisdiction {
                if let Some(event_jurisdiction) = &event.jurisdiction {
                    event_jurisdiction.code() == jurisdiction_filter
                } else {
                    false
                }
            } else {
                true
            }
        })
        .filter_map(|event| {
            let next_due = BusinessCalendar::next_due(&event.recurrence, today)?;
            let jurisdiction = event
                .jurisdiction
                .map(|j| j.code().to_string())
                .unwrap_or_else(|| "UNKNOWN".to_string());

            Some(CalendarEventRow {
                id: event.id.clone(),
                description: event.description.clone(),
                next_due_date: next_due.format("%Y-%m-%d").to_string(),
                jurisdiction,
                tags: event.tags.clone(),
            })
        })
        .collect();

    // Sort by next_due_date ascending.
    events.sort_by(|a, b| a.next_due_date.cmp(&b.next_due_date));

    events
}
