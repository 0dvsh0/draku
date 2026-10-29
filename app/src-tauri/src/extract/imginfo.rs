//! Image intelligence with no model: EXIF/GPS reading and a perceptual hash
//! for duplicate detection. Runs on every image the product reads — fast,
//! deterministic, and fully offline.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use exif::{Tag, Value};

pub struct ImageFacts {
    pub gps: Option<(f64, f64)>,
    pub taken: Option<chrono::NaiveDateTime>,
    /// 16-hex-char dhash when the image decodes.
    pub phash: Option<String>,
}

pub fn probe(path: &Path) -> ImageFacts {
    ImageFacts {
        gps: read_gps(path),
        taken: read_taken(path),
        phash: dhash(path),
    }
}

fn read_exif(path: &Path) -> Option<exif::Exif> {
    let file = File::open(path).ok()?;
    exif::Reader::new()
        .read_from_container(&mut BufReader::new(file))
        .ok()
}

fn rational_of<'a>(fields: &'a [exif::Field], tag: Tag) -> Option<&'a [exif::Rational]> {
    let f = fields.iter().find(|f| f.tag == tag)?;
    match &f.value {
        Value::Rational(v) if !v.is_empty() => Some(v),
        _ => None,
    }
}

fn ascii_hours<'a>(fields: &'a [exif::Field], tag: Tag) -> Option<String> {
    let f = fields.iter().find(|f| f.tag == tag)?;
    match &f.value {
        Value::Ascii(v) => v.first().and_then(|c| {
            let mut s = String::from_utf8(c.to_vec()).ok()?;
            let nul = s.find('\0').unwrap_or(s.len());
            s.truncate(nul);
            Some(s)
        }),
        _ => None,
    }
}

/// Decodes a [N|E|S|W]-ref'd DMS triple into signed decimal degrees.
fn dms(fields: &[exif::Field], tag: Tag, ref_tag: Tag, neg_ref: char) -> Option<f64> {
    let v = rational_of(fields, tag)?;
    if v.len() < 3 {
        return None;
    }
    let deg = v[0].to_f64();
    let min = v[1].to_f64();
    let sec = v[2].to_f64();
    let reference = fields
        .iter()
        .find(|f| f.tag == ref_tag)
        .and_then(|f| match &f.value {
            Value::Ascii(v) => v
                .first()
                .and_then(|c| c.first().copied().map(|b| b as char)),
            _ => None,
        })
        .unwrap_or(ref_tag.to_string().chars().next().unwrap_or('N'))
        .to_ascii_uppercase();
    let sign: f64 = if reference == neg_ref { -1.0 } else { 1.0 };
    Some(sign * (deg + min / 60.0 + sec / 3600.0))
}

fn read_gps(path: &Path) -> Option<(f64, f64)> {
    let exif = read_exif(path)?;
    let fields: Vec<exif::Field> = exif.fields().cloned().collect();
    let lat = dms(&fields, Tag::GPSLatitude, Tag::GPSLatitudeRef, 'S')?;
    let lon = dms(&fields, Tag::GPSLongitude, Tag::GPSLongitudeRef, 'W')?;
    Some((lat, lon))
}

fn read_taken(path: &Path) -> Option<chrono::NaiveDateTime> {
    let exif = read_exif(path)?;
    let fields: Vec<exif::Field> = exif.fields().cloned().collect();
    // DateTimeOriginal, then DateTimeDigitized/DateTime; both "2026:10:11 14:32:09".
    let dt = [Tag::DateTimeOriginal, Tag::DateTimeDigitized, Tag::DateTime]
        .into_iter()
        .find_map(|t| ascii_hours(&fields, t))?;
    chrono::NaiveDateTime::parse_from_str(&dt, "%Y:%m:%d %H:%M:%S").ok()
}

/// dhash: horizontal gradient of a 9×8 grayscale thumbnail → 64 bits.
/// Two files within a Hamming distance of ~8 of each other are the same shot.
pub fn dhash(path: &Path) -> Option<String> {
    let img = image::ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
    // Resize to exactly 9×8: `thumbnail` preserves aspect ratio and can return
    // a shorter row (e.g. 9×6 for a landscape photo), which used to panic.
    let exact = image::imageops::resize(&img, 9, 8, image::imageops::FilterType::Triangle);
    let small = image::imageops::grayscale(&exact);
    let mut bits: u64 = 0;
    for y in 0..8 {
        for x in 0..8 {
            let g = |px: u32, py: u32| small.get_pixel(px, py)[0] as i16;
            if g(x, y) > g(x + 1, y) {
                bits |= 1 << (y * 8 + x);
            }
        }
    }
    Some(format!("{bits:016x}"))
}

pub fn hamming_hex(a: &str, b: &str) -> u32 {
    let (a, b) = match (u64::from_str_radix(a, 16), u64::from_str_radix(b, 16)) {
        (Ok(a), Ok(b)) => (a, b),
        _ => return 64,
    };
    (a ^ b).count_ones()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn reads_gps_and_capture_time_from_real_exif() {
        let facts = probe(&fixture("geo.jpg"));
        let (lat, lon) = facts.gps.expect("fixture has GPS EXIF");
        assert!((lat - 12.9745).abs() < 0.01, "lat was {lat}");
        assert!((lon - 77.5917).abs() < 0.01, "lon was {lon}");
        assert_eq!(
            facts.taken.map(|t| t.to_string()),
            Some("2026-10-11 14:32:09".to_string())
        );
    }

    #[test]
    fn landscape_images_get_a_16_hex_dhash_without_panicking() {
        // 1000×700 used to shrink to 9×6 under `thumbnail`, then index 9×8.
        let hash = dhash(&fixture("IMG-20261011-WA0004.png")).expect("png decodes");
        assert_eq!(hash.len(), 16);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
