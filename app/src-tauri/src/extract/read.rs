//! The READER: one pass over a file → one `Reading`.
//! Find / Remember / Protect / Let go are all views of that reading.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{Local, NaiveDate, NaiveDateTime};
use once_cell::sync::Lazy;
use regex::Regex;

use super::classify::{self, Kind};
use super::ocr::{self, Chip, ChipFinder};
use super::{dates, imginfo, secrets};
use crate::model::{DateRef, Reading, SecretRef};
use crate::score::{self, Signals};

const TEXT_LIMIT: usize = 240_000; // bytes of text pulled per file
const KEY_TEXT_LIMIT: usize = 1_500; // chars embedded per file

pub fn read_file(root: &Path, rel: &str) -> Reading {
    let full: PathBuf = root.join(rel);
    let meta = fs::metadata(&full).ok();
    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    let mtime = meta
        .as_ref()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let path = Path::new(rel);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    let stem = path
        .file_stem()
        .and_then(|e| e.to_str())
        .unwrap_or(rel)
        .to_string();
    let kind = classify::kind_of(&ext);

    // Real text from real sources: PDF text, plain text, then on-device OCR.
    let body_text = match kind {
        Kind::Pdf => pdf_text(&full),
        Kind::Text => plain_text(&full),
        _ => String::new(),
    };

    // Image facts: EXIF/GPS + perceptual hash.
    let facts = if kind == Kind::Image {
        imginfo::probe(&full)
    } else {
        imginfo::ImageFacts {
            gps: None,
            taken: None,
            phash: None,
        }
    };

    let chips = if kind == Kind::Image {
        ocr_chips(&full, size)
    } else {
        vec![]
    };
    let ocr_text = chips
        .iter()
        .map(|c| c.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let all_text = if body_text.is_empty() {
        ocr_text
    } else if ocr_text.is_empty() {
        body_text
    } else {
        format!("{body_text}\n{ocr_text}")
    };

    let category = classify::category_of_body(&stem, &all_text, kind);
    let subject = subject_of(&stem, &all_text);
    let name_date = facts.taken.or_else(|| compact_date(&stem));
    let dates = gather_dates(&stem, &all_text, facts.taken);

    let mut secrets_found: Vec<SecretRef> = if chips.is_empty() {
        secrets::parse_text(&all_text, None)
    } else {
        chip_secrets(&chips, &all_text)
    };
    if let Some(gps) = facts.gps {
        // Coordinates a camera buried in metadata count as an exposed secret
        // with no words anywhere in the file.
        secrets_found.push(SecretRef {
            kind: "location".into(),
            preview: format!("{:.3}, {:.3}", gps.0, gps.1),
            page: None,
            bbox: None,
        });
    }

    let name = propose_name(&stem, name_date, category, &subject);
    let key_text = build_key_text(&name, category, &subject, &all_text);

    let today = Local::now().date_naive();
    let lower = all_text.to_lowercase();
    let signals = Signals {
        age_days: days_since(mtime),
        is_screenshot: is_machine_named(&stem),
        is_otp: category == "one-time code"
            || lower.contains("otp")
            || lower.contains("one-time code")
            || lower.contains("verification code"),
        is_ticket: category == "ticket"
            || lower.contains("boarding pass")
            || lower.contains(" e-ticket")
            || lower.contains("ticket"),
        date_passed: dates.iter().any(|d| {
            d.kind == "deadline" && parse_date(&d.date).map(|p| p < today).unwrap_or(false)
        }),
        is_dup: false, // filled by the duplicate pass in scan.rs
        size,
    };
    let verdict = score::judge(&signals);

    Reading {
        root: root.display().to_string(),
        rel: rel.to_string(),
        name,
        category: category.to_string(),
        key_text,
        dates,
        secrets: secrets_found,
        gps: facts.gps,
        phash: facts.phash,
        dup_of: None,
        dead: verdict.dead,
        dead_reason: verdict.reason,
        size,
        mtime,
    }
}

/// Re-cost a reading once the duplicate pass has labeled it.
pub fn with_dup(r: &mut Reading) {
    if r.dup_of.is_some() {
        let today = Local::now().date_naive();
        let lower = r.key_text.to_lowercase();
        let signals = Signals {
            age_days: days_since(r.mtime),
            is_screenshot: is_machine_named(&r.rel),
            is_otp: r.category == "one-time code"
                || lower.contains("otp")
                || lower.contains("verification code"),
            is_ticket: r.category == "ticket"
                || lower.contains("boarding pass")
                || lower.contains("ticket"),
            date_passed: r.dates.iter().any(|d| {
                d.kind == "deadline" && parse_date(&d.date).map(|p| p < today).unwrap_or(false)
            }),
            is_dup: true,
            size: r.size,
        };
        let v = score::judge(&signals);
        r.dead = v.dead;
        r.dead_reason = v.reason.clone();
    }
}

fn is_machine_named(stem: &str) -> bool {
    classify::looks_anonymous(stem)
}

/// OCR is deliberately skipped for enormous files (a 200 MB scan would stall
/// the whole scan; those still get EXIF, hashing and a real reading).
fn ocr_chips(full: &Path, size: u64) -> Vec<Chip> {
    if size > 30 * 1024 * 1024 {
        return vec![];
    }
    let decoded = image::ImageReader::open(full)
        .ok()
        .and_then(|r| r.with_guessed_format().ok())
        .and_then(|r| r.decode().ok());
    match decoded {
        Some(img) => {
            let rgba = img.to_rgba8();
            ocr::CHIP_FINDER.find_chips(&rgba)
        }
        None => vec![],
    }
}

fn chip_secrets(chips: &[Chip], full_text: &str) -> Vec<SecretRef> {
    let mut out: Vec<SecretRef> = vec![];
    for chip in chips {
        for s in secrets::parse_text_at(&chip.text, chip.page, Some(chip.bbox)) {
            push_secret(&mut out, s);
        }
    }
    // Cross-line values (a label on one block, the number on the next) still
    // get caught; they simply have no box to redact.
    for s in secrets::parse_text(full_text, None) {
        push_secret(&mut out, s);
    }
    out
}

fn push_secret(out: &mut Vec<SecretRef>, s: SecretRef) {
    if let Some(existing) = out
        .iter_mut()
        .find(|x| x.kind == s.kind && x.preview == s.preview)
    {
        if existing.bbox.is_none() {
            existing.bbox = s.bbox;
        }
        return;
    }
    out.push(s);
}

/// Pull real text out of a PDF.
fn pdf_text(full: &Path) -> String {
    match pdf_extract::extract_text(full) {
        Ok(t) => {
            let t = sanitize(&t);
            t.chars().take(TEXT_LIMIT).collect()
        }
        Err(_) => String::new(),
    }
}

fn sanitize(t: &str) -> String {
    t.replace('\u{FFFD}', " ")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn plain_text(full: &Path) -> String {
    if fs::metadata(full).map(|m| m.len()).unwrap_or(0) > TEXT_LIMIT as u64 {
        return String::new();
    }
    match fs::read(full) {
        Ok(b) => String::from_utf8_lossy(&b).to_string(),
        Err(_) => String::new(),
    }
}

static NOISE_WORD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(screenshot|img|dsc|dscn|dji|gopr|pxl|wa\d*|scan\d*|photo|video|untitled|download|document\d*|final|copy|edit|img-\d+-\w+)$",
    )
    .unwrap()
});

/// Subject words out of a filename ("dr-patel-blood-test-2026" →
/// "Dr patel blood test") or the most meaningful OCR/PDF line.
fn subject_of(stem: &str, body: &str) -> String {
    let from_name = clean_stem(stem);
    let stem_is_machine = classif_anon(stem);
    if !stem_is_machine && !from_name.is_empty() {
        return from_name.chars().take(80).collect();
    }
    let from_body = subject_from_body(body);
    if !from_body.is_empty() {
        return from_body;
    }
    if !from_name.is_empty() {
        return from_name.chars().take(80).collect();
    }
    String::new()
}

fn classif_anon(stem: &str) -> bool {
    classify::looks_anonymous(stem)
}

fn clean_stem(stem: &str) -> String {
    stem.replace(['_', '-', '.'], " ")
        .split_whitespace()
        .filter(|w| w.chars().any(|c| c.is_alphabetic()))
        .filter(|w| !NOISE_WORD.is_match(w))
        .filter(|w| !w.chars().all(|c| c.is_ascii_digit()))
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Choose a human line, preferring a line that names the document's purpose.
fn subject_from_body(body: &str) -> String {
    static SKIP: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?i)^(name|patient|address|phone|email|e-mail|dob|date of birth|account|card number|aadhaar|pan|invoice no|order no|receipt no)\b",
        )
        .unwrap()
    });
    static PREFER: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?i)\b(prescription|blood test|lab report|medical|warranty|receipt|invoice|ticket|boarding pass|insurance|statement|passport|aadhaar|licen[cs]e|pan card|certificate|report|contract|agreement|appointment|order|payment)\b",
        )
        .unwrap()
    });

    let mut fallback = String::new();
    for raw in body.lines().take(80) {
        let line = raw
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim_matches(|c: char| c.is_whitespace() || "|·•—-".contains(c))
            .to_string();
        if line.len() < 8 || line.len() > 110 {
            continue;
        }
        let letters = line.chars().filter(|c| c.is_alphabetic()).count();
        let digits = line.chars().filter(|c| c.is_ascii_digit()).count();
        if letters < 3 || digits > letters {
            continue;
        }
        if line.contains("http") || line.contains("www.") || SKIP.is_match(&line) {
            continue;
        }
        let clean = strip_label(&line);
        if PREFER.is_match(&clean) {
            return clean.chars().take(90).collect();
        }
        if fallback.is_empty() {
            fallback = clean;
        }
    }
    fallback.chars().take(90).collect()
}

fn strip_label(line: &str) -> String {
    static LABEL: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?i)^(subject|description|item|document|title|re)\s*[:\-]\s*").unwrap()
    });
    LABEL.replace(line, "").trim().to_string()
}

/// The visible name promise: `2026-10-11 Dr Patel blood test prescription.jpg`.
/// The original stem stays traceable whenever it carried anything.
fn propose_name(stem: &str, date: Option<NaiveDateTime>, category: &str, subject: &str) -> String {
    let date_prefix = date.map(|d| d.format("%Y-%m-%d").to_string());
    let mut subject = subject.trim().trim_matches(',').to_string();
    if subject.to_lowercase() == category.to_lowercase() {
        subject.clear();
    }
    let meaningful = !subject.is_empty() && !classif_anon(&subject);

    let mut parts: Vec<String> = vec![];
    if let Some(d) = date_prefix {
        parts.push(d);
    }
    if meaningful {
        parts.push(subject.chars().take(90).collect());
    } else {
        parts.push(category.to_string());
        let original = stem.trim();
        if !original.is_empty() {
            parts.push(format!("({original})"));
        }
    }
    parts.join(" ").trim().to_string()
}

fn build_key_text(name: &str, category: &str, subject: &str, body: &str) -> String {
    let snippet: String = body.chars().take(KEY_TEXT_LIMIT).collect();
    format!("{name}\n{category}\n{subject}\n{snippet}")
}

fn gather_dates(stem: &str, body: &str, taken: Option<NaiveDateTime>) -> Vec<DateRef> {
    let mut out: Vec<DateRef> = vec![];

    if let Some(t) = taken {
        out.push(DateRef {
            label: "Taken".into(),
            date: t.date().to_string(),
            kind: "taken".into(),
        });
    }
    if let Some(d) = compact_date(stem) {
        if taken.map(|t| t.date()) != Some(d.date()) {
            out.push(DateRef {
                label: "From the filename".into(),
                date: d.to_string(),
                kind: "taken".into(),
            });
        }
    }

    let mut sources = body.to_string();
    sources.push('\n');
    sources.push_str(stem);
    for f in dates::parse_all(&sources) {
        out.push(DateRef {
            label: f.label,
            date: f.date.to_string(),
            kind: f.kind,
        });
    }
    out.sort_by(|a, b| a.date.cmp(&b.date));
    out.dedup_by(|a, b| a.date == b.date && a.kind == b.kind);
    out
}

/// `IMG-20261011-WA0004` / `Screenshot 2026-10-11` carry the day in the name.
fn compact_date(stem: &str) -> Option<NaiveDateTime> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?i)(20\d{2})[-_.]?(0[1-9]|1[0-2])[-_.]?(0[1-9]|[12]\d|3[01])").unwrap()
    });
    static PREFIX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?i)^(img|dsc|dscn|dji|gopr|pxl|vid|scan|screenshot|screen[ _-]?shot|scr|wa|signal|download)").unwrap()
    });
    if !PREFIX.is_match(stem) && !classif_anon(stem) {
        return None;
    }
    for m in RE.captures_iter(stem) {
        let y = m[1].parse::<i32>().ok()?;
        let mo = m[2].parse::<u32>().ok()?;
        let d = m[3].parse::<u32>().ok()?;
        if let Some(date) = NaiveDate::from_ymd_opt(y, mo, d) {
            return date.and_hms_opt(0, 0, 0);
        }
    }
    None
}

fn parse_date(iso: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(iso, "%Y-%m-%d").ok()
}

fn days_since(mtime: i64) -> f32 {
    use chrono::DateTime;
    let now = Local::now();
    match DateTime::from_timestamp(mtime, 0) {
        Some(t) => {
            let d = now
                .signed_duration_since(t.with_timezone(&Local))
                .num_days();
            (d as f32).max(0.0)
        }
        None => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_names_get_a_real_name_from_ocr_text() {
        let subject = subject_of(
            "IMG-20261011-WA0004",
            "City Diagnostics\nBlood test report\nRefill by 2026-11-10",
        );
        assert_eq!(subject, "Blood test report");
        let name = propose_name(
            "IMG-20261011-WA0004",
            compact_date("IMG-20261011-WA0004"),
            "medical / prescription",
            &subject,
        );
        assert_eq!(name, "2026-10-11 Blood test report");
    }

    #[test]
    fn a_meaningful_filename_survives() {
        let subject = subject_of("dr-patel-blood-test-2026-10-11", "");
        assert!(subject.to_lowercase().contains("patel"));
        let name = propose_name(
            "dr-patel-blood-test-2026-10-11",
            None,
            "medical / prescription",
            &subject,
        );
        assert!(name.to_lowercase().contains("blood test"));
    }

    #[test]
    fn no_ocr_still_produces_a_traceable_name() {
        let name = propose_name("IMG_0042", None, "photo", "");
        assert_eq!(name, "photo (IMG_0042)");
    }

    #[test]
    fn compact_dates_only_come_from_machine_names() {
        assert!(compact_date("IMG-20261011-WA0004").is_some());
        assert!(compact_date("Screenshot 2026-10-11 at 10.12.33").is_some());
        assert!(compact_date("invoice-20261011").is_none());
    }

    #[test]
    fn date_refs_are_sorted_and_deduped() {
        let ds = gather_dates(
            "IMG_20261011",
            "refill by 2026-11-10",
            compact_date("IMG_20261011"),
        );
        assert!(ds.windows(2).all(|w| w[0].date <= w[1].date));
        assert!(ds.iter().any(|d| d.date == "2026-11-10"));
        assert!(ds.iter().any(|d| d.date == "2026-10-11"));
    }

    /// Real end-to-end reader test: a rendered receipt goes through the actual
    /// PaddleOCR models and comes back with text, a deadline, and a bbox that
    /// Protect can burn redactions into.
    #[cfg(feature = "ocr")]
    #[test]
    fn real_ocr_reads_text_dates_and_secret_boxes() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures");
        assert!(
            dir.join("IMG-20261011-WA0004.png").is_file(),
            "missing OCR fixture at {}",
            dir.display()
        );
        let reading = read_file(&dir, "IMG-20261011-WA0004.png");
        assert!(
            reading.key_text.to_lowercase().contains("diagnostics"),
            "OCR should read the receipt title; key_text was:\n{}",
            reading.key_text
        );
        assert_eq!(reading.category, "medical / prescription");
        assert!(
            reading.name.contains("Blood test report"),
            "the machine filename should be replaced by the OCR subject; name was {:?}",
            reading.name
        );
        assert!(
            reading
                .dates
                .iter()
                .any(|d| d.date == "2027-09-15" && d.kind == "deadline"),
            "OCR should surface the refill deadline; dates: {:?}",
            reading
                .dates
                .iter()
                .map(|d| (&d.date, &d.kind))
                .collect::<Vec<_>>()
        );
        let id = reading
            .secrets
            .iter()
            .find(|s| s.kind == "id_number")
            .unwrap_or_else(|| {
                panic!(
                    "Aadhaar number should be found in OCR text; secrets: {:?}\nkey_text:\n{}",
                    reading
                        .secrets
                        .iter()
                        .map(|s| (&s.kind, &s.preview, s.bbox))
                        .collect::<Vec<_>>(),
                    reading.key_text
                )
            });
        assert!(
            id.preview.contains('•') && !id.preview.contains("1234"),
            "the finding must be masked in the reading: {:?}",
            id.preview
        );
        assert!(
            id.bbox.is_some(),
            "the finding must carry the OCR box so Protect can pixelate it"
        );
    }
}
