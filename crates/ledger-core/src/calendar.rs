//! Business calendar and scheduler for tax deadlines and recurring ledger operations.
//!
//! The pure date-arithmetic portions (`next_due`, `upcoming`, `events_due_on`) are
//! fully functional. TOML loading delegates to `serde` + the `toml` crate.

use chrono::{Datelike, Days, Months, NaiveDate};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ledger_ops::OperationKind;
use crate::legal::Jurisdiction;

// ---------------------------------------------------------------------------
// Recurrence rule
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecurrenceRule {
    /// Fire on the Nth day of every month.
    Monthly { day_of_month: u8 },
    /// Fire on the same day in April, June, September, January (US estimated tax).
    QuarterlyEstimated { day: u8 },
    /// Fire once per year on month/day.
    Annual { month: u8, day: u8 },
    /// Fire every N days.
    EveryNDays { n: u32 },
    /// Cron expression — stored but evaluation is STUB (returns None from next_due).
    CronExpr(String),
}

// ---------------------------------------------------------------------------
// Scheduled event
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub id: String,
    pub description: String,
    pub recurrence: RecurrenceRule,
    pub operation: OperationKind,
    pub jurisdiction: Option<Jurisdiction>,
    pub enabled: bool,
    pub last_run: Option<NaiveDate>,
    pub tags: Vec<String>,
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum CalendarError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    Toml(String),
    #[error("invalid date: {0}")]
    InvalidDate(String),
}

// ---------------------------------------------------------------------------
// TOML shim types for loading
// ---------------------------------------------------------------------------

/// Mirror of [`ScheduledEvent`] but with string dates for TOML parsing.
#[derive(Debug, Deserialize)]
struct TomlScheduledEvent {
    id: String,
    description: String,
    recurrence: RecurrenceRule,
    operation: OperationKind,
    jurisdiction: Option<Jurisdiction>,
    #[serde(default = "default_true")]
    enabled: bool,
    last_run: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct TomlCalendar {
    name: String,
    #[serde(default)]
    events: Vec<TomlScheduledEvent>,
}

impl TryFrom<TomlScheduledEvent> for ScheduledEvent {
    type Error = CalendarError;

    fn try_from(t: TomlScheduledEvent) -> Result<Self, Self::Error> {
        let last_run = t
            .last_run
            .as_deref()
            .map(|s| {
                NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map_err(|e| CalendarError::InvalidDate(format!("{s}: {e}")))
            })
            .transpose()?;

        Ok(ScheduledEvent {
            id: t.id,
            description: t.description,
            recurrence: t.recurrence,
            operation: t.operation,
            jurisdiction: t.jurisdiction,
            enabled: t.enabled,
            last_run,
            tags: t.tags,
        })
    }
}

// ---------------------------------------------------------------------------
// BusinessCalendar
// ---------------------------------------------------------------------------

pub struct BusinessCalendar {
    pub name: String,
    pub events: Vec<ScheduledEvent>,
}

impl BusinessCalendar {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            events: Vec::new(),
        }
    }

    /// Load from a TOML file. The TOML format mirrors [`ScheduledEvent`] fields.
    pub fn from_toml_file(path: &std::path::Path) -> Result<Self, CalendarError> {
        let raw = std::fs::read_to_string(path)?;
        let parsed: TomlCalendar =
            toml::from_str(&raw).map_err(|e| CalendarError::Toml(e.to_string()))?;

        let events = parsed
            .events
            .into_iter()
            .map(ScheduledEvent::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            name: parsed.name,
            events,
        })
    }

    /// Merge another calendar into this one (for combining US + AU calendars).
    pub fn merge(&mut self, other: BusinessCalendar) {
        self.events.extend(other.events);
    }

    // -----------------------------------------------------------------------
    // Core date arithmetic
    // -----------------------------------------------------------------------

    /// Compute the next due date for a recurrence rule strictly after `after`.
    ///
    /// Returns `None` only for `CronExpr` (stub) or if no valid date can be
    /// constructed (e.g. day_of_month 31 in February).
    pub fn next_due(rule: &RecurrenceRule, after: NaiveDate) -> Option<NaiveDate> {
        match rule {
            RecurrenceRule::Monthly { day_of_month } => next_monthly(after, *day_of_month),
            RecurrenceRule::Annual { month, day } => next_annual(after, *month, *day),
            RecurrenceRule::QuarterlyEstimated { day } => next_quarterly_estimated(after, *day),
            RecurrenceRule::EveryNDays { n } => after.checked_add_days(Days::new(*n as u64)),
            RecurrenceRule::CronExpr(_) => None,
        }
    }

    /// Return all events due on exactly `date`.
    pub fn events_due_on(&self, date: NaiveDate) -> Vec<&ScheduledEvent> {
        self.events
            .iter()
            .filter(|ev| {
                if !ev.enabled {
                    return false;
                }
                let after = date.pred_opt().unwrap_or(date);
                Self::next_due(&ev.recurrence, after)
                    .map(|d| d == date)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Return all `(date, event)` pairs due within the next `horizon_days` from `from`.
    /// Sorted by date ascending.
    pub fn upcoming(
        &self,
        from: NaiveDate,
        horizon_days: u32,
    ) -> Vec<(NaiveDate, &ScheduledEvent)> {
        let end = from
            .checked_add_days(Days::new(horizon_days as u64))
            .unwrap_or(from);

        let mut results: Vec<(NaiveDate, &ScheduledEvent)> = self
            .events
            .iter()
            .filter(|ev| ev.enabled)
            .filter_map(|ev| {
                // Compute the first occurrence within the window.
                match Self::next_due(&ev.recurrence, from) {
                    Some(d) if d <= end => Some((d, ev)),
                    _ => None,
                }
            })
            .collect();

        results.sort_by_key(|(d, _)| *d);
        results
    }

    /// Return events tagged with a specific tag.
    pub fn events_by_tag(&self, tag: &str) -> Vec<&ScheduledEvent> {
        self.events
            .iter()
            .filter(|ev| ev.tags.iter().any(|t| t == tag))
            .collect()
    }

    // -----------------------------------------------------------------------
    // Default calendars
    // -----------------------------------------------------------------------

    /// Pre-populated US tax deadline events.
    pub fn us_tax_defaults() -> Self {
        let mut cal = Self::new("US Tax Defaults");
        cal.events = vec![
            ScheduledEvent {
                id: "us-quarterly-estimated".to_string(),
                description: "US quarterly estimated tax payment (Form 1040-ES)".to_string(),
                recurrence: RecurrenceRule::QuarterlyEstimated { day: 15 },
                operation: OperationKind::CheckTaxDeadline {
                    deadline_id: "us-quarterly-estimated".to_string(),
                },
                jurisdiction: Some(Jurisdiction::US),
                enabled: true,
                last_run: None,
                tags: vec!["us".to_string(), "estimated-tax".to_string()],
            },
            ScheduledEvent {
                id: "us-annual-return".to_string(),
                description: "US annual income tax return (Form 1040) due April 15".to_string(),
                recurrence: RecurrenceRule::Annual { month: 4, day: 15 },
                operation: OperationKind::CheckTaxDeadline {
                    deadline_id: "us-annual-return".to_string(),
                },
                jurisdiction: Some(Jurisdiction::US),
                enabled: true,
                last_run: None,
                tags: vec!["us".to_string(), "annual-return".to_string()],
            },
            ScheduledEvent {
                id: "us-extension-deadline".to_string(),
                description: "US extension deadline October 15".to_string(),
                recurrence: RecurrenceRule::Annual { month: 10, day: 15 },
                operation: OperationKind::CheckTaxDeadline {
                    deadline_id: "us-extension-deadline".to_string(),
                },
                jurisdiction: Some(Jurisdiction::US),
                enabled: true,
                last_run: None,
                tags: vec!["us".to_string(), "extension".to_string()],
            },
            ScheduledEvent {
                id: "us-fbar".to_string(),
                description: "FBAR (FinCEN 114) due April 15 (auto-extension Oct 15)".to_string(),
                recurrence: RecurrenceRule::Annual { month: 4, day: 15 },
                operation: OperationKind::CheckTaxDeadline {
                    deadline_id: "us-fbar".to_string(),
                },
                jurisdiction: Some(Jurisdiction::US),
                enabled: true,
                last_run: None,
                tags: vec!["us".to_string(), "fbar".to_string(), "expat".to_string()],
            },
            ScheduledEvent {
                id: "us-fatca-8938".to_string(),
                description: "FATCA Form 8938 (foreign financial assets) due with return"
                    .to_string(),
                recurrence: RecurrenceRule::Annual { month: 4, day: 15 },
                operation: OperationKind::CheckTaxDeadline {
                    deadline_id: "us-fatca-8938".to_string(),
                },
                jurisdiction: Some(Jurisdiction::US),
                enabled: true,
                last_run: None,
                tags: vec!["us".to_string(), "fatca".to_string(), "expat".to_string()],
            },
            ScheduledEvent {
                id: "us-monthly-ingest".to_string(),
                description: "Monthly statement ingest trigger (1st of month)".to_string(),
                recurrence: RecurrenceRule::Monthly { day_of_month: 1 },
                operation: OperationKind::IngestStatement {
                    source_glob: "statements/*.pdf".to_string(),
                },
                jurisdiction: Some(Jurisdiction::US),
                enabled: true,
                last_run: None,
                tags: vec!["us".to_string(), "ingest".to_string()],
            },
        ];
        cal
    }

    /// Pre-populated AU tax deadline events.
    pub fn au_tax_defaults() -> Self {
        let mut cal = Self::new("AU Tax Defaults");
        cal.events = vec![
            ScheduledEvent {
                id: "au-bas-quarterly".to_string(),
                description: "AU BAS quarterly lodgement (28 days after quarter end)".to_string(),
                recurrence: RecurrenceRule::QuarterlyEstimated { day: 28 },
                operation: OperationKind::CheckTaxDeadline {
                    deadline_id: "au-bas-quarterly".to_string(),
                },
                jurisdiction: Some(Jurisdiction::AU),
                enabled: true,
                last_run: None,
                tags: vec!["au".to_string(), "bas".to_string(), "gst".to_string()],
            },
            ScheduledEvent {
                id: "au-annual-return-individual".to_string(),
                description: "AU individual income tax return due October 31".to_string(),
                recurrence: RecurrenceRule::Annual { month: 10, day: 31 },
                operation: OperationKind::CheckTaxDeadline {
                    deadline_id: "au-annual-return-individual".to_string(),
                },
                jurisdiction: Some(Jurisdiction::AU),
                enabled: true,
                last_run: None,
                tags: vec!["au".to_string(), "annual-return".to_string()],
            },
            ScheduledEvent {
                id: "au-annual-return-tax-agent".to_string(),
                description: "AU income tax return with registered tax agent due May 15"
                    .to_string(),
                recurrence: RecurrenceRule::Annual { month: 5, day: 15 },
                operation: OperationKind::CheckTaxDeadline {
                    deadline_id: "au-annual-return-tax-agent".to_string(),
                },
                jurisdiction: Some(Jurisdiction::AU),
                enabled: true,
                last_run: None,
                tags: vec![
                    "au".to_string(),
                    "annual-return".to_string(),
                    "tax-agent".to_string(),
                ],
            },
            ScheduledEvent {
                id: "au-monthly-ingest".to_string(),
                description: "Monthly statement ingest trigger (1st of month)".to_string(),
                recurrence: RecurrenceRule::Monthly { day_of_month: 1 },
                operation: OperationKind::IngestStatement {
                    source_glob: "statements/au/*.pdf".to_string(),
                },
                jurisdiction: Some(Jurisdiction::AU),
                enabled: true,
                last_run: None,
                tags: vec!["au".to_string(), "ingest".to_string()],
            },
        ];
        cal
    }
}

// ---------------------------------------------------------------------------
// Date arithmetic helpers
// ---------------------------------------------------------------------------

/// Next monthly occurrence strictly after `after`.
fn next_monthly(after: NaiveDate, day_of_month: u8) -> Option<NaiveDate> {
    let day = day_of_month as u32;

    // Try this month first
    if let Some(candidate) = NaiveDate::from_ymd_opt(after.year(), after.month(), day) {
        if candidate > after {
            return Some(candidate);
        }
    }

    // Advance month by month until we find a month that has `day`
    let mut base = after;
    for _ in 0..24 {
        base = base.checked_add_months(Months::new(1))?;
        if let Some(candidate) = NaiveDate::from_ymd_opt(base.year(), base.month(), day) {
            return Some(candidate);
        }
        // Month doesn't have this day (e.g. Feb 30) — continue to next month
    }
    None
}

/// Next annual occurrence strictly after `after`.
fn next_annual(after: NaiveDate, month: u8, day: u8) -> Option<NaiveDate> {
    let m = month as u32;
    let d = day as u32;

    // Try this year
    if let Some(candidate) = NaiveDate::from_ymd_opt(after.year(), m, d) {
        if candidate > after {
            return Some(candidate);
        }
    }
    // Try next year
    NaiveDate::from_ymd_opt(after.year() + 1, m, d)
}

/// US quarterly estimated tax months: April, June, September, January.
/// January always belongs to the *next* calendar year relative to the preceding
/// September quarter (i.e. the Q4 payment). AU BAS uses the same pattern.
///
/// We represent candidates as (year, month) pairs and pick the earliest
/// that is strictly after `after`.
fn next_quarterly_estimated(after: NaiveDate, day: u8) -> Option<NaiveDate> {
    // Build candidates spanning two calendar years so we always find one.
    // The four quarter-payment months in calendar order:
    const QUARTER_MONTHS: [u32; 4] = [4, 6, 9, 1];
    let d = day as u32;
    let base_year = after.year();

    // Generate (year, month) pairs for base_year and base_year+1
    let mut candidates: Vec<NaiveDate> = Vec::with_capacity(8);
    for &yr_offset in &[0i32, 1] {
        for &month in &QUARTER_MONTHS {
            // January in the quarter cycle belongs to the year *after* September,
            // so for the base_year pass we use base_year+1 for January.
            let year = if month == 1 {
                base_year + 1
            } else {
                base_year + yr_offset
            };
            if let Some(d) = NaiveDate::from_ymd_opt(year, month, d) {
                candidates.push(d);
            }
        }
    }

    candidates.sort();
    candidates.into_iter().find(|&c| c > after)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    // -----------------------------------------------------------------------
    // Monthly
    // -----------------------------------------------------------------------

    #[test]
    fn monthly_advances_to_next_month_when_past() {
        let rule = RecurrenceRule::Monthly { day_of_month: 15 };
        let after = ymd(2024, 1, 20); // past the 15th
        assert_eq!(
            BusinessCalendar::next_due(&rule, after),
            Some(ymd(2024, 2, 15))
        );
    }

    #[test]
    fn monthly_returns_same_month_when_before() {
        let rule = RecurrenceRule::Monthly { day_of_month: 15 };
        let after = ymd(2024, 1, 10); // before the 15th
        assert_eq!(
            BusinessCalendar::next_due(&rule, after),
            Some(ymd(2024, 1, 15))
        );
    }

    #[test]
    fn monthly_day_1_from_end_of_month() {
        let rule = RecurrenceRule::Monthly { day_of_month: 1 };
        let after = ymd(2024, 1, 31);
        assert_eq!(
            BusinessCalendar::next_due(&rule, after),
            Some(ymd(2024, 2, 1))
        );
    }

    // -----------------------------------------------------------------------
    // Annual
    // -----------------------------------------------------------------------

    #[test]
    fn annual_before_date_this_year() {
        let rule = RecurrenceRule::Annual { month: 4, day: 15 };
        let after = ymd(2024, 3, 1); // before April 15
        assert_eq!(
            BusinessCalendar::next_due(&rule, after),
            Some(ymd(2024, 4, 15))
        );
    }

    #[test]
    fn annual_after_date_goes_to_next_year() {
        let rule = RecurrenceRule::Annual { month: 4, day: 15 };
        let after = ymd(2024, 5, 1); // after April 15 this year
        assert_eq!(
            BusinessCalendar::next_due(&rule, after),
            Some(ymd(2025, 4, 15))
        );
    }

    #[test]
    fn annual_exact_date_advances_to_next_year() {
        let rule = RecurrenceRule::Annual { month: 4, day: 15 };
        let after = ymd(2024, 4, 15); // exactly on the date → strictly after
        assert_eq!(
            BusinessCalendar::next_due(&rule, after),
            Some(ymd(2025, 4, 15))
        );
    }

    // -----------------------------------------------------------------------
    // Quarterly estimated
    // -----------------------------------------------------------------------

    #[test]
    fn quarterly_from_feb_fires_in_april() {
        let rule = RecurrenceRule::QuarterlyEstimated { day: 15 };
        let after = ymd(2024, 2, 1);
        let next = BusinessCalendar::next_due(&rule, after).unwrap();
        assert_eq!(next, ymd(2024, 4, 15));
    }

    #[test]
    fn quarterly_from_may_fires_in_june() {
        let rule = RecurrenceRule::QuarterlyEstimated { day: 15 };
        let after = ymd(2024, 5, 1);
        let next = BusinessCalendar::next_due(&rule, after).unwrap();
        assert_eq!(next, ymd(2024, 6, 15));
    }

    #[test]
    fn quarterly_from_july_fires_in_september() {
        let rule = RecurrenceRule::QuarterlyEstimated { day: 15 };
        let after = ymd(2024, 7, 1);
        let next = BusinessCalendar::next_due(&rule, after).unwrap();
        assert_eq!(next, ymd(2024, 9, 15));
    }

    #[test]
    fn quarterly_from_october_fires_in_january_next_year() {
        let rule = RecurrenceRule::QuarterlyEstimated { day: 15 };
        let after = ymd(2024, 10, 1);
        let next = BusinessCalendar::next_due(&rule, after).unwrap();
        assert_eq!(next, ymd(2025, 1, 15));
    }

    // -----------------------------------------------------------------------
    // EveryNDays
    // -----------------------------------------------------------------------

    #[test]
    fn every_n_days_adds_n() {
        let rule = RecurrenceRule::EveryNDays { n: 7 };
        let after = ymd(2024, 1, 1);
        assert_eq!(
            BusinessCalendar::next_due(&rule, after),
            Some(ymd(2024, 1, 8))
        );
    }

    // -----------------------------------------------------------------------
    // CronExpr
    // -----------------------------------------------------------------------

    #[test]
    fn cron_expr_returns_none() {
        let rule = RecurrenceRule::CronExpr("0 9 * * MON".to_string());
        assert_eq!(BusinessCalendar::next_due(&rule, ymd(2024, 1, 1)), None);
    }

    // -----------------------------------------------------------------------
    // events_due_on
    // -----------------------------------------------------------------------

    #[test]
    fn events_due_on_known_date() {
        let cal = BusinessCalendar::us_tax_defaults();
        // April 15 should fire the annual return
        let due = cal.events_due_on(ymd(2024, 4, 15));
        let ids: Vec<&str> = due.iter().map(|e| e.id.as_str()).collect();
        assert!(
            ids.contains(&"us-annual-return"),
            "expected us-annual-return in {ids:?}"
        );
    }

    #[test]
    fn events_due_on_first_of_month() {
        let cal = BusinessCalendar::us_tax_defaults();
        let due = cal.events_due_on(ymd(2024, 3, 1));
        let ids: Vec<&str> = due.iter().map(|e| e.id.as_str()).collect();
        assert!(
            ids.contains(&"us-monthly-ingest"),
            "expected us-monthly-ingest in {ids:?}"
        );
    }

    // -----------------------------------------------------------------------
    // upcoming
    // -----------------------------------------------------------------------

    #[test]
    fn upcoming_sorted_by_date() {
        let cal = BusinessCalendar::us_tax_defaults();
        let from = ymd(2024, 3, 1);
        let items = cal.upcoming(from, 60);
        // Verify sort order
        for w in items.windows(2) {
            assert!(
                w[0].0 <= w[1].0,
                "dates out of order: {:?} > {:?}",
                w[0].0,
                w[1].0
            );
        }
    }

    #[test]
    fn upcoming_respects_horizon() {
        let cal = BusinessCalendar::us_tax_defaults();
        let from = ymd(2024, 3, 1);
        let end = from.checked_add_days(Days::new(60)).unwrap();
        let items = cal.upcoming(from, 60);
        for (d, _) in &items {
            assert!(*d <= end, "date {d} outside horizon");
        }
    }

    // -----------------------------------------------------------------------
    // us_tax_defaults / au_tax_defaults
    // -----------------------------------------------------------------------

    #[test]
    fn us_tax_defaults_has_at_least_five_events() {
        let cal = BusinessCalendar::us_tax_defaults();
        assert!(
            cal.events.len() >= 5,
            "expected >= 5 events, got {}",
            cal.events.len()
        );
    }

    #[test]
    fn au_tax_defaults_has_events() {
        let cal = BusinessCalendar::au_tax_defaults();
        assert!(!cal.events.is_empty());
        let au_events: Vec<_> = cal
            .events
            .iter()
            .filter(|e| e.jurisdiction == Some(Jurisdiction::AU))
            .collect();
        assert!(!au_events.is_empty(), "expected AU jurisdiction events");
    }

    // -----------------------------------------------------------------------
    // merge
    // -----------------------------------------------------------------------

    #[test]
    fn merge_combines_two_calendars() {
        let mut us = BusinessCalendar::us_tax_defaults();
        let au = BusinessCalendar::au_tax_defaults();
        let us_count = us.events.len();
        let au_count = au.events.len();
        us.merge(au);
        assert_eq!(us.events.len(), us_count + au_count);
    }

    // -----------------------------------------------------------------------
    // events_by_tag
    // -----------------------------------------------------------------------

    #[test]
    fn events_by_tag_returns_matching() {
        let cal = BusinessCalendar::us_tax_defaults();
        let fbar_events = cal.events_by_tag("fbar");
        assert!(
            !fbar_events.is_empty(),
            "expected at least one fbar-tagged event"
        );
        for ev in &fbar_events {
            assert!(ev.tags.contains(&"fbar".to_string()));
        }
    }

    #[test]
    fn events_by_tag_no_match_returns_empty() {
        let cal = BusinessCalendar::us_tax_defaults();
        let results = cal.events_by_tag("nonexistent-tag-xyz");
        assert!(results.is_empty());
    }
}
