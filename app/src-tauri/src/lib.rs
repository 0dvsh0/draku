//! Draku — Your files, finally understood.
//!
//! Everything the product does begins from one law: nothing leaves the
//! machine. The only network call in the whole process is the one-time
//! EmbeddingGemma download on first run (see `embed.rs`).

mod actions;
mod embed;
mod extract;
mod ics;
mod model;
mod scan;
mod score;
mod state;
mod store;

use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use tauri::Manager;

use self::embed::Embedder;
use self::state::DrakuState;
use self::store::Store;

pub fn log_warn(msg: &str) {
    eprintln!("[draku] {msg}");
}

#[cfg(test)]
pub(crate) mod testutil {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Isolated directory under the OS temp dir. Tests clean it up themselves.
    pub fn temp_dir(tag: &str) -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "draku-test-{}-{}-{tag}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).expect("create test temp dir");
        dir
    }

    pub fn cleanup(dir: &std::path::Path) {
        let _ = std::fs::remove_dir_all(dir);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("draku"));
            std::fs::create_dir_all(&dir).ok();
            let store = Store::open(&dir).map_err(|e| {
                log_warn(&format!("store: {e}"));
                "store failed".to_string()
            })?;
            log_warn(&format!("store at {}", dir.join("draku.db").display()));
            let resources = app.path().resource_dir().ok();
            if let Some(dir) = resources.clone() {
                extract::ocr::set_resource_dir(dir);
            }
            let bundled = embed::bundled_model_dir(resources.as_deref());
            app.manage(DrakuState {
                store: Mutex::new(store),
                embedder: Mutex::new(Embedder::new(bundled)),
                scanning: AtomicBool::new(false),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            actions::get_state,
            actions::search,
            actions::start_scan,
            actions::cancel_scan,
            actions::add_to_calendar,
            actions::make_safe_copy,
            actions::set_aside,
            actions::set_aside_many,
            actions::restore,
            actions::restore_many,
            actions::undo_generated,
            actions::reveal_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Draku");
}
