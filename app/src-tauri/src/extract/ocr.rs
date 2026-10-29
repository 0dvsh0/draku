//! Real local OCR (M7): PaddleOCR's detection + angle + recognition models run
//! through ONNX Runtime. The recognizer returns text plus a normalized box,
//! which is what Protect needs to burn redactions into the right pixels.
//!
//! Everything is lazy: the models load on the first image and never touch the
//! network. Builds with `--no-default-features` keep the original no-op path.

use std::path::PathBuf;
use std::sync::OnceLock;

#[cfg(feature = "ocr")]
use once_cell::sync::Lazy;

#[derive(Clone, Debug)]
pub struct Chip {
    pub text: String,
    /// Normalized [x, y, w, h] within the image.
    pub bbox: [f32; 4],
    pub page: Option<u32>,
}

/// Any chip-text producer — PaddleOCR ONNX in production.
pub trait ChipFinder: Send + Sync {
    fn find_chips(&self, image: &image::RgbaImage) -> Vec<Chip>;
}

/// Optional resource directory from Tauri (`.../resources`). Must be set in
/// setup before the first scan; dev builds also fall back to the source tree.
pub fn set_resource_dir(dir: PathBuf) {
    let _ = RESOURCE_DIR.set(dir);
}

static RESOURCE_DIR: OnceLock<PathBuf> = OnceLock::new();

#[cfg(feature = "ocr")]
pub fn models_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("DRAKU_OCR_DIR") {
        let p = PathBuf::from(dir);
        if p.join("det.onnx").is_file() {
            return p;
        }
    }
    if let Some(res) = RESOURCE_DIR.get() {
        let p = res.join("assets").join("ocr");
        if p.join("det.onnx").is_file() {
            return p;
        }
        let p = res.join("ocr");
        if p.join("det.onnx").is_file() {
            return p;
        }
    }
    // Development + tests.
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("ocr");
    p
}

// ---------------------------------------------------------------------------
// Feature-gated PaddleOCR implementation
// ---------------------------------------------------------------------------

#[cfg(feature = "ocr")]
mod paddle {
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;

    use paddle_ocr_rs::ocr_lite::OcrLite;

    use super::{models_dir, Chip, ChipFinder};

    pub struct PaddleChips {
        engine: Mutex<Option<OcrLite>>,
        failed: AtomicBool,
    }

    impl PaddleChips {
        pub fn new() -> Self {
            Self {
                engine: Mutex::new(None),
                failed: AtomicBool::new(false),
            }
        }

        fn with_engine<R>(&self, f: impl FnOnce(&mut OcrLite) -> R) -> Option<R> {
            if self.failed.load(Ordering::Relaxed) {
                return None;
            }
            let mut guard = match self.engine.lock() {
                Ok(g) => g,
                Err(_) => return None,
            };
            if guard.is_none() {
                match init_engine(&models_dir()) {
                    Ok(engine) => *guard = Some(engine),
                    Err(e) => {
                        self.failed.store(true, Ordering::Relaxed);
                        crate::log_warn(&format!("OCR unavailable: {e}"));
                        return None;
                    }
                }
            }
            guard.as_mut().map(f)
        }
    }

    fn init_engine(dir: &Path) -> Result<OcrLite, String> {
        let det = dir.join("det.onnx");
        let cls = dir.join("cls.onnx");
        let rec = dir.join("rec.onnx");
        let dict = dir.join("dict.txt");
        for p in [&det, &cls, &rec, &dict] {
            if !p.is_file() {
                return Err(format!("missing model {}", p.display()));
            }
        }
        let threads = std::thread::available_parallelism()
            .map(|n| n.get().min(4))
            .unwrap_or(2);
        let mut ocr = OcrLite::new();
        ocr.init_models_with_dict(
            &path_str(&det)?,
            &path_str(&cls)?,
            &path_str(&rec)?,
            &path_str(&dict)?,
            threads,
        )
        .map_err(|e| e.to_string())?;
        Ok(ocr)
    }

    fn path_str(p: &Path) -> Result<String, String> {
        p.to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| format!("non-UTF-8 model path: {}", p.display()))
    }

    impl ChipFinder for PaddleChips {
        fn find_chips(&self, image: &image::RgbaImage) -> Vec<Chip> {
            let (w, h) = image.dimensions();
            if w == 0 || h == 0 {
                return vec![];
            }
            let rgb = image::DynamicImage::ImageRgba8(image.clone()).to_rgb8();
            let result =
                self.with_engine(|ocr| ocr.detect(&rgb, 50, 1600, 0.5, 0.3, 1.6, false, false));
            let Some(result) = result else {
                return vec![];
            };
            let result = match result {
                Ok(result) => result,
                Err(e) => {
                    crate::log_warn(&format!("OCR detect failed: {e}"));
                    return vec![];
                }
            };

            let mut chips: Vec<Chip> = result
                .text_blocks
                .into_iter()
                .filter(|b| b.text_score >= 0.40)
                .filter_map(|b| {
                    let text = b.text.trim().to_string();
                    if text.is_empty() || b.box_points.is_empty() {
                        return None;
                    }
                    let mut x0 = u32::MAX;
                    let mut y0 = u32::MAX;
                    let mut x1 = 0u32;
                    let mut y1 = 0u32;
                    for p in &b.box_points {
                        x0 = x0.min(p.x);
                        y0 = y0.min(p.y);
                        x1 = x1.max(p.x);
                        y1 = y1.max(p.y);
                    }
                    let x0 = (x0 as f32 / w as f32).clamp(0.0, 1.0);
                    let y0 = (y0 as f32 / h as f32).clamp(0.0, 1.0);
                    let x1 = (x1 as f32 / w as f32).clamp(0.0, 1.0);
                    let y1 = (y1 as f32 / h as f32).clamp(0.0, 1.0);
                    if x1 <= x0 || y1 <= y0 {
                        return None;
                    }
                    Some(Chip {
                        text,
                        bbox: [x0, y0, x1 - x0, y1 - y0],
                        page: None,
                    })
                })
                .collect();

            // Reading order: top to bottom, then left to right.
            chips.sort_by(|a, b| {
                let ay = a.bbox[1];
                let by = b.bbox[1];
                if (ay - by).abs() > 0.02 {
                    ay.partial_cmp(&by).unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    a.bbox[0]
                        .partial_cmp(&b.bbox[0])
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
            });
            chips
        }
    }
}

#[cfg(not(feature = "ocr"))]
pub struct NoOcr;

#[cfg(not(feature = "ocr"))]
impl ChipFinder for NoOcr {
    fn find_chips(&self, _image: &image::RgbaImage) -> Vec<Chip> {
        Vec::new()
    }
}

#[cfg(feature = "ocr")]
pub static CHIP_FINDER: Lazy<paddle::PaddleChips> = Lazy::new(paddle::PaddleChips::new);

#[cfg(not(feature = "ocr"))]
pub static CHIP_FINDER: NoOcr = NoOcr;
