//! Shared app state. The store and the embedder sit behind mutexes; nothing
//! else in the process touches the filesystem except the actions module.

use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use chrono::{Days, Local, NaiveDate};

use crate::model::{AppState, DeadlineItem, RestItem, SecretItem};
use crate::score::DEAD_CUTOFF;
use crate::store::Store;

pub struct DrakuState {
    pub store: Mutex<Store>,
    pub embedder: Mutex<crate::embed::Embedder>,
    /// True while a scan thread is running; a second scan comes back as an error.
    pub scanning: AtomicBool,
}

impl DrakuState {
    /// Build the whole One Screen in one go: deadlines, secrets, let-go.
    pub fn snapshot(&self) -> AppState {
        let store = self.store.lock().unwrap();
        let today = Local::now().date_naive();
        let horizon: Option<NaiveDate> = today.checked_add_days(Days::new(365));

        let mut deadlines: Vec<DeadlineItem> = vec![];
        {
            let mut st = match store
                .conn
                .prepare(
                    r#"SELECT f.file_id, f.root, f.rel, f.name, d.label, d.date
                       FROM files f JOIN file_dates d ON d.file_id = f.id
                       WHERE d.kind = 'deadline' AND d.date >= ?1 AND d.date <= ?2
                         AND f.file_id NOT IN (SELECT file_id FROM rest_log WHERE restored_at IS NULL)
                       ORDER BY d.date ASC"#,
                )
                .map_err(|_| ())
            {
                Ok(s) => s,
                Err(_) => return AppState::empty(),
            };
            let mapped = st.query_map(
                [
                    today.to_string(),
                    horizon.map(|h| h.to_string()).unwrap_or_default(),
                ],
                |r| {
                    Ok(DeadlineItem {
                        file_id: r.get(0)?,
                        path: format!(
                            "{}{}{}",
                            r.get::<_, String>(1)?,
                            std::path::MAIN_SEPARATOR,
                            r.get::<_, String>(2)?
                        ),
                        name: r.get(3)?,
                        label: r.get(4)?,
                        date: r.get(5)?,
                        days_left: 0,
                    })
                },
            );
            for mut d in mapped.into_iter().flatten().flatten() {
                let days = NaiveDate::parse_from_str(&d.date, "%Y-%m-%d")
                    .ok()
                    .and_then(|x| (x - today).num_days().try_into().ok())
                    .unwrap_or(0);
                if deadlines.iter().any(|x| x.file_id == d.file_id) {
                    // only the soonest date per file
                    continue;
                }
                d.days_left = days;
                deadlines.push(d);
            }
        }

        let mut secrets: Vec<SecretItem> = vec![];
        {
            let mut st = match store
                .conn
                .prepare(
                    r#"SELECT f.file_id, f.root, f.rel, f.name, s.kind, s.preview,
                              f.gps_lat, f.gps_lon,
                              CASE WHEN s.bbox IS NOT NULL THEN 1 ELSE 0 END
                       FROM files f LEFT JOIN file_secrets s ON s.file_id = f.id
                       WHERE (s.id IS NOT NULL OR f.gps_lat IS NOT NULL)
                         AND f.file_id NOT IN (SELECT file_id FROM rest_log WHERE restored_at IS NULL)"#,
                )
                .map_err(|_| ())
            {
                Ok(s) => s,
                Err(_) => return AppState::empty(),
            };
            let mapped = st.query_map([], |r| {
                let kind = r
                    .get::<_, Option<String>>(4)?
                    .or_else(|| {
                        let gps = r.get::<_, Option<f64>>(6).ok()?;
                        gps.is_some().then(|| "location".to_string())
                    })
                    .unwrap_or_else(|| "secret".into());
                Ok(SecretItem {
                    file_id: r.get(0)?,
                    path: format!(
                        "{}{}{}",
                        r.get::<_, String>(1)?,
                        std::path::MAIN_SEPARATOR,
                        r.get::<_, String>(2)?
                    ),
                    name: r.get(3)?,
                    kind,
                    preview: r.get::<_, Option<String>>(5)?.unwrap_or_default(),
                    has_gps: r.get::<_, Option<f64>>(6)?.is_some(),
                    redactable: r.get::<_, i64>(8)? != 0,
                })
            });
            for s in mapped.into_iter().flatten().flatten() {
                if secrets.iter().any(|x| x.file_id == s.file_id) {
                    continue;
                }
                secrets.push(s);
            }
            // Lead with a finding the app can actually burn out of pixels, not
            // a PDF that it can only copy as-is.
            secrets.sort_by(|a, b| {
                b.redactable
                    .cmp(&a.redactable)
                    .then_with(|| b.has_gps.cmp(&a.has_gps))
            });
        }

        let mut rest: Vec<RestItem> = vec![];
        let mut rest_freed: u64 = 0;
        {
            let mut st = match store
                .conn
                .prepare(
                    r#"SELECT file_id, root, rel, name, dead, dead_reason, size, mtime
                       FROM files
                       WHERE dead >= ?1
                         AND file_id NOT IN (SELECT file_id FROM rest_log WHERE restored_at IS NULL)"#,
                )
                .map_err(|_| ())
            {
                Ok(s) => s,
                Err(_) => return AppState::empty(),
            };
            let now = Local::now().timestamp();
            let mapped = st.query_map([DEAD_CUTOFF], |r| {
                let mtime = r.get::<_, i64>(7)?;
                let age_days = ((now - mtime).max(0) / 86_400) as u32;
                Ok(RestItem {
                    file_id: r.get(0)?,
                    path: format!(
                        "{}{}{}",
                        r.get::<_, String>(1)?,
                        std::path::MAIN_SEPARATOR,
                        r.get::<_, String>(2)?
                    ),
                    name: r.get(3)?,
                    size: r.get::<_, i64>(6)?.max(0) as u64,
                    age_days,
                    dead: r.get(4)?,
                    dead_reason: r.get(5)?,
                })
            });
            for item in mapped.into_iter().flatten().flatten() {
                rest_freed += item.size;
                rest.push(item);
            }
        }
        rest.sort_by(|a, b| {
            b.dead
                .partial_cmp(&a.dead)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        AppState {
            deadlines,
            secrets,
            rest,
            files_total: store.count(),
            rest_freed,
            roots: store.roots(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;
    use std::sync::Mutex;

    use chrono::{Days, Local};

    use super::*;
    use crate::model::{DateRef, Reading, SecretRef};
    use crate::testutil::{cleanup, temp_dir};

    fn reading(root: &std::path::Path, rel: &str, name: &str, dead: f32) -> Reading {
        Reading {
            root: root.display().to_string(),
            rel: rel.into(),
            name: name.into(),
            category: "medical / prescription".into(),
            key_text: name.into(),
            dates: vec![],
            secrets: vec![],
            gps: None,
            phash: None,
            dup_of: None,
            dead,
            dead_reason: Some("test reading".into()),
            size: 1_000,
            mtime: Local::now().timestamp(),
        }
    }

    #[test]
    fn snapshot_hides_rested_files_and_past_deadlines_but_keeps_duplicates() {
        let dir = temp_dir("state-snapshot");
        let store = Store::open(&dir).expect("open store");
        let today = Local::now().date_naive();

        let mut active = reading(&dir, "active.pdf", "Renew insurance", 0.2);
        active.dates.push(DateRef {
            label: "Renew by".into(),
            date: (today + Days::new(10)).to_string(),
            kind: "deadline".into(),
        });
        active.secrets.push(SecretRef {
            kind: "password".into(),
            preview: "hu••••2".into(),
            page: None,
            bbox: None,
        });

        let mut duplicate = reading(&dir, "duplicate.png", "Old lab report", 0.95);
        duplicate.dup_of = Some("active.pdf".into());
        duplicate.mtime = Local::now().timestamp() - 10 * 86_400;
        duplicate.dates.push(DateRef {
            label: "Expired on".into(),
            date: (today - Days::new(3)).to_string(),
            kind: "deadline".into(),
        });
        duplicate.secrets.push(SecretRef {
            kind: "id_number".into(),
            preview: "12••••••••••2".into(),
            page: None,
            bbox: None,
        });

        let mut rested = reading(&dir, "rested.png", "Rested secret", 0.99);
        rested.dates.push(DateRef {
            label: "Renew by".into(),
            date: (today + Days::new(5)).to_string(),
            kind: "deadline".into(),
        });
        rested.secrets.push(SecretRef {
            kind: "card_number".into(),
            preview: "41••••••••••••11".into(),
            page: None,
            bbox: None,
        });

        store.upsert_reading(&active, None).unwrap();
        store.upsert_reading(&duplicate, None).unwrap();
        store.upsert_reading(&rested, None).unwrap();
        store
            .conn
            .execute(
                "INSERT INTO rest_log (file_id,root,rel,rest_rel,moved_at) VALUES (?1,?2,?3,?4,?5)",
                rusqlite::params![
                    rested.file_id(),
                    rested.root,
                    rested.rel,
                    ".draku/rest/rested.png",
                    1
                ],
            )
            .unwrap();

        let state = DrakuState {
            store: Mutex::new(store),
            embedder: Mutex::new(crate::embed::Embedder::new()),
            scanning: AtomicBool::new(false),
        };
        let snap = state.snapshot();

        assert_eq!(
            snap.files_total, 3,
            "duplicates are real files and must be counted"
        );
        assert_eq!(
            snap.deadlines.len(),
            1,
            "past deadlines and rested files are not deadlines"
        );
        assert_eq!(snap.deadlines[0].name, "Renew insurance");
        assert_eq!(snap.deadlines[0].days_left, 10);

        assert_eq!(
            snap.rest.len(),
            1,
            "the duplicate is still offered in Let go"
        );
        assert_eq!(snap.rest[0].name, "Old lab report");
        assert!((snap.rest[0].dead - 0.95).abs() < f32::EPSILON);
        assert!(
            (9..=10).contains(&snap.rest[0].age_days),
            "age_days must be real, got {}",
            snap.rest[0].age_days
        );
        assert_eq!(snap.rest_freed, 1_000);

        assert_eq!(
            snap.secrets.len(),
            2,
            "rested secrets disappear from Protect"
        );
        assert!(snap.secrets.iter().any(|s| s.name == "Renew insurance"));
        assert!(snap.secrets.iter().any(|s| s.name == "Old lab report"));
        assert!(!snap.secrets.iter().any(|s| s.name == "Rested secret"));

        drop(state);
        cleanup(&dir);
    }
}
