//! What kind of file is this? Extension + filename/OCR heuristics, no model.

use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Image,
    Pdf,
    Text,
    Other,
}

pub fn kind_of(ext: &str) -> Kind {
    match ext {
        "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" | "heic" | "tif" | "tiff" => Kind::Image,
        "pdf" => Kind::Pdf,
        "txt" | "md" | "csv" | "log" | "json" => Kind::Text,
        _ => Kind::Other,
    }
}

/// Category priors feed both naming and the Let-go score.
pub fn category_of(stem: &str, ext_kind: Kind) -> &'static str {
    category_in(stem).unwrap_or_else(|| match ext_kind {
        Kind::Image => "photo",
        Kind::Pdf => "document",
        Kind::Text => "note",
        Kind::Other => "file",
    })
}

/// The filename wins; OCR/PDF text fills the gap. This is what turns a
/// screenshot called `IMG-20261011-WA0004` into "medical / prescription".
pub fn category_of_body(stem: &str, body: &str, ext_kind: Kind) -> &'static str {
    category_in(stem)
        .or_else(|| category_in(body))
        .unwrap_or_else(|| category_of("", ext_kind))
}

fn category_in(text: &str) -> Option<&'static str> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?i)\b(warranty|receipt|bill|invoice|ticket|otp|one[- ]time code|passcode|password|prescription|blood test|lab report|medical|report|admit|exam form|form|passport|aadhaar|licence|license|pan card|pancard|contract|agreement|insurance|statement|certificate)\b",
        )
        .unwrap()
    });

    let m = RE.captures(text)?;
    let w = m.get(1)?.as_str().to_lowercase();
    Some(match w.as_str() {
        "warranty" => "warranty",
        "receipt" => "receipt",
        "bill" | "invoice" => "bill",
        "ticket" => "ticket",
        "otp" | "one-time code" | "one time code" => "one-time code",
        "passcode" | "password" => "credentials",
        "prescription" | "blood test" | "lab report" | "medical" => "medical / prescription",
        "report" => "report",
        "admit" | "exam form" | "form" => "document",
        "passport" | "aadhaar" | "licence" | "license" | "pan card" | "pancard" => "identity",
        "contract" | "agreement" => "contract",
        "insurance" => "insurance",
        "statement" => "statement",
        "certificate" => "certificate",
        _ => "document",
    })
}

/// True for filenames that look like camera/phone machine names: IMG_0042,
/// `IMG-20261011-WA0004`, `Screenshot 2026-10-11 at 10.12.33`, `scan_1(2)`.
pub fn looks_anonymous(stem: &str) -> bool {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?ix)
            ^(?:
                img|dsc|dscn|dji|gopr|gopro|pxl|vid|photo|video|screenshot|screen[ _-]?shot|scr|scan|
                download|downloaded|untitled|new[ _-]?document|document|image|file|whatsapp[ _-]?image|signal
            )
            (?:[\s._-]*(?:\d+|wa\d*|hdr|panorama|burst|edit|copy|final|at|am|pm|\(\d+\)))*$
            ",
        )
        .unwrap()
    });
    RE.is_match(stem)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_names_from_phones_and_cameras_are_anonymous() {
        for name in [
            "IMG_0042",
            "IMG-20261011-WA0004",
            "Screenshot 2026-10-11 at 10.12.33",
            "scan_1(2)",
            "PXL_20240101_123456",
            "download",
        ] {
            assert!(looks_anonymous(name), "{name} should be anonymous");
        }
    }

    #[test]
    fn a_real_subject_is_not_anonymous() {
        for name in ["warranty-laptop", "Dr Patel blood test", "receipt amazon"] {
            assert!(!looks_anonymous(name), "{name} should keep its name");
        }
    }

    #[test]
    fn ocr_text_can_be_the_category() {
        let cat = category_of_body(
            "IMG-20261011-WA0004",
            "City Diagnostics\nBlood test report\nRefill by 2026-11-10",
            Kind::Image,
        );
        assert_eq!(cat, "medical / prescription");
    }
}
