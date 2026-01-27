use crate::AssetLoader;
use skia_safe::textlayout::{FontCollection, TypefaceFontProvider};
use skia_safe::{Data, Image, RuntimeEffect};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Manages heavy shared resources (Fonts, Shaders, Asset Loading).
///
/// This struct is extracted from the `Director` to allow passing asset context
/// to the renderer and other systems without mutably borrowing the entire `Director`
/// (which would cause borrow checker conflicts with the Scene Graph).
#[derive(Clone)]
pub struct AssetManager {
    /// Asset loader for resolving file paths to bytes.
    pub loader: Arc<dyn AssetLoader>,
    /// Global shader cache.
    pub shader_cache: Arc<Mutex<HashMap<String, RuntimeEffect>>>,
    /// Shared Font Collection (Skia).
    pub font_collection: Arc<Mutex<FontCollection>>,
    /// Shared Font Provider (Skia).
    pub font_provider: Arc<Mutex<TypefaceFontProvider>>,
    /// Image Cache
    pub image_cache: Arc<Mutex<HashMap<String, Image>>>,
    /// Blob/Metadata Cache (e.g. Lottie JSON, raw bytes)
    pub blob_cache: Arc<Mutex<HashMap<String, Arc<Vec<u8>>>>>,
}

impl AssetManager {
    /// Creates a new `AssetManager`.
    pub fn new(
        loader: Arc<dyn AssetLoader>,
        font_collection: Arc<Mutex<FontCollection>>,
        font_provider: Arc<Mutex<TypefaceFontProvider>>,
        shader_cache: Arc<Mutex<HashMap<String, RuntimeEffect>>>,
    ) -> Self {
        Self {
            loader,
            shader_cache,
            font_collection,
            font_provider,
            image_cache: Arc::new(Mutex::new(HashMap::new())),
            blob_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Loads an image from the given path, caching the result.
    pub fn load_image(&self, path: &str) -> Option<Image> {
        // 1. Check Cache
        {
            let cache = self.image_cache.lock().unwrap();
            if let Some(img) = cache.get(path) {
                return Some(img.clone());
            }
        }

        // 2. Load Bytes
        let bytes = self.loader.load_bytes(path).ok()?;

        // 3. Decode
        let data = Data::new_copy(&bytes);
        if let Some(img) = Image::from_encoded(data) {
            // 4. Cache
            let mut cache = self.image_cache.lock().unwrap();
            cache.insert(path.to_string(), img.clone());
            Some(img)
        } else {
            None
        }
    }

    /// Loads a raw blob (bytes) from the given path, caching the result.
    pub fn load_blob(&self, path: &str) -> anyhow::Result<Arc<Vec<u8>>> {
        // 1. Check Cache
        {
            let cache = self.blob_cache.lock().unwrap();
            if let Some(blob) = cache.get(path) {
                return Ok(blob.clone());
            }
        }

        // 2. Load Bytes
        let bytes = self.loader.load_bytes(path)?;
        let blob = Arc::new(bytes);

        // 3. Cache
        let mut cache = self.blob_cache.lock().unwrap();
        cache.insert(path.to_string(), blob.clone());

        Ok(blob)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockLoader {
        calls: AtomicUsize,
    }

    impl AssetLoader for MockLoader {
        fn load_bytes(&self, _path: &str) -> anyhow::Result<Vec<u8>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(vec![0u8; 10])
        }
    }

    #[test]
    fn test_blob_caching() {
        let loader = Arc::new(MockLoader {
            calls: AtomicUsize::new(0),
        });
        let am = AssetManager::new(
            loader.clone(),
            Arc::new(Mutex::new(FontCollection::new())),
            Arc::new(Mutex::new(TypefaceFontProvider::new())),
            Arc::new(Mutex::new(HashMap::new())),
        );

        let _ = am.load_blob("foo").unwrap();
        let _ = am.load_blob("foo").unwrap();

        let calls = loader.calls.load(Ordering::SeqCst);
        assert_eq!(calls, 1, "Should only load once for same path");

        let _ = am.load_blob("bar").unwrap();
        assert_eq!(
            loader.calls.load(Ordering::SeqCst),
            2,
            "Should load new path"
        );
    }
}
