//! SQLite store for readings. One database, in the app-data dir, forever local.

use std::path::Path;

use rusqlite::Connection;

use crate::model::{Reading, SearchHit, SecretRef};

pub struct Store {
    pub conn: Connection,
}

/// What the DB row holds for the action commands.
#[derive(Clone, Debug)]
pub struct Row {
    pub id: i64,
    pub root: String,
    pub rel: String,
    pub name: String,
    pub category: String,
    pub size: u64,
}

impl Store {
    pub fn open(dir: &Path) -> rusqlite::Result<Self> {
        std::fs::create_dir_all(dir).ok();
        let conn = Connection::open(dir.join("draku.db"))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS files (
                id        INTEGER PRIMARY KEY,
                file_id   TEXT UNIQUE NOT NULL,
                root      TEXT NOT NULL,
                rel       TEXT NOT NULL,
                name      TEXT NOT NULL,
                category  TEXT NOT NULL,
                key_text  TEXT NOT NULL DEFAULT '',
                ext       TEXT NOT NULL DEFAULT '',
                size      INTEGER NOT NULL DEFAULT 0,
                mtime     INTEGER NOT NULL DEFAULT 0,
                phash     TEXT,
                dup_of    TEXT,
                dead      REAL NOT NULL DEFAULT 0.0,
                dead_reason TEXT,
                gps_lat   REAL,
                gps_lon   REAL,
                emb       BLOB
            );
            CREATE INDEX IF NOT EXISTS idx_files_root ON files(root);
            CREATE INDEX IF NOT EXISTS idx_files_dead ON files(dead);

            CREATE TABLE IF NOT EXISTS file_dates (
                id      INTEGER PRIMARY KEY,
                file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
                label   TEXT NOT NULL,
                date    TEXT NOT NULL,
                kind    TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS file_secrets (
                id      INTEGER PRIMARY KEY,
                file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
                kind    TEXT NOT NULL,
                preview TEXT NOT NULL,
                page    INTEGER,
                bbox    TEXT
            );

            CREATE TABLE IF NOT EXISTS rest_log (
                id         INTEGER PRIMARY KEY,
                file_id    TEXT NOT NULL,
                root       TEXT NOT NULL,
                rel        TEXT NOT NULL,
                rest_rel   TEXT NOT NULL,
                moved_at   INTEGER NOT NULL,
                restored_at INTEGER
            );

            CREATE TABLE IF NOT EXISTS calendar_log (
                id        INTEGER PRIMARY KEY,
                file_id   TEXT NOT NULL,
                label     TEXT NOT NULL,
                date      TEXT NOT NULL,
                ics_path  TEXT NOT NULL,
                made_at   INTEGER NOT NULL
            );
            "#,
        )?;
        Ok(Self { conn })
    }

    pub fn upsert_reading(&self, r: &Reading, emb: Option<&[f32]>) -> rusqlite::Result<i64> {
        let emb_blob = emb.map(|v| {
            let mut b = Vec::with_capacity(v.len() * 4);
            for f in v {
                b.extend_from_slice(&f.to_le_bytes());
            }
            b
        });
        self.conn.execute(
            r#"INSERT INTO files
                 (file_id, root, rel, name, category, key_text, ext, size, mtime,
                  phash, dup_of, dead, dead_reason, gps_lat, gps_lon, emb)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)
               ON CONFLICT(file_id) DO UPDATE SET
                 name=excluded.name, category=excluded.category, key_text=excluded.key_text,
                 ext=excluded.ext, size=excluded.size, mtime=excluded.mtime,
                 phash=excluded.phash, dup_of=excluded.dup_of, dead=excluded.dead,
                 dead_reason=excluded.dead_reason, gps_lat=excluded.gps_lat,
                 gps_lon=excluded.gps_lon, emb=COALESCE(excluded.emb, files.emb)"#,
            rusqlite::params![
                r.file_id(),
                r.root,
                r.rel,
                r.name,
                r.category,
                r.key_text,
                std::path::Path::new(&r.rel)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase(),
                r.size as i64,
                r.mtime,
                r.phash,
                r.dup_of,
                r.dead,
                r.dead_reason,
                r.gps.map(|g| g.0),
                r.gps.map(|g| g.1),
                emb_blob
            ],
        )?;
        // `last_insert_rowid()` is meaningless after an ON CONFLICT DO UPDATE:
        // it still points at the previous insert. Ask for the row we just wrote.
        let id: i64 = self.conn.query_row(
            "SELECT id FROM files WHERE file_id=?1",
            [r.file_id()],
            |row| row.get(0),
        )?;

        self.conn
            .execute("DELETE FROM file_dates WHERE file_id=?1", [id])?;
        for d in &r.dates {
            self.conn.execute(
                "INSERT INTO file_dates (file_id,label,date,kind) VALUES (?1,?2,?3,?4)",
                rusqlite::params![id, d.label, d.date, d.kind],
            )?;
        }
        self.conn
            .execute("DELETE FROM file_secrets WHERE file_id=?1", [id])?;
        for s in &r.secrets {
            self.conn.execute(
                "INSERT INTO file_secrets (file_id,kind,preview,page,bbox) VALUES (?1,?2,?3,?4,?5)",
                rusqlite::params![
                    id,
                    s.kind,
                    s.preview,
                    s.page,
                    s.bbox.map(|b| serde_json::to_string(&b).unwrap())
                ],
            )?;
        }
        Ok(id)
    }

    pub fn row_by_id(&self, file_id: &str) -> Option<Row> {
        self.conn
            .query_row(
                r#"SELECT id, root, rel, name, category, size FROM files WHERE file_id=?1"#,
                [file_id],
                |r| {
                    Ok(Row {
                        id: r.get(0)?,
                        root: r.get(1)?,
                        rel: r.get(2)?,
                        name: r.get(3)?,
                        category: r.get(4)?,
                        size: r.get::<_, i64>(5)?.max(0) as u64,
                    })
                },
            )
            .ok()
    }

    pub fn embeds(&self) -> Vec<(String, Vec<f32>)> {
        let mut out = vec![];
        let mut st = match self
            .conn
            .prepare("SELECT file_id, emb FROM files WHERE emb IS NOT NULL")
        {
            Ok(s) => s,
            Err(_) => return out,
        };
        let rows = st.query_map([], |r| {
            let id: String = r.get(0)?;
            let blob: Vec<u8> = r.get(1)?;
            Ok((id, blob))
        });
        if let Ok(rows) = rows {
            for row in rows.flatten() {
                if row.1.len() % 4 != 0 {
                    continue;
                }
                let v: Vec<f32> = row
                    .1
                    .chunks_exact(4)
                    .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                    .collect();
                out.push((row.0, v));
            }
        }
        out
    }

    pub fn roots(&self) -> Vec<String> {
        let mut st = match self
            .conn
            .prepare("SELECT DISTINCT root FROM files ORDER BY root")
        {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        st.query_map([], |r| r.get::<_, String>(0))
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default()
    }

    pub fn count(&self) -> usize {
        // Every file that was read is indexed, duplicates included. The Let-go
        // card is where a duplicate is surfaced — hiding it here made the
        // promise "duplicates cleaned up" untrue.
        self.conn
            .query_row("SELECT COUNT(*) FROM files", [], |r| r.get::<_, i64>(0))
            .map(|n| n as usize)
            .unwrap_or(0)
    }

    /// File ids currently in the rest folder (not restored).
    pub fn rested_ids(&self) -> Vec<String> {
        let mut st = match self
            .conn
            .prepare("SELECT file_id FROM rest_log WHERE restored_at IS NULL")
        {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        st.query_map([], |r| r.get::<_, String>(0))
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default()
    }

    /// Text search used before the embedding model is ready and whenever the
    /// semantic pass finds nothing. Covers the whole reading, not just names.
    pub fn search_text(&self, query: &str, limit: usize) -> Vec<SearchHit> {
        let escaped = query
            .replace('\\', r"\\")
            .replace('%', r"\%")
            .replace('_', r"\_");
        let pat = format!("%{escaped}%");
        let mut st = match self.conn.prepare(
            r#"SELECT file_id, root, rel, name, category FROM files
               WHERE (name LIKE ?1 ESCAPE '\'
                   OR category LIKE ?1 ESCAPE '\'
                   OR key_text LIKE ?1 ESCAPE '\')
                 AND file_id NOT IN (SELECT file_id FROM rest_log WHERE restored_at IS NULL)
               ORDER BY dead ASC, name ASC
               LIMIT ?2"#,
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        st.query_map(rusqlite::params![pat, limit as i64], |r| {
            Ok(SearchHit {
                file_id: r.get(0)?,
                path: format!(
                    "{}{}{}",
                    r.get::<_, String>(1)?,
                    std::path::MAIN_SEPARATOR,
                    r.get::<_, String>(2)?
                ),
                name: r.get(3)?,
                category: r.get(4)?,
                score: 0.5,
            })
        })
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
    }

    pub fn secrets_of(&self, db_id: i64) -> Vec<SecretRef> {
        let mut st = match self
            .conn
            .prepare("SELECT kind, preview, page, bbox FROM file_secrets WHERE file_id=?1")
        {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        st.query_map([db_id], |r| {
            Ok(SecretRef {
                kind: r.get(0)?,
                preview: r.get(1)?,
                page: r.get(2)?,
                bbox: r
                    .get::<_, Option<String>>(3)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
            })
        })
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DateRef;
    use crate::testutil::{cleanup, temp_dir};

    fn reading(root: &Path, rel: &str, name: &str, category: &str) -> Reading {
        Reading {
            root: root.display().to_string(),
            rel: rel.into(),
            name: name.into(),
            category: category.into(),
            key_text: "Blood test report refill by 2026-11-10".into(),
            dates: vec![DateRef {
                label: "Refill by".into(),
                date: "2026-11-10".into(),
                kind: "deadline".into(),
            }],
            secrets: vec![SecretRef {
                kind: "id_number".into(),
                preview: "12••••••••••2".into(),
                page: None,
                bbox: Some([0.1, 0.2, 0.3, 0.04]),
            }],
            gps: None,
            phash: None,
            dup_of: None,
            dead: 0.1,
            dead_reason: None,
            size: 42,
            mtime: 1_700_000_000,
        }
    }

    #[test]
    fn upsert_returns_the_same_id_and_replaces_children() {
        let dir = temp_dir("store-upsert");
        let store = Store::open(&dir).expect("open store");
        let r = reading(
            &dir,
            "scans/IMG-1.png",
            "2026-10-11 Blood test report",
            "medical / prescription",
        );

        let first = store
            .upsert_reading(&r, Some(&[0.25, 0.5, 0.75]))
            .expect("insert");
        let second = store.upsert_reading(&r, None).expect("update");

        assert_eq!(first, second, "ON CONFLICT must not change the row id");
        assert_eq!(
            store.count(),
            1,
            "re-reading the same file must not duplicate it"
        );
        let dates: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM file_dates", [], |r| r.get(0))
            .unwrap();
        assert_eq!(dates, 1, "child dates must be replaced, not appended");
        let secrets: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM file_secrets", [], |r| r.get(0))
            .unwrap();
        assert_eq!(secrets, 1, "child secrets must be replaced, not appended");

        let embeds = store.embeds();
        assert_eq!(embeds.len(), 1);
        assert_eq!(
            embeds[0].1,
            vec![0.25, 0.5, 0.75],
            "a no-embedding rescan keeps the old vector"
        );

        drop(store);
        cleanup(&dir);
    }

    #[test]
    fn text_search_finds_the_body_escapes_wildcards_and_skips_rested_files() {
        let dir = temp_dir("store-search");
        let store = Store::open(&dir).expect("open store");
        let blood = reading(&dir, "a.png", "Blood test", "medical / prescription");
        let mut invoice = reading(&dir, "b.pdf", "March invoice", "bill");
        invoice.key_text = "Payment received for March services".into();
        store.upsert_reading(&blood, None).unwrap();
        store.upsert_reading(&invoice, None).unwrap();

        let hits = store.search_text("blood", 8);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].name, "Blood test");
        assert!(
            store.search_text("refill", 8).len() >= 1,
            "key_text is searchable"
        );
        assert!(
            store.search_text("%", 8).is_empty(),
            "LIKE wildcards are escaped"
        );
        assert!(
            store.search_text("_", 8).is_empty(),
            "LIKE wildcards are escaped"
        );

        store
            .conn
            .execute(
                "INSERT INTO rest_log (file_id,root,rel,rest_rel,moved_at) VALUES (?1,?2,?3,?4,?5)",
                rusqlite::params![
                    blood.file_id(),
                    blood.root,
                    blood.rel,
                    ".draku/rest/a.png",
                    1
                ],
            )
            .unwrap();
        assert_eq!(store.rested_ids(), vec![blood.file_id()]);
        assert!(
            store.search_text("blood", 8).is_empty(),
            "rested files stay out of search"
        );
        assert_eq!(store.count(), 2, "resting does not erase the reading");

        drop(store);
        cleanup(&dir);
    }
}
