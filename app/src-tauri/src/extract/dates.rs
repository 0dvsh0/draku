//! Date extraction from real text (PDF text, plain files, filenames, OCR output).
//! Day-first first (the product's primary audience writes 11/10/2026 this way),
//! with keyword context deciding what kind of date it is.

use chrono::NaiveDate;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Clone, Debug)]
pub struct Found {
    pub date: NaiveDate,
    /// Short human label, e.g. "Warranty ends", "Valid till", "Mentioned".
    pub label: String,
    /// "deadline" | "mentioned"
    pub kind: String,
}

static ISO: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(?<y>\d{4})-(?<m>\d{1,2})-(?<d>\d{1,2})\b").unwrap());

/// Day-first first; falls back to month-first when the pair is invalid in that order.
static NUMERIC: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(?<a>\d{1,2})[/.\-](?<b>\d{1,2})[/.\-](?<y>\d{2,4})\b").unwrap());

/// "12 Jan 2027" / "12-Jan-27"
static MONTH_A: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?ix)\b(?<d>\d{1,2})(?:st|nd|rd|th)?[\s.\-]+(?<mon>jan|feb|mar|apr|may|jun|jul|aug|sep|sept|oct|nov|dec)[a-z]*(?:[\s.\-,]+(?<y>\d{4}))?\b").unwrap()
});

/// "Jan 12, 2027" / "Jan 12"
static MONTH_B: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?ix)\b(?<mon>jan|feb|mar|apr|may|jun|jul|aug|sep|sept|oct|nov|dec)[a-z]*\.?[\s.\-]+(?<d>\d{1,2})(?:st|nd|rd|th)?(?:[,.]?[\s.\-]+(?<y>\d{4}))?\b").unwrap()
});

/// Verbose regex (`(?x)`) skips whitespace everywhere — including inside
/// classes — so literal spaces must be written as `\s+`.
static KW_DEADLINE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?ix)
        warranty[^\n]{0,24}(?:end|expire|until|till)
        | expires?\s+(?:on|by)
        | expiring\s+(?:date|on)
        | valid\s+(?:until|till)
        | last\s+date
        | due\s+(?:by|date|on)
        | close[sd](?:\s+on)?
        | closing\s+date
        | refill\s+by
        | renew\s+by
        | return\s+by
        | submit\s+by
        | pay\s+by
        | ends\s+in
        | appointment\s+on
        | delivery\s+by
        | journey\s+date
        | travel\s+date
        | departure\s+date
        | flight\s+date
        | event\s+date
        "#,
    )
    .unwrap()
});

fn month_num(w: &str) -> Option<u32> {
    let short: String = w.chars().take(3).collect::<String>().to_lowercase();
    Some(match short.as_str() {
        "jan" => 1,
        "feb" => 2,
        "mar" => 3,
        "apr" => 4,
        "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "aug" => 8,
        "sep" => 9,
        "oct" => 10,
        "nov" => 11,
        "dec" => 12,
        _ => return None,
    })
}

fn year_fix(y: i32) -> i32 {
    if (0..100).contains(&y) {
        2000 + y
    } else {
        y
    }
}

/// Extract every date-like fact, labeled by the language around it.
pub fn parse_all(text: &str) -> Vec<Found> {
    let mut seen: Vec<(NaiveDate, String)> = Vec::new();
    let mut out: Vec<Found> = Vec::new();

    let mut push = |d: NaiveDate, start: usize, end: usize| {
        // A date's meaning is local. Reading keywords from the whole file made
        // "Purchase date 2026-10-04" a deadline merely because "Warranty ends"
        // appeared three lines later, which then marked a live warranty dead.
        let context = context_at(text, start, end);
        let kind = kind_for(context);
        if !seen.iter().any(|(date, k)| *date == d && k == &kind) {
            seen.push((d, kind.clone()));
            out.push(Found {
                date: d,
                label: label_for(context),
                kind,
            });
        }
    };

    for m in ISO.captures_iter(text) {
        let (start, end) = match m.get(0) {
            Some(x) => (x.start(), x.end()),
            None => continue,
        };
        if let Some(d) = NaiveDate::from_ymd_opt(num_i32(&m["y"]), num(&m["m"]), num(&m["d"])) {
            push(d, start, end);
        }
    }

    for m in NUMERIC.captures_iter(text) {
        let (start, end) = match m.get(0) {
            Some(x) => (x.start(), x.end()),
            None => continue,
        };
        let (a, b, y) = (num(&m["a"]), num(&m["b"]), year_fix(num_i32(&m["y"])));
        let d = NaiveDate::from_ymd_opt(y, b, a).or_else(|| NaiveDate::from_ymd_opt(y, a, b));
        if let Some(d) = d {
            push(d, start, end);
        }
    }

    for m in MONTH_A.captures_iter(text) {
        let (start, end) = match m.get(0) {
            Some(x) => (x.start(), x.end()),
            None => continue,
        };
        let y = m
            .name("y")
            .and_then(|g| g.as_str().parse::<i32>().ok())
            .map(year_fix)
            .unwrap_or_else(current_year);
        if let Some(d) = NaiveDate::from_ymd_opt(y, month_num(&m["mon"]).unwrap_or(0), num(&m["d"]))
        {
            push(d, start, end);
        }
    }

    for m in MONTH_B.captures_iter(text) {
        let (start, end) = match m.get(0) {
            Some(x) => (x.start(), x.end()),
            None => continue,
        };
        let y = m
            .name("y")
            .and_then(|g| g.as_str().parse::<i32>().ok())
            .map(year_fix)
            .unwrap_or_else(current_year);
        if let Some(d) = NaiveDate::from_ymd_opt(y, month_num(&m["mon"]).unwrap_or(0), num(&m["d"]))
        {
            push(d, start, end);
        }
    }

    out
}

/// The date's own line plus the line above it. OCR often splits a label from
/// its date ("Refill by" / "2026-11-10"), while dates on a receipt stay next
/// to the word that gives them meaning.
fn context_at(text: &str, start: usize, end: usize) -> &str {
    let bytes = text.as_bytes();
    let mut line_start = start.min(bytes.len());
    while line_start > 0 && bytes[line_start - 1] != b'\n' {
        line_start -= 1;
    }
    let mut ctx_start = line_start;
    if ctx_start > 0 {
        let mut prev = ctx_start - 1;
        while prev > 0 && bytes[prev - 1] != b'\n' {
            prev -= 1;
        }
        ctx_start = prev;
    }
    let mut line_end = end.min(bytes.len());
    while line_end < bytes.len() && bytes[line_end] != b'\n' {
        line_end += 1;
    }
    &text[ctx_start..line_end]
}

fn num(s: &str) -> u32 {
    s.parse().unwrap_or(0)
}

fn num_i32(s: &str) -> i32 {
    s.parse::<i32>().unwrap_or(0)
}

/// Year to assume when the text says "Jan 12" with no year.
fn current_year() -> i32 {
    use chrono::Datelike;
    chrono::Local::now().year()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dates(s: &str) -> Vec<NaiveDate> {
        parse_all(s).into_iter().map(|f| f.date).collect()
    }

    #[test]
    fn reads_iso_dates() {
        assert!(dates(&"refill by 2026-11-10 signed")
            .contains(&NaiveDate::from_ymd_opt(2026, 11, 10).unwrap()));
    }

    #[test]
    fn prefers_day_first_for_numeric_dates() {
        let found = dates("11/10/2026 and 31/12/2027");
        assert!(found.contains(&NaiveDate::from_ymd_opt(2026, 10, 11).unwrap()));
        // ambiguous 31 can't be a month: falls back to month-first
        assert!(found.contains(&NaiveDate::from_ymd_opt(2027, 12, 31).unwrap()));
    }

    #[test]
    fn reads_written_months_both_ways() {
        assert!(dates("valid till 12 Jan 2027")
            .contains(&NaiveDate::from_ymd_opt(2027, 1, 12).unwrap()));
        assert!(dates("expires on Jan 12, 2027")
            .contains(&NaiveDate::from_ymd_opt(2027, 1, 12).unwrap()));
    }

    #[test]
    fn deadline_language_is_labeled_and_kind_marked() {
        let f = parse_all("warranty ends on 2026-12-01");
        assert_eq!(f[0].kind, "deadline");
        assert_ne!(f[0].label, "Mentioned");
    }

    #[test]
    fn duplicate_dates_are_deduped() {
        assert_eq!(dates("2026-12-01 … 2026-12-01").len(), 1);
    }

    #[test]
    fn a_purchase_date_does_not_inherit_warranty_language() {
        let found = parse_all("Purchase date 2026-10-04\nWarranty ends 2026-11-04");
        let purchase = found
            .iter()
            .find(|f| f.date == NaiveDate::from_ymd_opt(2026, 10, 4).unwrap())
            .expect("purchase date");
        let warranty = found
            .iter()
            .find(|f| f.date == NaiveDate::from_ymd_opt(2026, 11, 4).unwrap())
            .expect("warranty date");
        assert_eq!(purchase.kind, "mentioned");
        assert_eq!(warranty.kind, "deadline");
        assert!(warranty.label.to_lowercase().contains("warranty"));
    }

    #[test]
    fn a_label_on_the_line_above_its_date_still_counts() {
        let found = parse_all("Refill by\n2026-11-10");
        assert_eq!(found[0].kind, "deadline");
        assert!(found[0].label.to_lowercase().contains("refill"));
    }

    #[test]
    fn a_journey_date_is_an_evolving_deadline() {
        let found = parse_all("Journey date 2026-10-20");
        assert_eq!(found[0].kind, "deadline");
    }
}

fn label_for(text: &str) -> String {
    KW_DEADLINE
        .find(text)
        .map(|m| humanize(m.as_str()))
        .unwrap_or_else(|| "Mentioned".into())
}

fn kind_for(text: &str) -> String {
    if KW_DEADLINE.is_match(text) {
        "deadline".into()
    } else {
        "mentioned".into()
    }
}

/// "valid until" → "Valid until"
fn humanize(kw: &str) -> String {
    let phrase: String = kw
        .split_whitespace()
        .take(3)
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let phrase = if phrase.is_empty() {
        "Mentioned".to_string()
    } else {
        phrase
    };
    let mut chars = phrase.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => "Mentioned".into(),
    }
}
