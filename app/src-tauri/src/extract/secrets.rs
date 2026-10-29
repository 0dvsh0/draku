//! Local secret detection from real text (PDF text, TXT bodies, OCR output when
//! the `ocr` feature is on). Every detection shows masked in the UI and is
//! confirmed before a safe copy — "no guessing out loud".

use once_cell::sync::Lazy;
use regex::Regex;

use crate::model::SecretRef;

static CARD: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(?:\d[ -]?){13,19}\b").unwrap());

static AADHAAR: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b\d{4}\s?\d{4}\s?\d{4}\b").unwrap());

static KEY_VALUE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        r"(?i)\b(?:password|passwd|pwd|passcode|pass code|pass phrase|passphrase",
        r"|pin|network key|security key|otp|verification code)\s*[:=]?\s*",
        r"([A-Za-z0-9@#$%^&*+=_\-/\\]{3,40})"
    ))
    .unwrap()
});

static GPS_DECIMAL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(?:gps|coords?|coordinates|location)\s*[:=]?\s*(-?\d{1,3}\.\d{3,})\s*,\s*(-?\d{1,3}\.\d{3,})",
    )
    .unwrap()
});

fn mask(v: &str) -> String {
    let v = v.trim();
    let chars = v.chars().collect::<Vec<_>>();
    let n = chars.len();
    if n <= 6 {
        if n <= 2 {
            return "•".repeat(n);
        }
        let mut s = String::new();
        s.push(chars[0]);
        for _ in 1..n.saturating_sub(1) {
            s.push('•');
        }
        s.push(chars[n - 1]);
        s
    } else {
        let head: String = chars.iter().take(2).collect();
        let tail = chars[n - 1];
        let dots = (n.saturating_sub(3)).min(14);
        format!("{}{}{}", head, "•".repeat(dots), tail)
    }
}

fn push_if_news(out: &mut Vec<SecretRef>, s: SecretRef) {
    if out
        .iter()
        .any(|x| x.kind == s.kind && x.preview == s.preview)
    {
        return;
    }
    out.push(s);
}

/// Extract privacy findings from a body of text.
pub fn parse_text(text: &str, page: Option<u32>) -> Vec<SecretRef> {
    parse_text_at(text, page, None)
}

/// Same detector, with the OCR box attached. Protect uses this box to burn a
/// pixelation into the safe copy.
pub fn parse_text_at(text: &str, page: Option<u32>, bbox: Option<[f32; 4]>) -> Vec<SecretRef> {
    let mut out: Vec<SecretRef> = vec![];

    for m in KEY_VALUE.captures_iter(text) {
        push_if_news(
            &mut out,
            SecretRef {
                kind: "password".into(),
                preview: mask(m.get(1).map(|g| g.as_str()).unwrap_or("")),
                page,
                bbox,
            },
        );
    }

    if let Some(m) = AADHAAR.find(text) {
        // Sanity: not a timestamp and not the first 12 digits of a longer card
        // number that happens to be split by the same spacing.
        if !m.as_str().contains('-') && digits_around(text, m.start(), m.end()) <= 12 {
            push_if_news(
                &mut out,
                SecretRef {
                    kind: "id_number".into(),
                    preview: mask(m.as_str()),
                    page,
                    bbox,
                },
            );
        }
    }

    if let Some(m) = CARD.find(text) {
        let digits = m.as_str().chars().filter(|c| c.is_ascii_digit()).count();
        // 13..19 with at least one separator OR plain 16 (card-like, not Aadhaar 12)
        if (13..=19).contains(&digits)
            && (m.as_str().contains(char::is_whitespace)
                || m.as_str().contains('-')
                || digits == 16)
        {
            push_if_news(
                &mut out,
                SecretRef {
                    kind: "card_number".into(),
                    preview: mask(m.as_str()),
                    page,
                    bbox,
                },
            );
        }
    }

    for m in GPS_DECIMAL.captures_iter(text) {
        if let (Some(a), Some(b)) = (m.get(1), m.get(2)) {
            if a.as_str().parse::<f64>().is_ok() && b.as_str().parse::<f64>().is_ok() {
                push_if_news(
                    &mut out,
                    SecretRef {
                        kind: "location".into(),
                        preview: format!("{}, {}", mask(a.as_str()), mask(b.as_str())),
                        page,
                        bbox,
                    },
                );
            }
        }
    }

    out
}

/// Count digits in the contiguous run of digits/spaces/hyphens around a match.
fn digits_around(text: &str, start: usize, end: usize) -> usize {
    let bytes = text.as_bytes();
    let mut left = start;
    while left > 0 && matches!(bytes[left - 1], b'0'..=b'9' | b' ' | b'-' | b'.') {
        left -= 1;
    }
    let mut right = end;
    while right < bytes.len() && matches!(bytes[right], b'0'..=b'9' | b' ' | b'-' | b'.') {
        right += 1;
    }
    text[left..right]
        .chars()
        .filter(|c| c.is_ascii_digit())
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_card_without_calling_it_an_aadhaar() {
        let found = parse_text("card 4111 1111 1111 1111 expires 12/28", None);
        assert!(found.iter().any(|s| s.kind == "card_number"));
        assert!(!found.iter().any(|s| s.kind == "id_number"));
    }

    #[test]
    fn finds_a_twelve_digit_id() {
        let found = parse_text("Aadhaar 1234 5678 9012", None);
        assert!(found.iter().any(|s| s.kind == "id_number"));
    }

    #[test]
    fn carries_the_ocr_box_into_the_finding() {
        let bbox = [0.1, 0.2, 0.3, 0.04];
        let found = parse_text_at("password: hunter2", Some(1), Some(bbox));
        assert_eq!(found[0].bbox, Some(bbox));
        assert!(found[0].preview.contains('•'));
        assert_ne!(found[0].preview, "hunter2");
    }
}
