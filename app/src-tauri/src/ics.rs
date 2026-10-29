//! Remember — `.ics` calendar files. A date becomes a calendar entry only
//! when the user taps it (promise #3: no guessing out loud).

use std::fs;
use std::path::{Path, PathBuf};

pub struct Event {
    pub label: String,
    /// ISO date.
    pub date: String,
    /// Human source line, e.g. found inside of which file.
    pub source: String,
    pub file_stem: String,
}

pub fn ics_body(ev: &Event, today: &str) -> String {
    let stamp = today.replace('-', "");
    let date = ev.date.replace('-', "");
    let uid = format!(
        "draku-{}@draku.local",
        blake3_ish(&format!("{}|{}", ev.date, ev.source))
    );
    format!(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         PRODID:-//Draku//on-device file manager//EN\r\n\
         BEGIN:VEVENT\r\n\
         UID:{uid}\r\n\
         DTSTAMP:{stamp}T000000Z\r\n\
         DTSTART;VALUE=DATE:{date}\r\n\
         SUMMARY:{}\r\n\
         DESCRIPTION:{}\r\n\
         BEGIN:VALARM\r\n\
         TRIGGER:-PT60M\r\n\
         ACTION:DISPLAY\r\n\
         DESCRIPTION:{}\r\n\
         END:VALARM\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n",
        ev.label, ev.source, ev.label
    )
}

/// FNV-1a — short, dependency-free, and scope-limited to UIDs.
fn blake3_ish(s: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{h:x}")
}

/// Writes the event next to nothing the user didn't point at: inside the
/// scan root's `.draku/calendar/`, and turns the path back to the caller.
pub fn write(root: &Path, ev: &Event, today: &str) -> std::io::Result<PathBuf> {
    let dir = root.join(".draku").join("calendar");
    fs::create_dir_all(&dir)?;
    let slug: String = ev
        .file_stem
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let path = dir.join(format!("{}-{}.ics", ev.date, slug));
    fs::write(&path, ics_body(ev, today))?;
    Ok(path)
}
