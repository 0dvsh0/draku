//! Find — on-device semantic search. Google's EmbeddingGemma-300M (4-bit ONNX)
//! ships inside the installer, so the first scan works with the network cable
//! unplugged. If the bundled copy is missing (a dev checkout or a stripped
//! build), the official model is fetched once into the local fastembed cache
//! and every search after that runs with zero network.

use std::path::{Path, PathBuf};

use fastembed::{
    EmbeddingModel, InitOptionsUserDefined, OutputKey, Pooling, QuantizationMode, TextEmbedding,
    TextInitOptions, TokenizerFiles, UserDefinedEmbeddingModel,
};

const MODEL_FILES: [&str; 6] = [
    "model_q4.onnx",
    "model_q4.onnx_data",
    "tokenizer.json",
    "config.json",
    "special_tokens_map.json",
    "tokenizer_config.json",
];

pub struct Embedder {
    model: Option<TextEmbedding>,
    /// Directory holding the model that ships with the app, if there is one.
    bundled: Option<PathBuf>,
}

fn cache_dir() -> PathBuf {
    // Defer to fastembed's own convention so users who already have the model
    // from another tool stay consistent.
    if let Ok(dir) = std::env::var("FASTEMBED_CACHE_DIR") {
        return PathBuf::from(dir);
    }
    dirs_cache()
}

fn dirs_cache() -> PathBuf {
    // No extra crate for the one dir fastembed names itself.
    let base = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".cache")
        });
    base.join("fastembed")
}

/// The installed app keeps resources beside `draku.exe`; a dev checkout keeps
/// them in `src-tauri/resources`. Both are checked so `tauri dev` exercises the
/// same code path as the installer.
pub fn bundled_model_dir(resource_dir: Option<&Path>) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = vec![];
    if let Some(resources) = resource_dir {
        candidates.push(resources.join("models").join("embeddinggemma"));
    }
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("models")
            .join("embeddinggemma"),
    );
    candidates
        .into_iter()
        .find(|dir| MODEL_FILES.iter().all(|f| dir.join(f).is_file()))
}

impl Embedder {
    pub fn new(bundled: Option<PathBuf>) -> Self {
        Self {
            model: None,
            bundled,
        }
    }

    /// Loads the bundled model, or downloads the official one into the local
    /// cache when this checkout has no bundled copy.
    pub fn init(&mut self) -> Result<(), String> {
        if self.model.is_some() {
            return Ok(());
        }
        if let Some(dir) = self.bundled.clone() {
            match load_bundled(&dir) {
                Ok(model) => {
                    self.model = Some(model);
                    return Ok(());
                }
                // A corrupt or partially copied resource should not stop search:
                // fall through to the cache rather than failing the scan.
                Err(e) => crate::log_warn(&format!("bundled model failed ({e}); using cache")),
            }
        }
        let opts = TextInitOptions::new(EmbeddingModel::EmbeddingGemma300MQ4)
            .with_show_download_progress(true)
            .with_cache_dir(cache_dir());
        let model = TextEmbedding::try_new(opts).map_err(|e| format!("embedding model: {e}"))?;
        self.model = Some(model);
        Ok(())
    }

    pub fn ready(&self) -> bool {
        self.model.is_some()
    }

    pub fn embed(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>, String> {
        self.init()?;
        let m = self.model.as_mut().unwrap();
        m.embed(texts.to_vec(), None)
            .map_err(|e| format!("embed: {e}"))
    }
}

fn load_bundled(dir: &Path) -> Result<TextEmbedding, String> {
    let read = |name: &str| {
        std::fs::read(dir.join(name)).map_err(|e| format!("read {name}: {e}"))
    };
    // The 4-bit weights live in a separate ONNX initializer file. fastembed
    // hands it to ONNX Runtime in memory, so nothing needs to be written beside
    // the installed executable (which may be read-only).
    let mut bundled = UserDefinedEmbeddingModel::new(read("model_q4.onnx")?, TokenizerFiles {
        tokenizer_file: read("tokenizer.json")?,
        config_file: read("config.json")?,
        special_tokens_map_file: read("special_tokens_map.json")?,
        tokenizer_config_file: read("tokenizer_config.json")?,
    })
    .with_pooling(Pooling::Mean)
    .with_quantization(QuantizationMode::None)
    .with_external_initializer("model_q4.onnx_data".into(), read("model_q4.onnx_data")?);
    // No builder method for this field in fastembed 5.17; the default precedence
    // already finds it, but naming it keeps the contract explicit.
    bundled.output_key = Some(OutputKey::ByName("sentence_embedding".into()));

    TextEmbedding::try_new_from_user_defined(bundled, InitOptionsUserDefined::new())
        .map_err(|e| format!("bundled embedding model: {e}"))
}

/// Cosine similarity small helper; the index is tiny (a folder of files),
/// so a linear scan is the honest choice.
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    let (mut dot, mut na, mut nb) = (0.0_f32, 0.0_f32, 0.0_f32);
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na.sqrt() * nb.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_identical_vectors_is_one() {
        let v = vec![1.0, 2.0, 3.0];
        assert!((cosine(&v, &v) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_orthogonal_vectors_is_zero() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine(&a, &b)).abs() < 1e-5);
    }

    #[test]
    fn bundled_model_dir_locates_files() {
        let Some(dir) = bundled_model_dir(None) else {
            eprintln!("[draku test] bundled model not downloaded; skipping bundled_model_dir_locates_files");
            return;
        };
        for f in MODEL_FILES {
            assert!(dir.join(f).is_file(), "file {f} must exist in {dir:?}");
        }
    }

    #[test]
    fn bundled_model_loads_and_embeds() {
        let Some(dir) = bundled_model_dir(None) else {
            eprintln!("[draku test] bundled model not downloaded; skipping bundled_model_loads_and_embeds");
            return;
        };
        let mut embedder = Embedder::new(Some(dir));
        let texts = vec![
            "title: Wi-Fi password | text: Wi-Fi password tiger-lily-0429 router admin".into(),
            "title: Dr Patel blood test | text: Complete blood count lab report prescription".into(),
        ];
        let embeddings = embedder.embed(&texts).expect("embed must succeed");
        assert_eq!(embeddings.len(), 2);
        assert!(!embeddings[0].is_empty());
        assert_eq!(embeddings[0].len(), embeddings[1].len());

        let q = embedder
            .embed(&["task: search result | query: internet password".into()])
            .expect("query embed");
        let sim_wifi = cosine(&q[0], &embeddings[0]);
        let sim_blood = cosine(&q[0], &embeddings[1]);
        assert!(
            sim_wifi > sim_blood,
            "internet password should match wifi ({sim_wifi}) higher than blood test ({sim_blood})"
        );
    }
}

