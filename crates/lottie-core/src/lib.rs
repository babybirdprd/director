pub mod animatable;
#[cfg(feature = "expressions")]
pub mod expressions;
pub mod modifiers;
pub mod renderer;
pub mod text_path;

use animatable::Animator;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
#[cfg(feature = "expressions")]
use expressions::ExpressionEvaluator;
use glam::{Mat3, Mat4, Vec2, Vec3, Vec4};
use kurbo::{BezPath, Point, Shape as _};
use lottie_data::model::{self as data, LottieJson};
use modifiers::{
    GeometryModifier, OffsetPathModifier, PuckerBloatModifier, TwistModifier, WiggleModifier,
    ZigZagModifier,
};
pub use renderer::*;
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::sync::{Arc, Mutex, OnceLock};
use text_path::{TextPathLayoutOptions, TextPathRenderer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaSupport {
    Implemented,
    ParsedNoop,
    Missing,
}

pub const RENDERED_EFFECT_TYPES: &[u8] =
    &[5, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34];
pub const PARSED_NOOP_EFFECT_TYPES: &[u8] = &[];

pub fn effect_type_support(ty: u8) -> SchemaSupport {
    if RENDERED_EFFECT_TYPES.contains(&ty) {
        SchemaSupport::Implemented
    } else if PARSED_NOOP_EFFECT_TYPES.contains(&ty) {
        SchemaSupport::ParsedNoop
    } else {
        SchemaSupport::Missing
    }
}

pub fn layer_type_support(ty: u8) -> SchemaSupport {
    match ty {
        0 | 1 | 2 | 3 | 4 | 5 | 6 | 13 | 15 => SchemaSupport::Implemented,
        _ => SchemaSupport::Missing,
    }
}

pub fn asset_kind_support(kind: &str) -> SchemaSupport {
    match kind {
        "precomposition" | "image" | "data-source" | "sound" => SchemaSupport::Implemented,
        _ => SchemaSupport::Missing,
    }
}

fn log_unsupported_effect_once(ty: u8, name: Option<&str>) {
    static UNSUPPORTED_EFFECTS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    let key = format!("{ty}:{}", name.unwrap_or_default());
    let store = UNSUPPORTED_EFFECTS.get_or_init(|| Mutex::new(HashSet::new()));
    if let Ok(mut seen) = store.lock() {
        if seen.insert(key) {
            eprintln!(
                "[lottie-core] Unsupported effect type {ty} ({}) parsed as no-op",
                name.unwrap_or("unnamed")
            );
        }
    }
}

fn log_unsupported_layer_once(ty: u8, name: Option<&str>) {
    static UNSUPPORTED_LAYERS: OnceLock<Mutex<HashSet<u8>>> = OnceLock::new();
    let store = UNSUPPORTED_LAYERS.get_or_init(|| Mutex::new(HashSet::new()));
    if let Ok(mut seen) = store.lock() {
        if seen.insert(ty) {
            eprintln!(
                "[lottie-core] Layer type {ty} ({}) is parsed but not rendered",
                name.unwrap_or("unnamed")
            );
        }
    }
}

fn parse_color_channel(raw: f32) -> ColorChannel {
    match raw.round() as i32 {
        1 => ColorChannel::R,
        2 => ColorChannel::G,
        3 => ColorChannel::B,
        4 => ColorChannel::A,
        _ => ColorChannel::A,
    }
}

#[derive(Clone)]
struct PendingGeometry {
    kind: GeometryKind,
    transform: Mat3,
}

#[derive(Clone)]
enum GeometryKind {
    Path(BezPath),
    Rect { size: Vec2, pos: Vec2, radius: f32 },
    Polystar(PolystarParams),
    Ellipse { size: Vec2, pos: Vec2 },
    Merge(Vec<PendingGeometry>, MergeMode),
}

impl PendingGeometry {
    fn to_shape_geometry(&self, builder: &SceneGraphBuilder) -> ShapeGeometry {
        match &self.kind {
            GeometryKind::Merge(geoms, mode) => {
                let shapes = geoms.iter().map(|g| g.to_shape_geometry(builder)).collect();
                ShapeGeometry::Boolean {
                    mode: *mode,
                    shapes,
                }
            }
            _ => ShapeGeometry::Path(self.to_path(builder)),
        }
    }

    fn to_path(&self, builder: &SceneGraphBuilder) -> BezPath {
        let mut path = match &self.kind {
            GeometryKind::Path(p) => p.clone(),
            GeometryKind::Merge(geoms, _) => {
                let mut p = BezPath::new();
                for g in geoms {
                    p.extend(g.to_path(builder));
                }
                p
            }
            GeometryKind::Rect { size, pos, radius } => {
                let half = *size / 2.0;
                let rect = kurbo::Rect::new(
                    (pos.x - half.x) as f64,
                    (pos.y - half.y) as f64,
                    (pos.x + half.x) as f64,
                    (pos.y + half.y) as f64,
                );
                if *radius > 0.0 {
                    rect.to_rounded_rect(*radius as f64).to_path(0.1)
                } else {
                    rect.to_path(0.1)
                }
            }
            GeometryKind::Ellipse { size, pos } => {
                let half = *size / 2.0;
                let ellipse = kurbo::Ellipse::new(
                    (pos.x as f64, pos.y as f64),
                    (half.x as f64, half.y as f64),
                    0.0,
                );
                ellipse.to_path(0.1)
            }
            GeometryKind::Polystar(params) => builder.generate_polystar_path(params),
        };

        let m = self.transform.to_cols_array();
        let affine = kurbo::Affine::new([
            m[0] as f64,
            m[1] as f64,
            m[3] as f64,
            m[4] as f64,
            m[6] as f64,
            m[7] as f64,
        ]);
        path.apply_affine(affine);
        path
    }
}

#[derive(Clone, Copy)]
struct PolystarParams {
    pos: Vec2,
    outer_radius: f32,
    inner_radius: f32,
    outer_roundness: f32,
    inner_roundness: f32,
    rotation: f32,
    points: f32,
    kind: u8,           // 1=star, 2=polygon
    corner_radius: f32, // From RoundCorners modifier
}

pub enum ImageSource {
    Data(Vec<u8>), // Encoded bytes (PNG/JPG)
}

pub trait TextMeasurer: Send + Sync {
    /// Returns the width of the text string for the given font and size.
    fn measure(&self, text: &str, font_family: &str, size: f32) -> f32;
}

fn load_asset_bytes(asset: &data::Asset) -> Option<Vec<u8>> {
    if let Some(p) = &asset.p {
        if p.starts_with("data:") && p.contains(";base64,") {
            let split: Vec<&str> = p.splitn(2, ',').collect();
            if split.len() > 1 {
                match BASE64_STANDARD.decode(split[1]) {
                    Ok(bytes) => Some(bytes),
                    Err(_) => None,
                }
            } else {
                None
            }
        } else {
            let preferred = if let Some(u) = &asset.u {
                if u.is_empty() {
                    p.clone()
                } else {
                    format!("{u}{p}")
                }
            } else {
                p.clone()
            };
            if let Ok(bytes) = std::fs::read(&preferred) {
                Some(bytes)
            } else if let Ok(bytes) = std::fs::read(p) {
                Some(bytes)
            } else {
                None
            }
        }
    } else {
        None
    }
}

fn asset_path(asset: &data::Asset) -> Option<String> {
    let p = asset.p.as_ref()?;
    if p.starts_with("data:") {
        return Some(p.clone());
    }
    if let Some(u) = &asset.u {
        if !u.is_empty() {
            return Some(format!("{u}{p}"));
        }
    }
    Some(p.clone())
}

fn is_sound_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".wav")
        || lower.ends_with(".mp3")
        || lower.ends_with(".ogg")
        || lower.ends_with(".aac")
        || lower.ends_with(".m4a")
        || lower.ends_with(".flac")
}

fn is_data_path(path: &str) -> bool {
    path.to_ascii_lowercase().ends_with(".json")
}

fn asset_type_code(asset: &data::Asset) -> Option<i64> {
    asset.t.as_ref().and_then(|v| match v {
        serde_json::Value::Number(n) => n.as_i64(),
        serde_json::Value::String(s) => s.parse::<i64>().ok(),
        _ => None,
    })
}

fn collect_nonvisual_ref_ids<'a>(
    layers: &'a [data::Layer],
    asset_by_id: &HashMap<String, &'a data::Asset>,
    audio_ref_ids: &mut HashSet<String>,
    data_ref_ids: &mut HashSet<String>,
) {
    for layer in layers {
        match layer.ty {
            6 => {
                if let Some(ref_id) = &layer.ref_id {
                    audio_ref_ids.insert(ref_id.clone());
                }
            }
            15 => {
                if let Some(ref_id) = &layer.ref_id {
                    data_ref_ids.insert(ref_id.clone());
                }
            }
            _ => {}
        }

        if let Some(ref_id) = &layer.ref_id {
            if let Some(asset) = asset_by_id.get(ref_id) {
                if let Some(sub_layers) = &asset.layers {
                    collect_nonvisual_ref_ids(sub_layers, asset_by_id, audio_ref_ids, data_ref_ids);
                }
            }
        }
    }
}

/// Immutable, shared assets for a Lottie animation.
pub struct LottieAsset {
    pub model: LottieJson,
    pub width: f32,
    pub height: f32,
    pub duration_frames: f32,
    pub _frame_rate: f32,
    pub assets: HashMap<String, ImageSource>,
    pub data_sources: HashMap<String, RuntimeDataSource>,
    pub sound_assets: HashMap<String, RuntimeSoundAsset>,
    pub text_measurer: Option<Box<dyn TextMeasurer>>,
}

impl LottieAsset {
    pub fn from_model(model: LottieJson) -> Self {
        let width = model.w as f32;
        let height = model.h as f32;
        let frame_rate = model.fr;
        let duration_frames = model.op - model.ip;

        let mut assets = HashMap::new();
        let mut data_sources = HashMap::new();
        let mut sound_assets = HashMap::new();

        let mut asset_by_id = HashMap::new();
        for asset in &model.assets {
            asset_by_id.insert(asset.id.clone(), asset);
        }

        let mut audio_ref_ids = HashSet::new();
        let mut data_ref_ids = HashSet::new();
        collect_nonvisual_ref_ids(
            &model.layers,
            &asset_by_id,
            &mut audio_ref_ids,
            &mut data_ref_ids,
        );

        for asset in &model.assets {
            let path = asset_path(asset);
            let bytes = load_asset_bytes(asset);
            let is_data_source = data_ref_ids.contains(&asset.id)
                || asset_type_code(asset) == Some(3)
                || path.as_deref().is_some_and(is_data_path);
            let is_sound = audio_ref_ids.contains(&asset.id)
                || path.as_deref().is_some_and(is_sound_path);

            if is_data_source {
                let payload = bytes
                    .as_ref()
                    .and_then(|raw| serde_json::from_slice::<serde_json::Value>(raw).ok());
                data_sources.insert(
                    asset.id.clone(),
                    RuntimeDataSource {
                        id: asset.id.clone(),
                        path,
                        payload,
                    },
                );
                continue;
            }

            if is_sound {
                sound_assets.insert(
                    asset.id.clone(),
                    RuntimeSoundAsset {
                        id: asset.id.clone(),
                        path,
                        byte_len: bytes.as_ref().map_or(0, std::vec::Vec::len),
                    },
                );
                continue;
            }

            if let Some(data) = bytes {
                assets.insert(asset.id.clone(), ImageSource::Data(data));
            }
        }

        Self {
            model,
            width,
            height,
            duration_frames,
            _frame_rate: frame_rate,
            assets,
            data_sources,
            sound_assets,
            text_measurer: None,
        }
    }

    pub fn set_text_measurer(&mut self, measurer: Box<dyn TextMeasurer>) {
        self.text_measurer = Some(measurer);
    }

    pub fn set_asset(&mut self, id: String, data: Vec<u8>) {
        self.assets.insert(id, ImageSource::Data(data));
    }
}

pub struct LottiePlayer {
    pub asset: Option<Arc<LottieAsset>>,
    pub current_frame: f32,
    #[cfg(feature = "expressions")]
    pub expression_evaluator: Option<ExpressionEvaluator>,
}

impl LottiePlayer {
    pub fn new() -> Self {
        #[cfg(feature = "expressions")]
        let expression_evaluator = Some(ExpressionEvaluator::new());
        Self {
            asset: None,
            current_frame: 0.0,
            #[cfg(feature = "expressions")]
            expression_evaluator,
        }
    }

    pub fn load(&mut self, asset: Arc<LottieAsset>) {
        self.current_frame = asset.model.ip; // Start at in-point
        self.asset = Some(asset);
    }

    // Legacy load for convenience (creates new Asset wrapper)
    pub fn load_json(&mut self, data: LottieJson) {
        let asset = Arc::new(LottieAsset::from_model(data));
        self.load(asset);
    }

    pub fn advance(&mut self, dt: f32) {
        if let Some(asset) = &self.asset {
            // dt is in seconds
            let frames = dt * asset._frame_rate;
            self.current_frame += frames;

            // Loop
            if self.current_frame >= asset.model.op {
                let duration = asset.model.op - asset.model.ip;
                self.current_frame =
                    asset.model.ip + (self.current_frame - asset.model.op) % duration;
            }
        }
    }

    pub fn render_tree(&mut self) -> RenderTree {
        if let Some(asset) = &self.asset {
            #[cfg(feature = "expressions")]
            let evaluator = self.expression_evaluator.as_mut();
            #[cfg(not(feature = "expressions"))]
            let evaluator: Option<&mut ()> = None;

            let mut builder = SceneGraphBuilder::new(asset, self.current_frame);
            builder.build(evaluator)
        } else {
            // Return empty tree
            RenderTree::mock_sample()
        }
    }
}

struct SceneGraphBuilder<'a> {
    asset: &'a LottieAsset,
    frame: f32,
    model_assets_map: HashMap<String, &'a data::Asset>,
    audio_events: Vec<RuntimeAudioEvent>,
    data_bindings: Vec<RuntimeDataBinding>,
}

impl<'a> SceneGraphBuilder<'a> {
    fn new(asset: &'a LottieAsset, frame: f32) -> Self {
        let mut model_assets_map = HashMap::new();
        for a in &asset.model.assets {
            model_assets_map.insert(a.id.clone(), a);
        }
        Self {
            asset,
            frame,
            model_assets_map,
            audio_events: Vec::new(),
            data_bindings: Vec::new(),
        }
    }

    fn build(
        &mut self,
        #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>,
    ) -> RenderTree {
        let mut layer_map = HashMap::new();
        for layer in &self.asset.model.layers {
            if let Some(ind) = layer.ind {
                layer_map.insert(ind, layer);
            }
        }

        let (view_matrix, projection_matrix) = self.get_camera_matrices(
            &self.asset.model.layers,
            &layer_map,
            evaluator.as_deref_mut(),
        );

        let root_node = self.build_composition(&self.asset.model.layers, &layer_map, evaluator);
        let mut data_sources: Vec<_> = self.asset.data_sources.values().cloned().collect();
        let mut sound_assets: Vec<_> = self.asset.sound_assets.values().cloned().collect();
        data_sources.sort_by(|a, b| a.id.cmp(&b.id));
        sound_assets.sort_by(|a, b| a.id.cmp(&b.id));

        RenderTree {
            width: self.asset.width,
            height: self.asset.height,
            root: root_node,
            view_matrix,
            projection_matrix,
            audio_events: std::mem::take(&mut self.audio_events),
            data_bindings: std::mem::take(&mut self.data_bindings),
            data_sources,
            sound_assets,
        }
    }

    fn get_camera_matrices(
        &self,
        layers: &'a [data::Layer],
        map: &HashMap<u32, &'a data::Layer>,
        #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>,
    ) -> (Mat4, Mat4) {
        // Step 1: Find Active Camera (Top-most, ty=13, visible)
        let mut camera_layer = None;
        for layer in layers {
            if layer.ty == 13 {
                // Check visibility
                if self.frame >= layer.ip && self.frame < layer.op {
                    camera_layer = Some(layer);
                    break; // Top-most found
                }
            }
        }

        if let Some(cam) = camera_layer {
            // Step 2: Compute View Matrix
            let cam_transform = self.resolve_transform(cam, map, evaluator.as_deref_mut());
            let view_matrix = cam_transform.inverse();

            // Step 3: Compute Projection Matrix
            let pe = if let Some(prop) = &cam.pe {
                Animator::resolve_simple(
                    prop,
                    self.frame - cam.st,
                    |v| *v,
                    0.0,
                    evaluator,
                    self.asset._frame_rate,
                )
            } else {
                0.0
            };

            let perspective = if pe > 0.0 { pe } else { 1000.0 }; // Default ?

            // FOV Calculation
            // pe is distance. Height is comp height.
            // tan(fov/2) = (height/2) / pe
            // fov = 2 * atan(height / (2 * pe))
            let fov = 2.0 * (self.asset.height / (2.0 * perspective)).atan();

            let aspect = self.asset.width / self.asset.height;
            let near = 0.1;
            let far = 10000.0;

            let projection_matrix = Mat4::perspective_rh(fov, aspect, near, far);

            (view_matrix, projection_matrix)
        } else {
            // Default 2D View
            (Mat4::IDENTITY, Mat4::IDENTITY)
        }
    }

    fn build_composition(
        &mut self,
        layers: &'a [data::Layer],
        layer_map: &HashMap<u32, &'a data::Layer>,
        #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>,
    ) -> RenderNode {
        let mut nodes = Vec::new();
        let mut consumed_indices = HashSet::new();
        let len = layers.len();

        for i in (0..len).rev() {
            if consumed_indices.contains(&i) {
                continue;
            }

            let layer = &layers[i];

            if let Some(tt) = layer.tt {
                if i > 0 {
                    let matte_idx = i - 1;
                    if !consumed_indices.contains(&matte_idx) {
                        consumed_indices.insert(matte_idx);
                        let matte_layer = &layers[matte_idx];

                        if let Some(mut content_node) =
                            self.process_layer(layer, layer_map, evaluator.as_deref_mut())
                        {
                            if let Some(matte_node) =
                                self.process_layer(matte_layer, layer_map, evaluator.as_deref_mut())
                            {
                                let mode = match tt {
                                    1 => MatteMode::Alpha,
                                    2 => MatteMode::AlphaInverted,
                                    3 => MatteMode::Luma,
                                    4 => MatteMode::LumaInverted,
                                    _ => MatteMode::Alpha,
                                };
                                content_node.matte = Some(Box::new(Matte {
                                    mode,
                                    node: matte_node,
                                }));
                            }
                            nodes.push(content_node);
                        }
                        continue;
                    }
                }
            }

            if let Some(node) = self.process_layer(layer, layer_map, evaluator.as_deref_mut()) {
                nodes.push(node);
            }
        }

        RenderNode {
            transform: Mat4::IDENTITY,
            alpha: 1.0,
            blend_mode: BlendMode::Normal,
            content: NodeContent::Group(nodes),
            masks: vec![],
            styles: vec![],
            matte: None,
            effects: vec![],
            is_adjustment_layer: false,
        }
    }

    #[inline]
    fn layer_time(&self, _layer: &data::Layer) -> f32 {
        // Layer property keyframes are authored in composition time.
        self.frame
    }

    #[inline]
    fn layer_source_time(&self, layer: &data::Layer) -> f32 {
        // Precomp source playback is offset by layer start time.
        self.frame - layer.st
    }

    #[inline]
    fn mat3_to_mat4_2d(m: Mat3) -> Mat4 {
        let c = m.to_cols_array();
        Mat4::from_cols(
            Vec4::new(c[0], c[1], 0.0, c[2]),
            Vec4::new(c[3], c[4], 0.0, c[5]),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(c[6], c[7], 0.0, c[8]),
        )
    }

    fn process_layer(
        &mut self,
        layer: &'a data::Layer,
        layer_map: &HashMap<u32, &'a data::Layer>,
        #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>,
    ) -> Option<RenderNode> {
        // Check if layer is hidden
        if layer.hd == Some(true) {
            return None;
        }

        let _is_adjustment_layer = layer.ao == Some(1);

        if self.frame < layer.ip || self.frame >= layer.op {
            return None;
        }

        if layer.ty == 6 {
            self.capture_audio_layer(layer);
            return None;
        }
        if layer.ty == 15 {
            self.capture_data_layer(layer);
            return None;
        }

        if !matches!(layer.ty, 0 | 1 | 2 | 3 | 4 | 5 | 6 | 13 | 15) {
            log_unsupported_layer_once(layer.ty, layer.nm.as_deref());
        }

        let transform = self.resolve_transform(layer, layer_map, evaluator.as_deref_mut());

        let opacity = Animator::resolve_simple(
            &layer.ks.o,
            self.layer_time(layer),
            |v| *v / 100.0,
            1.0,
            evaluator.as_deref_mut(),
            self.asset._frame_rate,
        );

        let content = if let Some(shapes) = &layer.shapes {
            let layer_blend_mode = self.resolve_blend_mode(layer.bm);
            let shape_nodes = self.process_shapes(
                shapes,
                self.frame,
                evaluator.as_deref_mut(),
                None,
                layer_blend_mode,
            );
            NodeContent::Group(shape_nodes)
        } else if let Some(text_data) = &layer.t {
            // Text Layer
            let doc = Animator::resolve_simple(
                &text_data.d,
                self.layer_time(layer),
                |v| v.clone(),
                data::TextDocument::default(),
                evaluator.as_deref_mut(),
                self.asset._frame_rate,
            );

            let base_fill_color = Vec4::new(doc.fc[0], doc.fc[1], doc.fc[2], 1.0);
            let base_stroke_color = if let Some(sc) = &doc.sc {
                Some(Vec4::new(sc[0], sc[1], sc[2], 1.0))
            } else {
                None
            };

            let chars: Vec<char> = doc.t.chars().collect();
            let char_count = chars.len();

            let mut glyphs = Vec::with_capacity(char_count);

            for &c in &chars {
                let g = RenderGlyph {
                    character: c,
                    pos: Vec3::ZERO,
                    scale: Vec3::ONE,
                    rotation: Vec3::ZERO,
                    tracking: 0.0,
                    alpha: 1.0,
                    fill: Some(Fill {
                        paint: Paint::Solid(base_fill_color),
                        opacity: 1.0,
                        rule: FillRule::NonZero,
                    }),
                    stroke: if let Some(col) = base_stroke_color {
                        Some(Stroke {
                            paint: Paint::Solid(col),
                            width: doc.sw.unwrap_or(1.0),
                            opacity: 1.0,
                            cap: LineCap::Round,
                            join: LineJoin::Round,
                            miter_limit: None,
                            dash: None,
                        })
                    } else {
                        None
                    },
                };
                glyphs.push(g);
            }

            if let Some(animators) = &text_data.a {
                for animator in animators {
                    let sel = &animator.s;
                    let start_val = Animator::resolve_simple(
                        sel.s.as_ref().unwrap_or(&data::Property::default()),
                        self.layer_time(layer),
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let end_val = Animator::resolve_simple(
                        sel.e.as_ref().unwrap_or(&data::Property::default()),
                        self.layer_time(layer),
                        |v| *v,
                        100.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let offset_val = Animator::resolve_simple(
                        sel.o.as_ref().unwrap_or(&data::Property::default()),
                        self.layer_time(layer),
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );

                    let start_idx = char_count as f32 * start_val / 100.0;
                    let end_idx = char_count as f32 * end_val / 100.0;
                    let offset_idx = char_count as f32 * offset_val / 100.0;

                    let style = &animator.a;
                    let p_delta = Animator::resolve_simple(
                        style.p.as_ref().unwrap_or(&data::Property::default()),
                        self.layer_time(layer),
                        |v| Vec3::from(v.0),
                        Vec3::ZERO,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let s_val = Animator::resolve_simple(
                        style.s.as_ref().unwrap_or(&data::Property::default()),
                        self.layer_time(layer),
                        |v| Vec3::from(v.0) / 100.0,
                        Vec3::ONE,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let o_val = Animator::resolve_simple(
                        style.o.as_ref().unwrap_or(&data::Property::default()),
                        self.layer_time(layer),
                        |v| *v,
                        100.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    // RZ
                    let r_val = Animator::resolve_simple(
                        style.r.as_ref().unwrap_or(&data::Property::default()),
                        self.layer_time(layer),
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );

                    // Tracking
                    let t_val = Animator::resolve_simple(
                        style.t.as_ref().unwrap_or(&data::Property::default()),
                        self.layer_time(layer),
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );

                    let fc_val = if let Some(fc_prop) = &style.fc {
                        Some(Animator::resolve_simple(
                            fc_prop,
                            self.layer_time(layer),
                            |v| Vec4::from_slice(v),
                            Vec4::ONE,
                            evaluator.as_deref_mut(),
                            self.asset._frame_rate,
                        ))
                    } else {
                        None
                    };

                    let sc_val = if let Some(sc_prop) = &style.sc {
                        Some(Animator::resolve_simple(
                            sc_prop,
                            self.layer_time(layer),
                            |v| Vec4::from_slice(v),
                            Vec4::ONE,
                            evaluator.as_deref_mut(),
                            self.asset._frame_rate,
                        ))
                    } else {
                        None
                    };

                    for (i, glyph) in glyphs.iter_mut().enumerate() {
                        let idx = i as f32;
                        let effective_start = start_idx + offset_idx;
                        let effective_end = end_idx + offset_idx;

                        let overlap_start = idx.max(effective_start);
                        let overlap_end = (idx + 1.0).min(effective_end);

                        let factor = (overlap_end - overlap_start).max(0.0).min(1.0);

                        if factor > 0.0 {
                            glyph.pos += p_delta * factor;

                            // Scale mixing
                            let scale_factor = Vec3::ONE + (s_val - Vec3::ONE) * factor;
                            glyph.scale *= scale_factor;

                            // Rotation (RZ only for now, mapped to Z component)
                            glyph.rotation.z += r_val.to_radians() * factor;

                            glyph.tracking += t_val * factor;

                            let target_alpha = o_val / 100.0;
                            let alpha_mult = 1.0 + (target_alpha - 1.0) * factor;
                            glyph.alpha *= alpha_mult;

                            if let Some(target_color) = fc_val {
                                if let Some(fill) = &mut glyph.fill {
                                    if let Paint::Solid(current_color) = &mut fill.paint {
                                        *current_color = current_color.lerp(target_color, factor);
                                    }
                                }
                            }

                            if let Some(target_color) = sc_val {
                                if let Some(stroke) = &mut glyph.stroke {
                                    if let Paint::Solid(current_color) = &mut stroke.paint {
                                        *current_color = current_color.lerp(target_color, factor);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Layout
            if let Some(measurer) = self.asset.text_measurer.as_deref() {
                let box_size = doc.sz.map(|v| Vec2::from_slice(&v));
                let box_pos = doc.ps.map(|v| Vec2::from_slice(&v)).unwrap_or(Vec2::ZERO);
                let tracking_val = doc.tr;

                // Check for text on path first
                if let Some(ref path_data) = text_data.path {
                    // Text on Path layout
                    if let Some(ref masks) = layer.masks_properties {
                        if let Some(mask_idx) = path_data.mask_index {
                            if let Some(mask) = masks.get(mask_idx as usize) {
                                // Evaluate mask path at current frame
                                let path = Animator::resolve_simple(
                                    &mask.pt,
                                    self.layer_time(layer),
                                    |v| v.clone(),
                                    data::BezierPath::default(),
                                    evaluator.as_deref_mut(),
                                    self.asset._frame_rate,
                                );

                                // Convert BezierPath to kurbo::BezPath
                                let bez_path = self.convert_bezier_path(&path);

                                // Evaluate margin properties
                                let first_margin = path_data
                                    .first_margin
                                    .as_ref()
                                    .map(|p| {
                                        Animator::resolve_simple(
                                            p,
                                            self.layer_time(layer),
                                            |v| *v,
                                            0.0,
                                            evaluator.as_deref_mut(),
                                            self.asset._frame_rate,
                                        )
                                    })
                                    .unwrap_or(0.0);

                                let last_margin = path_data
                                    .last_margin
                                    .as_ref()
                                    .map(|p| {
                                        Animator::resolve_simple(
                                            p,
                                            self.layer_time(layer),
                                            |v| *v,
                                            0.0,
                                            evaluator.as_deref_mut(),
                                            self.asset._frame_rate,
                                        )
                                    })
                                    .unwrap_or(0.0);

                                // Calculate glyph widths
                                let mut glyph_widths = Vec::new();
                                for g in &glyphs {
                                    if g.character != '\n' {
                                        let w = measurer.measure(
                                            &g.character.to_string(),
                                            &doc.f,
                                            doc.s,
                                        );
                                        glyph_widths.push(w + tracking_val + g.tracking);
                                    }
                                }

                                // Build layout options
                                let options = TextPathLayoutOptions {
                                    mask_index: mask_idx as usize,
                                    first_margin,
                                    last_margin,
                                    force_alignment: path_data.force_alignment.unwrap_or(false),
                                    perpendicular: path_data.perpendicular.unwrap_or(false),
                                    reverse: path_data.reversed.unwrap_or(false),
                                    justify: doc.j,
                                    tracking: tracking_val,
                                };

                                // Layout glyphs on path
                                let layout = TextPathRenderer::layout_text_on_path(
                                    &bez_path,
                                    &glyph_widths,
                                    &options,
                                );

                                // Apply layout to glyphs
                                let mut layout_idx = 0;
                                for (i, g) in glyphs.iter_mut().enumerate() {
                                    if g.character != '\n' && layout_idx < layout.positions.len() {
                                        g.pos = Vec3::new(
                                            layout.positions[layout_idx].x,
                                            layout.positions[layout_idx].y,
                                            0.0,
                                        );
                                        // Apply rotation from path tangent
                                        g.rotation.z = layout.rotations[layout_idx];
                                        // Apply scale from force alignment
                                        g.scale = Vec3::new(
                                            layout.scales[layout_idx].x,
                                            layout.scales[layout_idx].y,
                                            1.0,
                                        );
                                        layout_idx += 1;
                                    }
                                }
                            }
                        }
                    }
                } else if let Some(sz) = box_size {
                    // Box Text
                    let box_width = sz.x;
                    let mut lines: Vec<Vec<usize>> = Vec::new();
                    let mut current_line: Vec<usize> = Vec::new();
                    let mut current_line_width = 0.0;

                    let mut i = 0;
                    while i < glyphs.len() {
                        let start = i;
                        let mut end = i;
                        let mut word_width = 0.0;

                        while end < glyphs.len() {
                            let g = &glyphs[end];
                            let char_str = g.character.to_string();
                            let w = measurer.measure(&char_str, &doc.f, doc.s);
                            let advance = w + tracking_val + g.tracking;
                            word_width += advance;
                            let is_space = g.character == ' ';
                            let is_newline = g.character == '\n';
                            end += 1;
                            if is_space || is_newline {
                                break;
                            }
                        }

                        let is_newline = if end > 0 {
                            glyphs[end - 1].character == '\n'
                        } else {
                            false
                        };

                        if is_newline {
                            for k in start..end {
                                current_line.push(k);
                            }
                            lines.push(current_line);
                            current_line = Vec::new();
                            current_line_width = 0.0;
                        } else {
                            if !current_line.is_empty()
                                && current_line_width + word_width > box_width
                            {
                                lines.push(current_line);
                                current_line = Vec::new();
                                current_line_width = 0.0;
                            }
                            for k in start..end {
                                current_line.push(k);
                            }
                            current_line_width += word_width;
                        }
                        i = end;
                    }
                    if !current_line.is_empty() {
                        lines.push(current_line);
                    }

                    let mut current_y = box_pos.y;
                    for line_indices in lines {
                        let mut line_width = 0.0;
                        let mut advances = Vec::new();

                        for &idx in &line_indices {
                            let g = &glyphs[idx];
                            let w = measurer.measure(&g.character.to_string(), &doc.f, doc.s);
                            let advance = w + tracking_val + g.tracking;
                            advances.push(advance);
                            line_width += advance;
                        }

                        let align_width = line_width;
                        let start_x = match doc.j {
                            1 => box_width - align_width,
                            2 => (box_width - align_width) / 2.0,
                            _ => 0.0,
                        };

                        let mut x = box_pos.x + start_x;
                        for (k, &idx) in line_indices.iter().enumerate() {
                            let g = &mut glyphs[idx];
                            g.pos += Vec3::new(x, current_y, 0.0);
                            x += advances[k];
                        }
                        current_y += doc.lh;
                    }
                } else {
                    // Point Text
                    let mut current_y = 0.0;
                    let mut lines: Vec<Vec<usize>> = Vec::new();
                    let mut current_line = Vec::new();

                    for (i, g) in glyphs.iter().enumerate() {
                        if g.character == '\n' {
                            lines.push(current_line);
                            current_line = Vec::new();
                        } else {
                            current_line.push(i);
                        }
                    }
                    lines.push(current_line);

                    for line_indices in lines {
                        let mut line_width = 0.0;
                        let mut advances = Vec::new();
                        for &idx in &line_indices {
                            let g = &glyphs[idx];
                            let w = measurer.measure(&g.character.to_string(), &doc.f, doc.s);
                            let advance = w + tracking_val + g.tracking;
                            advances.push(advance);
                            line_width += advance;
                        }

                        let start_x = match doc.j {
                            1 => -line_width,
                            2 => -line_width / 2.0,
                            _ => 0.0,
                        };

                        let mut x = start_x;
                        for (k, &idx) in line_indices.iter().enumerate() {
                            let g = &mut glyphs[idx];
                            g.pos += Vec3::new(x, current_y, 0.0);
                            x += advances[k];
                        }
                        current_y += doc.lh;
                    }
                }
            } else {
                let fixed_width = 10.0;
                let mut x = 0.0;
                let mut y = 0.0;
                for g in &mut glyphs {
                    if g.character == '\n' {
                        x = 0.0;
                        y += doc.lh;
                    } else {
                        g.pos += Vec3::new(x, y, 0.0);
                        x += fixed_width + doc.tr + g.tracking;
                    }
                }
            }

            NodeContent::Text(Text {
                glyphs,
                font_family: doc.f,
                size: doc.s,
                justify: match doc.j {
                    1 => Justification::Right,
                    2 => Justification::Center,
                    _ => Justification::Left,
                },
                tracking: doc.tr,
                line_height: doc.lh,
            })
        } else if let Some(ref_id) = &layer.ref_id {
            if let Some(asset) = self.model_assets_map.get(ref_id) {
                if let Some(layers) = &asset.layers {
                    let local_frame = if let Some(tm_prop) = &layer.tm {
                        let tm_sec = Animator::resolve_simple(
                            tm_prop,
                            self.frame,
                            |v| *v,
                            0.0,
                            evaluator.as_deref_mut(),
                            self.asset._frame_rate,
                        );
                        tm_sec * self.asset._frame_rate
                    } else {
                        self.layer_source_time(layer)
                    };

                    let mut sub_layer_map = HashMap::new();
                    for l in layers {
                        if let Some(ind) = l.ind {
                            sub_layer_map.insert(ind, l);
                        }
                    }

                    let mut sub_builder = SceneGraphBuilder::new(self.asset, local_frame);
                    let root = sub_builder.build_composition(
                        layers,
                        &sub_layer_map,
                        evaluator.as_deref_mut(),
                    );

                    // Preserve layer properties from the precomp layer
                    let mut precomp_node = root;
                    precomp_node.transform =
                        self.resolve_transform(layer, layer_map, evaluator.as_deref_mut());
                    precomp_node.alpha = Animator::resolve_simple(
                        &layer.ks.o,
                        self.layer_time(layer),
                        |v| *v / 100.0,
                        1.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    precomp_node.blend_mode = self.resolve_blend_mode(layer.bm);
                    precomp_node.masks = if let Some(props) = &layer.masks_properties {
                        self.process_masks(props, self.layer_time(layer), evaluator.as_deref_mut())
                    } else {
                        vec![]
                    };
                    precomp_node.effects = self.process_effects(layer, evaluator.as_deref_mut());
                    precomp_node.styles =
                        self.process_layer_styles(layer, evaluator.as_deref_mut());
                    precomp_node.is_adjustment_layer = layer.ao == Some(1);

                    NodeContent::Group(vec![precomp_node])
                } else {
                    let data =
                        if let Some(ImageSource::Data(bytes)) = self.asset.assets.get(&asset.id) {
                            Some(bytes.clone())
                        } else {
                            None
                        };

                    NodeContent::Image(Image {
                        data,
                        width: asset.w.unwrap_or(100),
                        height: asset.h.unwrap_or(100),
                        id: Some(asset.id.clone()),
                    })
                }
            } else {
                NodeContent::Group(vec![])
            }
        } else if let Some(color) = &layer.color {
            let w = layer.sw.unwrap_or(100) as f64;
            let h = layer.sh.unwrap_or(100) as f64;
            let mut path = BezPath::new();
            path.move_to((0.0, 0.0));
            path.line_to((w, 0.0));
            path.line_to((w, h));
            path.line_to((0.0, h));
            path.close_path();

            let c_str = color.trim_start_matches('#');
            let r = u8::from_str_radix(&c_str[0..2], 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&c_str[2..4], 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&c_str[4..6], 16).unwrap_or(0) as f32 / 255.0;

            NodeContent::Shape(renderer::Shape {
                geometry: renderer::ShapeGeometry::Path(path),
                fill: Some(Fill {
                    paint: Paint::Solid(Vec4::new(r, g, b, 1.0)),
                    opacity: 1.0,
                    rule: FillRule::NonZero,
                }),
                stroke: None,
                trim: None,
            })
        } else {
            NodeContent::Group(vec![])
        };

        let masks = if let Some(props) = &layer.masks_properties {
            self.process_masks(props, self.layer_time(layer), evaluator.as_deref_mut())
        } else {
            vec![]
        };

        let effects = self.process_effects(layer, evaluator.as_deref_mut());
        let styles = self.process_layer_styles(layer, evaluator.as_deref_mut());

        Some(RenderNode {
            transform,
            alpha: opacity,
            blend_mode: self.resolve_blend_mode(layer.bm),
            content,
            masks,
            matte: None,
            effects,
            styles,
            is_adjustment_layer: false,
        })
    }

    fn resolve_property_numbers(
        &self,
        prop: &data::Property<serde_json::Value>,
        local_frame: f32,
    ) -> Vec<f32> {
        let pick_value = |value: &serde_json::Value| -> Vec<f32> {
            match value {
                serde_json::Value::Number(n) => n.as_f64().map(|v| vec![v as f32]).unwrap_or_default(),
                serde_json::Value::Array(arr) => arr
                    .iter()
                    .filter_map(|item| item.as_f64().map(|v| v as f32))
                    .collect(),
                _ => Vec::new(),
            }
        };

        match &prop.k {
            data::Value::Static(v) => pick_value(v),
            data::Value::Animated(keys) => {
                let mut selected = keys
                    .iter()
                    .find_map(|kf| kf.s.as_ref())
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                for key in keys {
                    if key.t <= local_frame {
                        if let Some(s) = &key.s {
                            selected = s.clone();
                        }
                    }
                }
                pick_value(&selected)
            }
            data::Value::Default => Vec::new(),
        }
    }

    fn capture_audio_layer(&mut self, layer: &'a data::Layer) {
        let mut level = Vec::new();
        if let Some(au) = &layer.au {
            if let Some(lv) = au.get("lv") {
                if let Ok(prop) = serde_json::from_value::<data::Property<serde_json::Value>>(lv.clone()) {
                    level = self.resolve_property_numbers(&prop, self.layer_time(layer));
                }
            }
        }
        if level.is_empty() {
            level.push(100.0);
        }

        let sound_path = layer
            .ref_id
            .as_ref()
            .and_then(|id| self.asset.sound_assets.get(id))
            .and_then(|sound| sound.path.clone());

        self.audio_events.push(RuntimeAudioEvent {
            layer_name: layer.nm.clone(),
            ref_id: layer.ref_id.clone(),
            sound_path,
            start_frame: layer.ip,
            end_frame: layer.op,
            stretch: layer.sr,
            level,
        });
    }

    fn capture_data_layer(&mut self, layer: &'a data::Layer) {
        let (source_path, payload) = if let Some(ref_id) = &layer.ref_id {
            if let Some(data_source) = self.asset.data_sources.get(ref_id) {
                (data_source.path.clone(), data_source.payload.clone())
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        self.data_bindings.push(RuntimeDataBinding {
            layer_name: layer.nm.clone(),
            ref_id: layer.ref_id.clone(),
            source_path,
            payload,
        });
    }

    fn process_layer_styles(
        &self,
        layer: &data::Layer,
        #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>,
    ) -> Vec<LayerStyle> {
        let mut styles = Vec::new();
        if let Some(sy_list) = &layer.sy {
            for sy in sy_list {
                let ty = sy.ty.unwrap_or(8);
                let mut kind = None;
                if ty == 0 {
                    kind = Some("DropShadow");
                } else if ty == 1 {
                    kind = Some("InnerShadow");
                } else if ty == 2 {
                    kind = Some("OuterGlow");
                } else if let Some(nm) = &sy.nm {
                    if nm.contains("Stroke") {
                        kind = Some("Stroke");
                    }
                }

                if kind.is_none() {
                    if ty == 3 || ty == 8 {
                        kind = Some("Stroke");
                    }
                }

                if let Some(k) = kind {
                    match k {
                        "DropShadow" => {
                            let color = self.resolve_json_vec4_arr(
                                &sy.c,
                                self.layer_time(layer),
                                evaluator.as_deref_mut(),
                            );
                            let opacity = Animator::resolve_simple(
                                &sy.o,
                                self.layer_time(layer),
                                |v| *v / 100.0,
                                1.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            let angle = Animator::resolve_simple(
                                &sy.a,
                                self.layer_time(layer),
                                |v| *v,
                                0.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            let distance = Animator::resolve_simple(
                                &sy.d,
                                self.layer_time(layer),
                                |v| *v,
                                0.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            let size = Animator::resolve_simple(
                                &sy.s,
                                self.layer_time(layer),
                                |v| *v,
                                0.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            let spread = Animator::resolve_simple(
                                &sy.ch,
                                self.layer_time(layer),
                                |v| *v,
                                0.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            styles.push(LayerStyle::DropShadow {
                                color,
                                opacity,
                                angle,
                                distance,
                                size,
                                spread,
                            });
                        }
                        "InnerShadow" => {
                            let color = self.resolve_json_vec4_arr(
                                &sy.c,
                                self.layer_time(layer),
                                evaluator.as_deref_mut(),
                            );
                            let opacity = Animator::resolve_simple(
                                &sy.o,
                                self.layer_time(layer),
                                |v| *v / 100.0,
                                1.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            let angle = Animator::resolve_simple(
                                &sy.a,
                                self.layer_time(layer),
                                |v| *v,
                                0.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            let distance = Animator::resolve_simple(
                                &sy.d,
                                self.layer_time(layer),
                                |v| *v,
                                0.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            let size = Animator::resolve_simple(
                                &sy.s,
                                self.layer_time(layer),
                                |v| *v,
                                0.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            let choke = Animator::resolve_simple(
                                &sy.ch,
                                self.layer_time(layer),
                                |v| *v,
                                0.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            styles.push(LayerStyle::InnerShadow {
                                color,
                                opacity,
                                angle,
                                distance,
                                size,
                                choke,
                            });
                        }
                        "OuterGlow" => {
                            let color = self.resolve_json_vec4_arr(
                                &sy.c,
                                self.layer_time(layer),
                                evaluator.as_deref_mut(),
                            );
                            let opacity = Animator::resolve_simple(
                                &sy.o,
                                self.layer_time(layer),
                                |v| *v / 100.0,
                                1.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            let size = Animator::resolve_simple(
                                &sy.s,
                                self.layer_time(layer),
                                |v| *v,
                                0.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            let range = Animator::resolve_simple(
                                &sy.ch,
                                self.layer_time(layer),
                                |v| *v,
                                0.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            styles.push(LayerStyle::OuterGlow {
                                color,
                                opacity,
                                size,
                                range,
                            });
                        }
                        "Stroke" => {
                            let color = self.resolve_json_vec4_arr(
                                &sy.c,
                                self.layer_time(layer),
                                evaluator.as_deref_mut(),
                            );
                            let opacity = Animator::resolve_simple(
                                &sy.o,
                                self.layer_time(layer),
                                |v| *v / 100.0,
                                1.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            let width = Animator::resolve_simple(
                                &sy.s,
                                self.layer_time(layer),
                                |v| *v,
                                0.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            styles.push(LayerStyle::Stroke {
                                color,
                                width,
                                opacity,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
        styles
    }

    fn process_effects(
        &self,
        layer: &data::Layer,
        #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>,
    ) -> Vec<Effect> {
        let mut effects = Vec::new();
        if let Some(ef_list) = &layer.ef {
            for ef in ef_list {
                if let Some(en) = ef.en {
                    if en == 0 {
                        continue;
                    }
                }
                let ty = ef.ty.unwrap_or(0);
                let values = if let Some(vals) = &ef.ef {
                    vals
                } else {
                    continue;
                };

                match ty {
                    20 => {
                        let black = self.find_effect_vec4(
                            values,
                            0,
                            "Black",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let white = self.find_effect_vec4(
                            values,
                            1,
                            "White",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let amount = self.find_effect_scalar(
                            values,
                            2,
                            "Intensity",
                            layer,
                            evaluator.as_deref_mut(),
                        ) / 100.0;
                        effects.push(Effect::Tint {
                            black,
                            white,
                            amount,
                        });
                    }
                    21 => {
                        let color = self.find_effect_vec4(
                            values,
                            2,
                            "Color",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let opacity = self.find_effect_scalar(
                            values,
                            6,
                            "Opacity",
                            layer,
                            evaluator.as_deref_mut(),
                        ) / 100.0;
                        effects.push(Effect::Fill { color, opacity });
                    }
                    22 => {
                        let color = self.find_effect_vec4(
                            values,
                            3,
                            "Color",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let width = self.find_effect_scalar(
                            values,
                            4,
                            "Brush Size",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let opacity = self.find_effect_scalar(
                            values,
                            6,
                            "Opacity",
                            layer,
                            evaluator.as_deref_mut(),
                        ) / 100.0;
                        let all_masks_val = self.find_effect_scalar(
                            values,
                            9999,
                            "All Masks",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let all_masks = all_masks_val > 0.5;
                        let mut mask_idx_val = self.find_effect_scalar(
                            values,
                            9999,
                            "Path",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        if mask_idx_val < 0.5 {
                            mask_idx_val = self.find_effect_scalar(
                                values,
                                9999,
                                "Mask",
                                layer,
                                evaluator.as_deref_mut(),
                            );
                        }
                        let mask_index = if mask_idx_val >= 0.5 {
                            Some(mask_idx_val.round() as usize)
                        } else {
                            None
                        };
                        effects.push(Effect::Stroke {
                            color,
                            width,
                            opacity,
                            mask_index,
                            all_masks,
                        });
                    }
                    23 => {
                        let highlights = self.find_effect_vec4(
                            values,
                            0,
                            "bright",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let midtones = self.find_effect_vec4(
                            values,
                            1,
                            "mid",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let shadows = self.find_effect_vec4(
                            values,
                            2,
                            "dark",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        effects.push(Effect::Tritone {
                            highlights,
                            midtones,
                            shadows,
                        });
                    }
                    24 => {
                        let in_black = self.find_effect_scalar(
                            values,
                            3,
                            "inblack",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let in_white = self.find_effect_scalar(
                            values,
                            4,
                            "inwhite",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let gamma = self.find_effect_scalar(
                            values,
                            5,
                            "gamma",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let out_black = self.find_effect_scalar(
                            values,
                            6,
                            "outblack",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let out_white = self.find_effect_scalar(
                            values,
                            7,
                            "outwhite",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        effects.push(Effect::Levels {
                            in_black,
                            in_white,
                            gamma,
                            out_black,
                            out_white,
                        });
                    }
                    25 => {
                        let mut color = self.find_effect_vec4(
                            values,
                            0,
                            "color",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let opacity = self.find_effect_scalar(
                            values,
                            1,
                            "opacity",
                            layer,
                            evaluator.as_deref_mut(),
                        ) / 100.0;
                        color.w *= opacity.clamp(0.0, 1.0);

                        let direction = self.find_effect_scalar(
                            values,
                            2,
                            "direction",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let distance = self.find_effect_scalar(
                            values,
                            3,
                            "distance",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let blur = self
                            .find_effect_scalar(
                                values,
                                4,
                                "softness",
                                layer,
                                evaluator.as_deref_mut(),
                            )
                            .max(0.0);
                        let angle = direction.to_radians();
                        let offset = Vec2::new(angle.cos() * distance, angle.sin() * distance);
                        effects.push(Effect::DropShadow {
                            color,
                            offset,
                            blur,
                        });
                    }
                    26 => {
                        let completion = (self.find_effect_scalar(
                            values,
                            0,
                            "completion",
                            layer,
                            evaluator.as_deref_mut(),
                        ) / 100.0)
                            .clamp(0.0, 1.0);
                        let start_angle = self.find_effect_scalar(
                            values,
                            1,
                            "start angle",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let center_v = self.find_effect_vec4(
                            values,
                            2,
                            "center",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let wipe = self.find_effect_scalar(
                            values,
                            3,
                            "wipe",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let feather = (self.find_effect_scalar(
                            values,
                            4,
                            "feather",
                            layer,
                            evaluator.as_deref_mut(),
                        ) / 100.0)
                            .clamp(0.0, 1.0);

                        effects.push(Effect::RadialWipe {
                            completion,
                            start_angle,
                            center: Vec2::new(center_v.x, center_v.y),
                            wipe,
                            feather,
                        });
                    }
                    27 => {
                        let max_h = self.find_effect_scalar(
                            values,
                            9999,
                            "horizontal",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let max_v = self.find_effect_scalar(
                            values,
                            9999,
                            "vertical",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let mut scale = max_h.abs().max(max_v.abs());
                        if scale <= 0.0 {
                            scale = self
                                .find_effect_scalar(
                                    values,
                                    0,
                                    "scale",
                                    layer,
                                    evaluator.as_deref_mut(),
                                )
                                .abs();
                        }

                        let x_channel = parse_color_channel(self.find_effect_scalar(
                            values,
                            9999,
                            "use for horizontal",
                            layer,
                            evaluator.as_deref_mut(),
                        ));
                        let y_channel = parse_color_channel(self.find_effect_scalar(
                            values,
                            9999,
                            "use for vertical",
                            layer,
                            evaluator.as_deref_mut(),
                        ));
                        effects.push(Effect::DisplacementMap {
                            scale,
                            x_channel,
                            y_channel,
                        });
                    }
                    28 => {
                        let channel = self
                            .find_effect_scalar(
                                values,
                                1,
                                "channel",
                                layer,
                                evaluator.as_deref_mut(),
                            )
                            .round() as i32;
                        let invert = self.find_effect_scalar(
                            values,
                            2,
                            "invert",
                            layer,
                            evaluator.as_deref_mut(),
                        ) > 0.5;
                        let show_mask = self.find_effect_scalar(
                            values,
                            4,
                            "show mask",
                            layer,
                            evaluator.as_deref_mut(),
                        ) > 0.5;
                        let premultiply = self.find_effect_scalar(
                            values,
                            5,
                            "premultiply",
                            layer,
                            evaluator.as_deref_mut(),
                        ) > 0.5;
                        effects.push(Effect::Matte3 {
                            channel,
                            invert,
                            show_mask,
                            premultiply,
                        });
                    }
                    29 => {
                        let blurriness = self
                            .find_effect_scalar(
                                values,
                                0,
                                "blurriness",
                                layer,
                                evaluator.as_deref_mut(),
                            )
                            .max(0.0);
                        let sigma = blurriness / 2.0;
                        effects.push(Effect::GaussianBlur { sigma });
                    }
                    30 => {
                        let angle = self.find_effect_scalar(
                            values,
                            0,
                            "angle",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let radius = self
                            .find_effect_scalar(
                                values,
                                1,
                                "radius",
                                layer,
                                evaluator.as_deref_mut(),
                            )
                            .max(0.0);
                        let center_v = self.find_effect_vec4(
                            values,
                            2,
                            "center",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        effects.push(Effect::Twirl {
                            angle,
                            radius,
                            center: Vec2::new(center_v.x, center_v.y),
                        });
                    }
                    31 => {
                        let rows = self
                            .find_effect_scalar(values, 0, "rows", layer, evaluator.as_deref_mut())
                            .max(1.0);
                        let columns = self
                            .find_effect_scalar(
                                values,
                                1,
                                "columns",
                                layer,
                                evaluator.as_deref_mut(),
                            )
                            .max(1.0);
                        let quality = self.find_effect_scalar(
                            values,
                            2,
                            "quality",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        effects.push(Effect::MeshWarp {
                            rows,
                            columns,
                            quality,
                        });
                    }
                    32 => {
                        let radius = self.find_effect_scalar(
                            values,
                            0,
                            "radius",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let center_v = self.find_effect_vec4(
                            values,
                            1,
                            "center",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let conversion_type = self
                            .find_effect_scalar(
                                values,
                                2,
                                "conversion type",
                                layer,
                                evaluator.as_deref_mut(),
                            )
                            .round() as i32;
                        let speed = self
                            .find_effect_scalar(values, 3, "speed", layer, evaluator.as_deref_mut())
                            .round() as i32;
                        let width = self.find_effect_scalar(
                            values,
                            4,
                            "width",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let height = self.find_effect_scalar(
                            values,
                            5,
                            "height",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let phase = self.find_effect_scalar(
                            values,
                            6,
                            "phase",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        effects.push(Effect::Wavy {
                            radius,
                            center: Vec2::new(center_v.x, center_v.y),
                            conversion_type,
                            speed,
                            width,
                            height,
                            phase,
                        });
                    }
                    33 => {
                        let radius = self
                            .find_effect_scalar(
                                values,
                                0,
                                "radius",
                                layer,
                                evaluator.as_deref_mut(),
                            )
                            .max(0.0);
                        let center_v = self.find_effect_vec4(
                            values,
                            1,
                            "center",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        effects.push(Effect::Spherize {
                            radius,
                            center: Vec2::new(center_v.x, center_v.y),
                        });
                    }
                    34 => {
                        let engine = self
                            .find_effect_scalar(
                                values,
                                0,
                                "engine",
                                layer,
                                evaluator.as_deref_mut(),
                            )
                            .round() as i32;
                        let refinement = self.find_effect_scalar(
                            values,
                            1,
                            "refinement",
                            layer,
                            evaluator.as_deref_mut(),
                        );
                        let on_transparent = self
                            .find_effect_scalar(
                                values,
                                2,
                                "transparent",
                                layer,
                                evaluator.as_deref_mut(),
                            )
                            .round() as i32;
                        effects.push(Effect::Puppet {
                            engine,
                            refinement,
                            on_transparent,
                        });
                    }
                    5 => {
                        let controls = values
                            .iter()
                            .map(|v| CustomEffectControl {
                                name: v.nm.clone(),
                                ty: v.ty,
                                values: self.extract_effect_control_values(v),
                            })
                            .collect();
                        effects.push(Effect::CustomGroup {
                            name: ef.nm.clone(),
                            controls,
                        });
                    }
                    _ => {
                        log_unsupported_effect_once(ty, ef.nm.as_deref());
                        effects.push(Effect::Unsupported {
                            ty,
                            name: ef.nm.clone(),
                        });
                    }
                }
            }
        }
        effects
    }

    fn extract_effect_control_values(&self, effect_value: &data::EffectValue) -> Vec<f32> {
        let Some(prop) = &effect_value.v else {
            return Vec::new();
        };

        let to_numbers = |v: &serde_json::Value| -> Vec<f32> {
            match v {
                serde_json::Value::Number(n) => n.as_f64().map(|x| vec![x as f32]).unwrap_or_default(),
                serde_json::Value::Array(arr) => arr
                    .iter()
                    .filter_map(|item| item.as_f64().map(|x| x as f32))
                    .collect(),
                _ => Vec::new(),
            }
        };

        match &prop.k {
            data::Value::Static(v) => to_numbers(v),
            data::Value::Animated(keys) => {
                for key in keys {
                    if let Some(s) = &key.s {
                        let values = to_numbers(s);
                        if !values.is_empty() {
                            return values;
                        }
                    }
                }
                Vec::new()
            }
            data::Value::Default => Vec::new(),
        }
    }

    fn find_effect_scalar(
        &self,
        values: &[data::EffectValue],
        index: usize,
        name_hint: &str,
        layer: &data::Layer,
        #[cfg(feature = "expressions")] mut evaluator_opt: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>,
    ) -> f32 {
        let hint = name_hint.to_ascii_lowercase();
        if let Some(v) = values.get(index) {
            if let Some(prop) = &v.v {
                #[cfg(feature = "expressions")]
                return self.resolve_json_scalar(
                    prop,
                    self.layer_time(layer),
                    evaluator_opt.as_deref_mut(),
                );
                #[cfg(not(feature = "expressions"))]
                return self.resolve_json_scalar(
                    prop,
                    self.layer_time(layer),
                    evaluator.as_deref_mut(),
                );
            }
        }
        for v in values {
            if let Some(nm) = &v.nm {
                if nm.to_ascii_lowercase().contains(&hint) {
                    if let Some(prop) = &v.v {
                        #[cfg(feature = "expressions")]
                        return self.resolve_json_scalar(
                            prop,
                            self.layer_time(layer),
                            evaluator_opt.as_deref_mut(),
                        );
                        #[cfg(not(feature = "expressions"))]
                        return self.resolve_json_scalar(
                            prop,
                            self.layer_time(layer),
                            evaluator.as_deref_mut(),
                        );
                    }
                }
            }
        }
        0.0
    }

    fn find_effect_vec4(
        &self,
        values: &[data::EffectValue],
        index: usize,
        name_hint: &str,
        layer: &data::Layer,
        #[cfg(feature = "expressions")] mut evaluator_opt: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>,
    ) -> Vec4 {
        let hint = name_hint.to_ascii_lowercase();
        if let Some(v) = values.get(index) {
            if let Some(prop) = &v.v {
                #[cfg(feature = "expressions")]
                return self.resolve_json_vec4(
                    prop,
                    self.layer_time(layer),
                    evaluator_opt.as_deref_mut(),
                );
                #[cfg(not(feature = "expressions"))]
                return self.resolve_json_vec4(
                    prop,
                    self.layer_time(layer),
                    evaluator.as_deref_mut(),
                );
            }
        }
        for v in values {
            if let Some(nm) = &v.nm {
                if nm.to_ascii_lowercase().contains(&hint) {
                    if let Some(prop) = &v.v {
                        #[cfg(feature = "expressions")]
                        return self.resolve_json_vec4(
                            prop,
                            self.layer_time(layer),
                            evaluator_opt.as_deref_mut(),
                        );
                        #[cfg(not(feature = "expressions"))]
                        return self.resolve_json_vec4(
                            prop,
                            self.layer_time(layer),
                            evaluator.as_deref_mut(),
                        );
                    }
                }
            }
        }
        Vec4::ZERO
    }

    fn resolve_json_scalar(
        &self,
        prop: &data::Property<serde_json::Value>,
        frame: f32,
        #[cfg(feature = "expressions")] evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] evaluator: Option<&mut ()>,
    ) -> f32 {
        Animator::resolve_simple(
            prop,
            frame,
            |v| {
                if let Some(n) = v.as_f64() {
                    n as f32
                } else if let Some(arr) = v.as_array() {
                    arr.get(0).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32
                } else {
                    0.0
                }
            },
            0.0,
            evaluator,
            self.asset._frame_rate,
        )
    }

    fn resolve_json_vec4(
        &self,
        prop: &data::Property<serde_json::Value>,
        frame: f32,
        #[cfg(feature = "expressions")] evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] evaluator: Option<&mut ()>,
    ) -> Vec4 {
        Animator::resolve_simple(
            prop,
            frame,
            |v| {
                if let Some(arr) = v.as_array() {
                    let r = arr.get(0).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
                    let g = arr.get(1).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
                    let b = arr.get(2).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
                    let a = arr.get(3).and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
                    Vec4::new(r, g, b, a)
                } else {
                    Vec4::ZERO
                }
            },
            Vec4::ZERO,
            evaluator,
            self.asset._frame_rate,
        )
    }

    fn resolve_json_vec4_arr(
        &self,
        prop: &data::Property<Vec<f32>>,
        frame: f32,
        #[cfg(feature = "expressions")] evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] evaluator: Option<&mut ()>,
    ) -> Vec4 {
        Animator::resolve_simple(
            prop,
            frame,
            |v| {
                if v.len() >= 4 {
                    Vec4::new(v[0], v[1], v[2], v[3])
                } else if v.len() >= 3 {
                    Vec4::new(v[0], v[1], v[2], 1.0)
                } else {
                    Vec4::ZERO
                }
            },
            Vec4::ONE,
            evaluator,
            self.asset._frame_rate,
        )
    }

    fn resolve_transform(
        &self,
        layer: &data::Layer,
        map: &HashMap<u32, &data::Layer>,
        #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>,
    ) -> Mat4 {
        let local = self.get_layer_transform(layer, evaluator.as_deref_mut());

        if let Some(parent_ind) = layer.parent {
            if let Some(parent) = map.get(&parent_ind) {
                return self.resolve_transform(parent, map, evaluator) * local;
            }
        }
        local
    }

    fn get_layer_transform(
        &self,
        layer: &data::Layer,
        #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>,
    ) -> Mat4 {
        let t_frame = self.layer_time(layer);
        let ks = &layer.ks;

        let is_3d = layer.ddd.unwrap_or(0) == 1 || layer.ty == 13;

        // Camera LookAt Check
        if layer.ty == 13 {
            // Position
            let pos = match &ks.p {
                data::PositionProperty::Unified(p) => Animator::resolve_simple(
                    p,
                    t_frame,
                    |v| Vec3::from(v.0),
                    Vec3::ZERO,
                    evaluator.as_deref_mut(),
                    self.asset._frame_rate,
                ),
                data::PositionProperty::Split { x, y, z } => {
                    let px = Animator::resolve_simple(
                        x,
                        t_frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let py = Animator::resolve_simple(
                        y,
                        t_frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let pz = if let Some(z_prop) = z {
                        Animator::resolve_simple(
                            z_prop,
                            t_frame,
                            |v| *v,
                            0.0,
                            evaluator.as_deref_mut(),
                            self.asset._frame_rate,
                        )
                    } else {
                        0.0
                    };
                    Vec3::new(px, py, pz)
                }
            };

            // Point of Interest (Anchor)
            let anchor = Animator::resolve_simple(
                &ks.a,
                t_frame,
                |v| Vec3::from(v.0),
                Vec3::ZERO,
                evaluator.as_deref_mut(),
                self.asset._frame_rate,
            );

            // Use LookAt logic
            // View = LookAt(pos, anchor, up)
            // Global Transform = View.inverse()
            // But we are resolving LOCAL transform here?
            // As established, we assume p and a are in parent/global space context.
            // If Camera has parent, this local transform is applied relative to parent.
            // LookAt constructs a matrix that transforms points from Local(Camera) to World (or Parent).
            // Actually `look_at_rh` creates View Matrix (World -> Camera).
            // We want Camera -> World (Transform).
            // So we return `look_at_rh(pos, anchor, UP).inverse()`.

            let up = Vec3::new(0.0, -1.0, 0.0); // Y down -> Up is -Y
            let view = Mat4::look_at_rh(pos, anchor, up);

            // Roll?
            // If we apply roll, it is around the local Z axis.
            // Camera looks down -Z.
            // Roll Z means rotate around Z.
            let rz = Animator::resolve_simple(
                &ks.rz,
                t_frame,
                |v| v.to_radians(),
                0.0,
                evaluator.as_deref_mut(),
                self.asset._frame_rate,
            );
            // Inverse of (Roll * View) ? No.
            // Camera Transform = (RotZ * View).inverse() ?
            // Or Transform = View.inverse() * RotZ?
            // Let's assume Transform = LookAtInv * RotZ.
            return view.inverse() * Mat4::from_rotation_z(rz);
        }

        let mut anchor = Animator::resolve_simple(
            &ks.a,
            t_frame,
            |v| Vec3::from(v.0),
            Vec3::ZERO,
            evaluator.as_deref_mut(),
            self.asset._frame_rate,
        );

        let mut pos = match &ks.p {
            data::PositionProperty::Unified(p) => {
                let resolved = Animator::resolve_simple(
                    p,
                    t_frame,
                    |v| Vec3::from(v.0),
                    Vec3::ZERO,
                    evaluator.as_deref_mut(),
                    self.asset._frame_rate,
                );
                resolved
            }
            data::PositionProperty::Split { x, y, z } => {
                let px = Animator::resolve_simple(
                    x,
                    t_frame,
                    |v| *v,
                    0.0,
                    evaluator.as_deref_mut(),
                    self.asset._frame_rate,
                );
                let py = Animator::resolve_simple(
                    y,
                    t_frame,
                    |v| *v,
                    0.0,
                    evaluator.as_deref_mut(),
                    self.asset._frame_rate,
                );
                let pz = if let Some(z_prop) = z {
                    Animator::resolve_simple(
                        z_prop,
                        t_frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    )
                } else {
                    0.0
                };
                Vec3::new(px, py, pz)
            }
        };

        let scale = Animator::resolve_simple(
            &ks.s,
            t_frame,
            |v| Vec3::from(v.0) / 100.0,
            Vec3::ONE,
            evaluator.as_deref_mut(),
            self.asset._frame_rate,
        );

        let rz = Animator::resolve_simple(
            &ks.rz,
            t_frame,
            |v| v.to_radians(),
            0.0,
            evaluator.as_deref_mut(),
            self.asset._frame_rate,
        );
        let mut rx = 0.0;
        let mut ry = 0.0;
        if let Some(p) = &ks.rx {
            rx = Animator::resolve_simple(
                p,
                t_frame,
                |v| v.to_radians(),
                0.0,
                evaluator.as_deref_mut(),
                self.asset._frame_rate,
            );
        }
        if let Some(p) = &ks.ry {
            ry = Animator::resolve_simple(
                p,
                t_frame,
                |v| v.to_radians(),
                0.0,
                evaluator.as_deref_mut(),
                self.asset._frame_rate,
            );
        }

        let mut orientation = if let Some(or) = &ks.or {
            Animator::resolve_simple(
                or,
                t_frame,
                |v| Vec3::from(v.0),
                Vec3::ZERO,
                evaluator.as_deref_mut(),
                self.asset._frame_rate,
            )
        } else {
            Vec3::ZERO
        };

        // Resolve skew properties
        let skew = Animator::resolve_simple(
            &ks.sk,
            t_frame,
            |v| v.to_radians(),
            0.0,
            evaluator.as_deref_mut(),
            self.asset._frame_rate,
        );
        let skew_axis = Animator::resolve_simple(
            &ks.sa,
            t_frame,
            |v| v.to_radians(),
            0.0,
            evaluator.as_deref_mut(),
            self.asset._frame_rate,
        );

        // Handle auto-orient along path
        let auto_orient_rotation = if layer.ao == Some(1) {
            self.calculate_auto_orient_rotation(&ks.p, t_frame, evaluator.as_deref_mut())
        } else {
            0.0
        };

        // Enforce 2D limits if not 3D layer
        if !is_3d {
            pos.z = 0.0;
            rx = 0.0;
            ry = 0.0;
            orientation = Vec3::ZERO; // Usually ignored in 2D
                                      // scale.z? leave as is (usually 1.0)
            anchor.z = 0.0;
        }

        // Calculation: T * R * Skew * S * -A
        let mat_t = Mat4::from_translation(pos);

        // Rotation: Orientation * X * Y * Z
        // Orientation (degrees)
        let mat_or = Mat4::from_euler(
            glam::EulerRot::YXZ,
            orientation.y.to_radians(),
            orientation.x.to_radians(),
            orientation.z.to_radians(),
        );

        // Axis Rotations (including auto-orient)
        // Note: negate rotation because Lottie uses clockwise (positive) rotation,
        // but glam uses standard mathematical convention (counter-clockwise for positive)
        let mat_rx = Mat4::from_rotation_x(-rx);
        let mat_ry = Mat4::from_rotation_y(-ry);
        let mat_rz = Mat4::from_rotation_z(-rz - auto_orient_rotation);

        let mat_r = mat_or * mat_rx * mat_ry * mat_rz;

        // Skew matrix: shear along axis defined by skew_axis
        let tan_sk = skew.tan();
        let mat_skew = if skew_axis.abs() < 0.01 {
            // Skew along X axis (sa = 0): shear Y based on X
            Mat4::from_cols(
                Vec4::new(1.0, 0.0, 0.0, 0.0),
                Vec4::new(tan_sk, 1.0, 0.0, 0.0),
                Vec4::new(0.0, 0.0, 1.0, 0.0),
                Vec4::new(0.0, 0.0, 0.0, 1.0),
            )
        } else if (skew_axis - std::f32::consts::FRAC_PI_2).abs() < 0.01 {
            // Skew along Y axis (sa = 90): shear X based on Y
            Mat4::from_cols(
                Vec4::new(1.0, tan_sk, 0.0, 0.0),
                Vec4::new(0.0, 1.0, 0.0, 0.0),
                Vec4::new(0.0, 0.0, 1.0, 0.0),
                Vec4::new(0.0, 0.0, 0.0, 1.0),
            )
        } else {
            // Arbitrary skew axis: rotate, skew, rotate back
            let rot_axis = Mat4::from_rotation_z(skew_axis);
            let rot_back = Mat4::from_rotation_z(-skew_axis);
            let skew_x = Mat4::from_cols(
                Vec4::new(1.0, 0.0, 0.0, 0.0),
                Vec4::new(tan_sk, 1.0, 0.0, 0.0),
                Vec4::new(0.0, 0.0, 1.0, 0.0),
                Vec4::new(0.0, 0.0, 0.0, 1.0),
            );
            rot_axis * skew_x * rot_back
        };

        let mat_s = Mat4::from_scale(scale);
        let mat_a = Mat4::from_translation(-anchor);

        mat_t * mat_r * mat_skew * mat_s * mat_a
    }

    /// Calculate rotation for auto-orient along path
    /// Returns rotation angle in radians to add to Z rotation
    fn calculate_auto_orient_rotation(
        &self,
        position_prop: &data::PositionProperty,
        frame: f32,
        #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>,
    ) -> f32 {
        // Get position at current frame
        let pos = match position_prop {
            data::PositionProperty::Unified(p) => Animator::resolve_simple(
                p,
                frame,
                |v| Vec3::from(v.0),
                Vec3::ZERO,
                evaluator.as_deref_mut(),
                self.asset._frame_rate,
            ),
            data::PositionProperty::Split { x, y, z } => {
                let px = Animator::resolve_simple(
                    x,
                    frame,
                    |v| *v,
                    0.0,
                    evaluator.as_deref_mut(),
                    self.asset._frame_rate,
                );
                let py = Animator::resolve_simple(
                    y,
                    frame,
                    |v| *v,
                    0.0,
                    evaluator.as_deref_mut(),
                    self.asset._frame_rate,
                );
                let pz = if let Some(z_prop) = z {
                    Animator::resolve_simple(
                        z_prop,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    )
                } else {
                    0.0
                };
                Vec3::new(px, py, pz)
            }
        };

        // Get position at next frame for tangent calculation
        let next_frame = frame + 0.1; // Small delta for tangent
        let next_pos = match position_prop {
            data::PositionProperty::Unified(p) => Animator::resolve_simple(
                p,
                next_frame,
                |v| Vec3::from(v.0),
                Vec3::ZERO,
                evaluator.as_deref_mut(),
                self.asset._frame_rate,
            ),
            data::PositionProperty::Split { x, y, z } => {
                let px = Animator::resolve_simple(
                    x,
                    next_frame,
                    |v| *v,
                    0.0,
                    evaluator.as_deref_mut(),
                    self.asset._frame_rate,
                );
                let py = Animator::resolve_simple(
                    y,
                    next_frame,
                    |v| *v,
                    0.0,
                    evaluator.as_deref_mut(),
                    self.asset._frame_rate,
                );
                let pz = if let Some(z_prop) = z {
                    Animator::resolve_simple(
                        z_prop,
                        next_frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    )
                } else {
                    0.0
                };
                Vec3::new(px, py, pz)
            }
        };

        // Calculate tangent and rotation angle
        let tangent = next_pos - pos;
        if tangent.length_squared() < 0.0001 {
            return 0.0;
        }

        // Calculate angle from tangent (2D projection on XY plane)
        let angle = tangent.y.atan2(tangent.x);
        angle
    }

    // Shapes logic must remain 2D (Mat3)
    fn get_shape_transform_2d(
        &self,
        ks: &data::Transform,
        frame: f32,
        #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>,
    ) -> Mat3 {
        // Anchor (2D)
        let anchor_3d = Animator::resolve_simple(
            &ks.a,
            frame,
            |v| Vec3::from(v.0),
            Vec3::ZERO,
            evaluator.as_deref_mut(),
            self.asset._frame_rate,
        );
        let anchor = Vec2::new(anchor_3d.x, anchor_3d.y);

        // Position (2D)
        let pos = match &ks.p {
            data::PositionProperty::Unified(p) => {
                let v3 = Animator::resolve_simple(
                    p,
                    frame,
                    |v| Vec3::from(v.0),
                    Vec3::ZERO,
                    evaluator.as_deref_mut(),
                    self.asset._frame_rate,
                );
                Vec2::new(v3.x, v3.y)
            }
            data::PositionProperty::Split { x, y, .. } => {
                let px = Animator::resolve_simple(
                    x,
                    frame,
                    |v| *v,
                    0.0,
                    evaluator.as_deref_mut(),
                    self.asset._frame_rate,
                );
                let py = Animator::resolve_simple(
                    y,
                    frame,
                    |v| *v,
                    0.0,
                    evaluator.as_deref_mut(),
                    self.asset._frame_rate,
                );
                Vec2::new(px, py)
            }
        };

        // Scale (2D)
        let s3 = Animator::resolve_simple(
            &ks.s,
            frame,
            |v| Vec3::from(v.0) / 100.0,
            Vec3::ONE,
            evaluator.as_deref_mut(),
            self.asset._frame_rate,
        );
        let scale = Vec2::new(s3.x, s3.y);

        // Rotation (Z)
        let r = Animator::resolve_simple(
            &ks.rz,
            frame,
            |v| v.to_radians(),
            0.0,
            evaluator.as_deref_mut(),
            self.asset._frame_rate,
        );

        // Skew properties (2D)
        let skew = Animator::resolve_simple(
            &ks.sk,
            frame,
            |v| v.to_radians(),
            0.0,
            evaluator.as_deref_mut(),
            self.asset._frame_rate,
        );
        let skew_axis = Animator::resolve_simple(
            &ks.sa,
            frame,
            |v| v.to_radians(),
            0.0,
            evaluator.as_deref_mut(),
            self.asset._frame_rate,
        );

        let mat_t = Mat3::from_translation(pos);
        // Note: negate rotation because Lottie uses clockwise (positive) rotation,
        // but glam uses standard mathematical convention (counter-clockwise for positive)
        let mat_r = Mat3::from_rotation_z(-r);

        // Skew matrix in 2D
        let tan_sk = skew.tan();
        let mat_skew = if skew_axis.abs() < 0.01 {
            // Skew along X axis (sa = 0): shear Y based on X
            Mat3::from_cols(
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(tan_sk, 1.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            )
        } else if (skew_axis - std::f32::consts::FRAC_PI_2).abs() < 0.01 {
            // Skew along Y axis (sa = 90): shear X based on Y
            Mat3::from_cols(
                Vec3::new(1.0, tan_sk, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            )
        } else {
            // Arbitrary skew axis: rotate, skew, rotate back
            let rot_axis = Mat3::from_rotation_z(skew_axis);
            let rot_back = Mat3::from_rotation_z(-skew_axis);
            let skew_x = Mat3::from_cols(
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(tan_sk, 1.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            );
            rot_axis * skew_x * rot_back
        };

        let mat_s = Mat3::from_scale(scale);
        let mat_a = Mat3::from_translation(-anchor);

        mat_t * mat_r * mat_skew * mat_s * mat_a
    }

    fn process_shapes(
        &self,
        shapes: &'a [data::Shape],
        frame: f32,
        #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>,
        inherited_trim: Option<Trim>,
        blend_mode: BlendMode,
    ) -> Vec<RenderNode> {
        let mut processed_nodes: Vec<RenderNode> = Vec::new();
        let mut active_geometries: Vec<PendingGeometry> = Vec::new();

        let mut trim: Option<Trim> = inherited_trim;
        for item in shapes {
            if let data::Shape::Trim(t) = item {
                let s = Animator::resolve_simple(
                    &t.s,
                    frame,
                    |v| *v / 100.0,
                    0.0,
                    evaluator.as_deref_mut(),
                    self.asset._frame_rate,
                );
                let e = Animator::resolve_simple(
                    &t.e,
                    frame,
                    |v| *v / 100.0,
                    1.0,
                    evaluator.as_deref_mut(),
                    self.asset._frame_rate,
                );
                let o = Animator::resolve_simple(
                    &t.o,
                    frame,
                    |v| *v / 360.0,
                    0.0,
                    evaluator.as_deref_mut(),
                    self.asset._frame_rate,
                );
                trim = Some(Trim {
                    start: s,
                    end: e,
                    offset: o,
                });
            }
        }

        for item in shapes {
            match item {
                data::Shape::MergePaths(mp) => {
                    if !active_geometries.is_empty() {
                        let mode = match mp.mm {
                            1 => MergeMode::Merge,
                            2 => MergeMode::Add,
                            3 => MergeMode::Subtract,
                            4 => MergeMode::Intersect,
                            5 => MergeMode::Exclude,
                            _ => MergeMode::Merge,
                        };
                        let merged = PendingGeometry {
                            kind: GeometryKind::Merge(active_geometries.clone(), mode),
                            transform: Mat3::IDENTITY,
                        };
                        active_geometries.clear();
                        active_geometries.push(merged);
                    }
                }
                data::Shape::Path(p) => {
                    let path = self.convert_path(p, frame, evaluator.as_deref_mut());
                    active_geometries.push(PendingGeometry {
                        kind: GeometryKind::Path(path),
                        transform: Mat3::IDENTITY,
                    });
                }
                data::Shape::Rect(r) => {
                    let size = Animator::resolve_simple(
                        &r.s,
                        frame,
                        |v| Vec2::from_slice(v),
                        Vec2::ZERO,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let pos = Animator::resolve_simple(
                        &r.p,
                        frame,
                        |v| Vec2::from_slice(v),
                        Vec2::ZERO,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let radius = Animator::resolve_simple(
                        &r.r,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    active_geometries.push(PendingGeometry {
                        kind: GeometryKind::Rect { size, pos, radius },
                        transform: Mat3::IDENTITY,
                    });
                }
                data::Shape::Ellipse(e) => {
                    let size = Animator::resolve_simple(
                        &e.s,
                        frame,
                        |v| Vec2::from_slice(v),
                        Vec2::ZERO,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let pos = Animator::resolve_simple(
                        &e.p,
                        frame,
                        |v| Vec2::from_slice(v),
                        Vec2::ZERO,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    active_geometries.push(PendingGeometry {
                        kind: GeometryKind::Ellipse { size, pos },
                        transform: Mat3::IDENTITY,
                    });
                }
                data::Shape::Polystar(sr) => {
                    let pos = match &sr.p {
                        data::PositionProperty::Unified(p) => Animator::resolve_simple(
                            p,
                            0.0,
                            |v| Vec2::from_slice(&v.0[0..2]),
                            Vec2::ZERO,
                            evaluator.as_deref_mut(),
                            self.asset._frame_rate,
                        ),
                        data::PositionProperty::Split { x, y, .. } => {
                            let px = Animator::resolve_simple(
                                x,
                                0.0,
                                |v| *v,
                                0.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            let py = Animator::resolve_simple(
                                y,
                                0.0,
                                |v| *v,
                                0.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            Vec2::new(px, py)
                        }
                    };
                    let or = Animator::resolve_simple(
                        &sr.or,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let os = Animator::resolve_simple(
                        &sr.os,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let r = Animator::resolve_simple(
                        &sr.r,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let pt = Animator::resolve_simple(
                        &sr.pt,
                        frame,
                        |v| *v,
                        5.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let ir = if let Some(prop) = &sr.ir {
                        Animator::resolve_simple(
                            prop,
                            0.0,
                            |v| *v,
                            0.0,
                            evaluator.as_deref_mut(),
                            self.asset._frame_rate,
                        )
                    } else {
                        0.0
                    };
                    let is = if let Some(prop) = &sr.is {
                        Animator::resolve_simple(
                            prop,
                            0.0,
                            |v| *v,
                            0.0,
                            evaluator.as_deref_mut(),
                            self.asset._frame_rate,
                        )
                    } else {
                        0.0
                    };

                    active_geometries.push(PendingGeometry {
                        kind: GeometryKind::Polystar(PolystarParams {
                            pos,
                            outer_radius: or,
                            inner_radius: ir,
                            outer_roundness: os,
                            inner_roundness: is,
                            rotation: r,
                            points: pt,
                            kind: sr.sy,
                            corner_radius: 0.0,
                        }),
                        transform: Mat3::IDENTITY,
                    });
                }
                data::Shape::Transform(tr) => {
                    // Shape transforms can appear after fill/stroke entries.
                    // Apply to both pending geometry and already materialized nodes.
                    let local = self.get_shape_transform_2d(&tr.t, frame, evaluator.as_deref_mut());
                    for geom in &mut active_geometries {
                        geom.transform = local * geom.transform;
                    }
                    let local4 = Self::mat3_to_mat4_2d(local);
                    for node in &mut processed_nodes {
                        node.transform = local4 * node.transform;
                    }
                }
                data::Shape::RoundCorners(rd) => {
                    let r = Animator::resolve_simple(
                        &rd.r,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    if r > 0.0 {
                        for geom in &mut active_geometries {
                            match &mut geom.kind {
                                GeometryKind::Rect { radius, .. } => *radius += r,
                                GeometryKind::Polystar(p) => p.corner_radius += r,
                                _ => {}
                            }
                        }
                    }
                }
                data::Shape::ZigZag(zz) => {
                    let ridges = Animator::resolve_simple(
                        &zz.r,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let size = Animator::resolve_simple(
                        &zz.s,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let pt = Animator::resolve_simple(
                        &zz.pt,
                        frame,
                        |v| *v,
                        1.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let modifier = ZigZagModifier {
                        ridges,
                        size,
                        smooth: pt > 1.5,
                    };
                    self.apply_modifier_to_active(&mut active_geometries, &modifier);
                }
                data::Shape::PuckerBloat(pb) => {
                    let amount = Animator::resolve_simple(
                        &pb.a,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let modifier = PuckerBloatModifier {
                        amount,
                        center: Vec2::ZERO,
                    };
                    self.apply_modifier_to_active(&mut active_geometries, &modifier);
                }
                data::Shape::Twist(tw) => {
                    let angle = Animator::resolve_simple(
                        &tw.a,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let center = Animator::resolve_simple(
                        &tw.c,
                        frame,
                        |v| Vec2::from_slice(v),
                        Vec2::ZERO,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let modifier = TwistModifier { angle, center };
                    self.apply_modifier_to_active(&mut active_geometries, &modifier);
                }
                data::Shape::OffsetPath(op) => {
                    let amount = Animator::resolve_simple(
                        &op.a,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let miter_limit = op.ml.unwrap_or(4.0);
                    let line_join = op.lj;
                    let modifier = OffsetPathModifier {
                        amount,
                        miter_limit,
                        line_join,
                    };
                    self.apply_modifier_to_active(&mut active_geometries, &modifier);
                }
                data::Shape::WigglePath(wg) => {
                    let speed = Animator::resolve_simple(
                        &wg.s,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let size = Animator::resolve_simple(
                        &wg.w,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let correlation = Animator::resolve_simple(
                        &wg.r,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let seed_prop = Animator::resolve_simple(
                        &wg.sh,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let mut modifier = WiggleModifier {
                        seed: seed_prop,
                        time: frame / 60.0,
                        speed: speed / self.asset._frame_rate,
                        amount: size,
                        correlation,
                    };
                    modifier.time = frame;
                    modifier.speed = speed / self.asset._frame_rate;
                    self.apply_modifier_to_active(&mut active_geometries, &modifier);
                }
                data::Shape::Repeater(rp) => {
                    let copies = Animator::resolve_simple(
                        &rp.c,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let offset = Animator::resolve_simple(
                        &rp.o,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let t_anchor_3d = Animator::resolve_simple(
                        &rp.tr.t.a,
                        frame,
                        |v| Vec3::from(v.0),
                        Vec3::ZERO,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let t_anchor = Vec2::new(t_anchor_3d.x, t_anchor_3d.y);

                    let t_pos = match &rp.tr.t.p {
                        data::PositionProperty::Unified(p) => Animator::resolve_simple(
                            p,
                            0.0,
                            |v| Vec2::from_slice(&v.0[0..2]),
                            Vec2::ZERO,
                            evaluator.as_deref_mut(),
                            self.asset._frame_rate,
                        ),
                        data::PositionProperty::Split { x, y, .. } => {
                            let px = Animator::resolve_simple(
                                x,
                                0.0,
                                |v| *v,
                                0.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            let py = Animator::resolve_simple(
                                y,
                                0.0,
                                |v| *v,
                                0.0,
                                evaluator.as_deref_mut(),
                                self.asset._frame_rate,
                            );
                            Vec2::new(px, py)
                        }
                    };
                    let t_scale_3d = Animator::resolve_simple(
                        &rp.tr.t.s,
                        0.0,
                        |v| Vec3::from(v.0) / 100.0,
                        Vec3::ONE,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let t_scale = Vec2::new(t_scale_3d.x, t_scale_3d.y);

                    let t_rot = Animator::resolve_simple(
                        &rp.tr.t.rz,
                        frame,
                        |v| v.to_radians(),
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );

                    let start_opacity = Animator::resolve_simple(
                        &rp.tr.so,
                        frame,
                        |v| *v / 100.0,
                        1.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let end_opacity = Animator::resolve_simple(
                        &rp.tr.eo,
                        frame,
                        |v| *v / 100.0,
                        1.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );

                    self.apply_repeater(
                        copies,
                        offset,
                        t_anchor,
                        t_pos,
                        t_scale,
                        t_rot,
                        start_opacity,
                        end_opacity,
                        &mut active_geometries,
                        &mut processed_nodes,
                    );
                }
                data::Shape::Fill(f) => {
                    let color = Animator::resolve_simple(
                        &f.c,
                        frame,
                        |v| Vec4::from_slice(v),
                        Vec4::ONE,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let opacity = Animator::resolve_simple(
                        &f.o,
                        frame,
                        |v| *v / 100.0,
                        1.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    for geom in &active_geometries {
                        let path = self.convert_geometry(geom);
                        processed_nodes.push(RenderNode {
                            transform: Mat4::IDENTITY,
                            alpha: 1.0,
                            blend_mode,
                            content: NodeContent::Shape(renderer::Shape {
                                geometry: path,
                                fill: Some(Fill {
                                    paint: Paint::Solid(color),
                                    opacity,
                                    rule: FillRule::NonZero,
                                }),
                                stroke: None,
                                trim: trim.clone(),
                            }),
                            masks: vec![],
                            styles: vec![],
                            matte: None,
                            effects: vec![],
                            is_adjustment_layer: false,
                        });
                    }
                }
                data::Shape::GradientFill(gf) => {
                    let start = Animator::resolve_simple(
                        &gf.s,
                        frame,
                        |v| Vec2::from_slice(v),
                        Vec2::ZERO,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let end = Animator::resolve_simple(
                        &gf.e,
                        frame,
                        |v| Vec2::from_slice(v),
                        Vec2::ZERO,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let opacity = Animator::resolve_simple(
                        &gf.o,
                        frame,
                        |v| *v / 100.0,
                        1.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let raw_stops = Animator::resolve_simple(
                        &gf.g.k,
                        frame,
                        |v| v.clone(),
                        Vec::new(),
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let stops = parse_gradient_stops(&raw_stops, gf.g.p as usize);
                    let kind = if gf.t == 1 {
                        GradientKind::Linear
                    } else {
                        GradientKind::Radial
                    };
                    for geom in &active_geometries {
                        let path = self.convert_geometry(geom);
                        processed_nodes.push(RenderNode {
                            transform: Mat4::IDENTITY,
                            alpha: 1.0,
                            blend_mode,
                            content: NodeContent::Shape(renderer::Shape {
                                geometry: path,
                                fill: Some(Fill {
                                    paint: Paint::Gradient(Gradient {
                                        kind,
                                        stops: stops.clone(),
                                        start,
                                        end,
                                    }),
                                    opacity,
                                    rule: FillRule::NonZero,
                                }),
                                stroke: None,
                                trim: trim.clone(),
                            }),
                            masks: vec![],
                            styles: vec![],
                            matte: None,
                            effects: vec![],
                            is_adjustment_layer: false,
                        });
                    }
                }
                data::Shape::GradientStroke(gs) => {
                    let start = Animator::resolve_simple(
                        &gs.s,
                        frame,
                        |v| Vec2::from_slice(v),
                        Vec2::ZERO,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let end = Animator::resolve_simple(
                        &gs.e,
                        frame,
                        |v| Vec2::from_slice(v),
                        Vec2::ZERO,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let width = Animator::resolve_simple(
                        &gs.w,
                        frame,
                        |v| *v,
                        1.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let opacity = Animator::resolve_simple(
                        &gs.o,
                        frame,
                        |v| *v / 100.0,
                        1.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let raw_stops = Animator::resolve_simple(
                        &gs.g.k,
                        frame,
                        |v| v.clone(),
                        Vec::new(),
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let stops = parse_gradient_stops(&raw_stops, gs.g.p as usize);
                    let kind = if gs.t == 1 {
                        GradientKind::Linear
                    } else {
                        GradientKind::Radial
                    };
                    let dash = self.resolve_dash(&gs.d, frame, evaluator.as_deref_mut());
                    for geom in &active_geometries {
                        let path = self.convert_geometry(geom);
                        processed_nodes.push(RenderNode {
                            transform: Mat4::IDENTITY,
                            alpha: 1.0,
                            blend_mode,
                            content: NodeContent::Shape(renderer::Shape {
                                geometry: path,
                                fill: None,
                                stroke: Some(Stroke {
                                    paint: Paint::Gradient(Gradient {
                                        kind,
                                        stops: stops.clone(),
                                        start,
                                        end,
                                    }),
                                    width,
                                    opacity,
                                    cap: match gs.lc {
                                        1 => LineCap::Butt,
                                        3 => LineCap::Square,
                                        _ => LineCap::Round,
                                    },
                                    join: match gs.lj {
                                        1 => LineJoin::Miter,
                                        3 => LineJoin::Bevel,
                                        _ => LineJoin::Round,
                                    },
                                    miter_limit: gs.ml,
                                    dash: dash.clone(),
                                }),
                                trim: trim.clone(),
                            }),
                            masks: vec![],
                            styles: vec![],
                            matte: None,
                            effects: vec![],
                            is_adjustment_layer: false,
                        });
                    }
                }
                data::Shape::Stroke(s) => {
                    let color = Animator::resolve_simple(
                        &s.c,
                        frame,
                        |v| Vec4::from_slice(v),
                        Vec4::ONE,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let width = Animator::resolve_simple(
                        &s.w,
                        frame,
                        |v| *v,
                        1.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let opacity = Animator::resolve_simple(
                        &s.o,
                        frame,
                        |v| *v / 100.0,
                        1.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    );
                    let dash = self.resolve_dash(&s.d, frame, evaluator.as_deref_mut());
                    for geom in &active_geometries {
                        let path = self.convert_geometry(geom);
                        processed_nodes.push(RenderNode {
                            transform: Mat4::IDENTITY,
                            alpha: 1.0,
                            blend_mode,
                            content: NodeContent::Shape(renderer::Shape {
                                geometry: path,
                                fill: None,
                                stroke: Some(Stroke {
                                    paint: Paint::Solid(color),
                                    width,
                                    opacity,
                                    cap: match s.lc {
                                        1 => LineCap::Butt,
                                        3 => LineCap::Square,
                                        _ => LineCap::Round,
                                    },
                                    join: match s.lj {
                                        1 => LineJoin::Miter,
                                        3 => LineJoin::Bevel,
                                        _ => LineJoin::Round,
                                    },
                                    miter_limit: s.ml,
                                    dash: dash.clone(),
                                }),
                                trim: trim.clone(),
                            }),
                            masks: vec![],
                            styles: vec![],
                            matte: None,
                            effects: vec![],
                            is_adjustment_layer: false,
                        });
                    }
                }
                data::Shape::Group(g) => {
                    let group_nodes = self.process_shapes(
                        &g.it,
                        frame,
                        evaluator.as_deref_mut(),
                        trim.clone(),
                        blend_mode,
                    );
                    processed_nodes.push(RenderNode {
                        transform: Mat4::IDENTITY,
                        alpha: 1.0,
                        blend_mode,
                        content: NodeContent::Group(group_nodes),
                        masks: vec![],
                        styles: vec![],
                        matte: None,
                        effects: vec![],
                        is_adjustment_layer: false,
                    });
                }
                _ => {}
            }
        }
        processed_nodes
    }

    fn resolve_dash(
        &self,
        props: &[data::DashProperty],
        frame: f32,
        #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>,
    ) -> Option<DashPattern> {
        if props.is_empty() {
            return None;
        }
        let mut array = Vec::new();
        let mut offset = 0.0;
        for prop in props {
            match prop.n.as_deref() {
                Some("o") => {
                    offset = Animator::resolve_simple(
                        &prop.v,
                        frame,
                        |v| *v,
                        0.0,
                        evaluator.as_deref_mut(),
                        self.asset._frame_rate,
                    )
                }
                Some("d") | Some("v") | Some("g") => array.push(Animator::resolve_simple(
                    &prop.v,
                    frame,
                    |v| *v,
                    0.0,
                    evaluator.as_deref_mut(),
                    self.asset._frame_rate,
                )),
                _ => {}
            }
        }
        if !array.is_empty() {
            if array.len() % 2 != 0 {
                let clone = array.clone();
                array.extend(clone);
            }
            let total: f32 = array.iter().sum();
            if total > 0.0 {
                offset = (offset % total + total) % total;
            } else {
                offset = 0.0;
            }
            Some(DashPattern { array, offset })
        } else {
            None
        }
    }

    fn apply_repeater(
        &self,
        copies: f32,
        _offset: f32,
        anchor: Vec2,
        pos: Vec2,
        scale: Vec2,
        rot: f32,
        start_op: f32,
        end_op: f32,
        geoms: &mut Vec<PendingGeometry>,
        nodes: &mut Vec<RenderNode>,
    ) {
        let num_copies = copies.round() as usize;
        if num_copies <= 1 {
            return;
        }

        let original_geoms = geoms.clone();
        let original_nodes = nodes.clone();

        let mat_t = Mat3::from_translation(pos);
        let mat_r = Mat3::from_rotation_z(rot); // Radians
        let mat_s = Mat3::from_scale(scale);
        let mat_a = Mat3::from_translation(-anchor);
        let mat_pre_a = Mat3::from_translation(anchor);

        let pivot_transform = mat_pre_a * mat_r * mat_s * mat_a;
        let step_transform = mat_t * pivot_transform;

        // RenderNode uses Mat4, but repeater internal logic here is mixed?
        // nodes have Mat4. step_transform is Mat3.
        // We need Mat4 step transform for nodes.
        let mat_t4 = Mat4::from_translation(Vec3::new(pos.x, pos.y, 0.0));
        let mat_r4 = Mat4::from_rotation_z(rot);
        let mat_s4 = Mat4::from_scale(Vec3::new(scale.x, scale.y, 1.0));
        let mat_a4 = Mat4::from_translation(Vec3::new(-anchor.x, -anchor.y, 0.0));
        let mat_pre_a4 = Mat4::from_translation(Vec3::new(anchor.x, anchor.y, 0.0));
        let step_transform4 = mat_t4 * mat_pre_a4 * mat_r4 * mat_s4 * mat_a4;

        geoms.clear();
        nodes.clear();

        for i in 0..num_copies {
            let t = if num_copies > 1 {
                i as f32 / (num_copies as f32 - 1.0)
            } else {
                0.0
            };
            let op = start_op + (end_op - start_op) * t;

            let mut copy_transform = Mat3::IDENTITY;
            let mut copy_transform4 = Mat4::IDENTITY;
            for _ in 0..i {
                copy_transform = copy_transform * step_transform;
                copy_transform4 = copy_transform4 * step_transform4;
            }

            for geom in &original_geoms {
                let mut g = geom.clone();
                g.transform = copy_transform * g.transform;
                geoms.push(g);
            }

            for node in &original_nodes {
                let mut n = node.clone();
                n.transform = copy_transform4 * n.transform;
                n.alpha *= op;
                nodes.push(n);
            }
        }
    }

    fn apply_modifier_to_active(
        &self,
        active: &mut Vec<PendingGeometry>,
        modifier: &impl GeometryModifier,
    ) {
        for geom in active.iter_mut() {
            let mut path = geom.to_path(self);
            modifier.modify(&mut path);
            geom.transform = Mat3::IDENTITY;
            geom.kind = GeometryKind::Path(path);
        }
    }

    fn convert_geometry(&self, geom: &PendingGeometry) -> ShapeGeometry {
        geom.to_shape_geometry(self)
    }

    fn generate_polystar_path(&self, params: &PolystarParams) -> BezPath {
        let mut path = BezPath::new();
        let num_points = params.points.round();
        if num_points < 3.0 {
            return path;
        }

        let is_star = params.kind == 1;
        let has_roundness =
            params.outer_roundness.abs() > 0.01 || (is_star && params.inner_roundness.abs() > 0.01);
        let total_points = if is_star {
            num_points * 2.0
        } else {
            num_points
        } as usize;
        let current_angle = (params.rotation - 90.0).to_radians();
        let angle_step = 2.0 * PI / total_points as f64;

        if has_roundness {
            let mut elements = Vec::with_capacity(total_points);
            for i in 0..total_points {
                let (r, roundness) = if is_star {
                    if i % 2 == 0 {
                        (params.outer_radius, params.outer_roundness)
                    } else {
                        (params.inner_radius, params.inner_roundness)
                    }
                } else {
                    (params.outer_radius, params.outer_roundness)
                };

                let angle = current_angle as f64 + angle_step * i as f64;
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                let x = params.pos.x as f64 + r as f64 * cos_a;
                let y = params.pos.y as f64 + r as f64 * sin_a;
                let vertex = Point::new(x, y);

                let tx = -sin_a;
                let ty = cos_a;
                let tangent = kurbo::Vec2::new(tx, ty);
                let cp_d = r as f64 * angle_step * roundness as f64 * 0.01;
                let in_cp = vertex - tangent * cp_d;
                let out_cp = vertex + tangent * cp_d;
                elements.push((vertex, in_cp, out_cp));
            }
            if elements.is_empty() {
                return path;
            }
            path.move_to(elements[0].0);
            let len = elements.len();
            for i in 0..len {
                let curr_idx = i;
                let next_idx = (i + 1) % len;
                let curr_out_cp = elements[curr_idx].2;
                let next_in_cp = elements[next_idx].1;
                let next_vertex = elements[next_idx].0;
                path.curve_to(curr_out_cp, next_in_cp, next_vertex);
            }
            path.close_path();
            return path;
        }

        let mut vertices = Vec::with_capacity(total_points);
        for i in 0..total_points {
            let r = if is_star {
                if i % 2 == 0 {
                    params.outer_radius
                } else {
                    params.inner_radius
                }
            } else {
                params.outer_radius
            };
            let angle = current_angle as f64 + angle_step * i as f64;
            let x = params.pos.x as f64 + r as f64 * angle.cos();
            let y = params.pos.y as f64 + r as f64 * angle.sin();
            vertices.push(Point::new(x, y));
        }

        let radius = params.corner_radius as f64;
        if radius <= 0.1 {
            if !vertices.is_empty() {
                path.move_to(vertices[0]);
                for v in vertices.iter().skip(1) {
                    path.line_to(*v);
                }
                path.close_path();
            }
            return path;
        }

        let len = vertices.len();
        for i in 0..len {
            let prev = vertices[(i + len - 1) % len];
            let curr = vertices[i];
            let next = vertices[(i + 1) % len];
            let v1 = prev - curr;
            let v2 = next - curr;
            let len1 = v1.hypot();
            let len2 = v2.hypot();

            if len1 < 0.001 || len2 < 0.001 {
                if i == 0 {
                    path.move_to(curr);
                } else {
                    path.line_to(curr);
                }
                continue;
            }

            let u1 = v1 * (1.0 / len1);
            let u2 = v2 * (1.0 / len2);
            let dot = (u1.x * u2.x + u1.y * u2.y).clamp(-1.0, 1.0);
            let angle = dot.acos();
            let dist = if angle.abs() < 0.001 {
                0.0
            } else {
                radius / (angle / 2.0).tan()
            };
            let max_d = (len1.min(len2)) * 0.5;
            let d = dist.min(max_d);
            let p_start = curr + u1 * d;
            let p_end = curr + u2 * d;

            if i == 0 {
                path.move_to(p_start);
            } else {
                path.line_to(p_start);
            }
            path.quad_to(curr, p_end);
        }
        path.close_path();
        path
    }

    fn convert_path(
        &self,
        p: &data::PathShape,
        frame: f32,
        #[cfg(feature = "expressions")] mut evaluator_opt: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] evaluator: Option<&mut ()>,
    ) -> BezPath {
        #[cfg(feature = "expressions")]
        let path_data = Animator::resolve_simple(
            &p.ks,
            frame,
            |v| v.clone(),
            data::BezierPath::default(),
            evaluator_opt,
            self.asset._frame_rate,
        );
        #[cfg(not(feature = "expressions"))]
        let path_data = Animator::resolve_simple(
            &p.ks,
            frame,
            |v| v.clone(),
            data::BezierPath::default(),
            evaluator,
            self.asset._frame_rate,
        );
        self.convert_bezier_path(&path_data)
    }

    fn convert_bezier_path(&self, path_data: &data::BezierPath) -> BezPath {
        let mut bp = BezPath::new();
        if path_data.v.is_empty() {
            return bp;
        }
        let start = path_data.v[0];
        bp.move_to(Point::new(start[0] as f64, start[1] as f64));
        for i in 0..path_data.v.len() {
            let next_idx = (i + 1) % path_data.v.len();
            if next_idx == 0 && !path_data.c {
                break;
            }
            let p0 = path_data.v[i];
            let p1 = path_data.v[next_idx];
            let o = if i < path_data.o.len() {
                path_data.o[i]
            } else {
                [0.0, 0.0]
            };
            let in_ = if next_idx < path_data.i.len() {
                path_data.i[next_idx]
            } else {
                [0.0, 0.0]
            };
            let cp1 = [p0[0] + o[0], p0[1] + o[1]];
            let cp2 = [p1[0] + in_[0], p1[1] + in_[1]];
            bp.curve_to(
                Point::new(cp1[0] as f64, cp1[1] as f64),
                Point::new(cp2[0] as f64, cp2[1] as f64),
                Point::new(p1[0] as f64, p1[1] as f64),
            );
        }
        if path_data.c {
            bp.close_path();
        }
        bp
    }

    fn process_masks(
        &self,
        masks_props: &[data::MaskProperties],
        frame: f32,
        #[cfg(feature = "expressions")] mut evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] mut evaluator: Option<&mut ()>,
    ) -> Vec<Mask> {
        let mut masks = Vec::new();
        for m in masks_props {
            let mode = match m.mode.as_deref() {
                Some("n") => MaskMode::None,
                Some("a") => MaskMode::Add,
                Some("s") => MaskMode::Subtract,
                Some("i") => MaskMode::Intersect,
                Some("l") => MaskMode::Lighten,
                Some("d") => MaskMode::Darken,
                Some("f") => MaskMode::Difference,
                _ => continue,
            };
            let path_data = Animator::resolve_simple(
                &m.pt,
                frame,
                |v| v.clone(),
                data::BezierPath::default(),
                evaluator.as_deref_mut(),
                self.asset._frame_rate,
            );
            let geometry = self.convert_bezier_path(&path_data);
            let opacity = Animator::resolve_simple(
                &m.o,
                frame,
                |v| *v / 100.0,
                1.0,
                evaluator.as_deref_mut(),
                self.asset._frame_rate,
            );
            let expansion = Animator::resolve_simple(
                &m.x,
                frame,
                |v| *v,
                0.0,
                evaluator.as_deref_mut(),
                self.asset._frame_rate,
            );
            let feather = Animator::resolve_simple(
                &m.feather,
                frame,
                |v| Vec2::from(*v),
                Vec2::ZERO,
                evaluator.as_deref_mut(),
                self.asset._frame_rate,
            );
            let inverted = m.inv;
            masks.push(Mask {
                mode,
                geometry,
                opacity,
                expansion,
                feather,
                inverted,
            });
        }
        masks
    }

    /// Map Lottie blend mode value to BlendMode enum
    /// Lottie blend modes: 0=Normal, 1=Multiply, 2=Screen, 3=Overlay, 4=Darken, 5=Lighten,
    /// 6=ColorDodge, 7=ColorBurn, 8=HardLight, 9=SoftLight, 10=Difference, 11=Exclusion,
    /// 12=Hue, 13=Saturation, 14=Color, 15=Luminosity
    fn resolve_blend_mode(&self, bm: Option<u8>) -> BlendMode {
        match bm.unwrap_or(0) {
            0 => BlendMode::Normal,
            1 => BlendMode::Multiply,
            2 => BlendMode::Screen,
            3 => BlendMode::Overlay,
            4 => BlendMode::Darken,
            5 => BlendMode::Lighten,
            6 => BlendMode::ColorDodge,
            7 => BlendMode::ColorBurn,
            8 => BlendMode::HardLight,
            9 => BlendMode::SoftLight,
            10 => BlendMode::Difference,
            11 => BlendMode::Exclusion,
            12 => BlendMode::Hue,
            13 => BlendMode::Saturation,
            14 => BlendMode::Color,
            15 => BlendMode::Luminosity,
            _ => BlendMode::Normal,
        }
    }
}

// Helpers
struct ColorStop {
    t: f32,
    r: f32,
    g: f32,
    b: f32,
}
struct AlphaStop {
    t: f32,
    a: f32,
}

fn parse_gradient_stops(raw: &[f32], color_count: usize) -> Vec<GradientStop> {
    let mut stops = Vec::new();
    if raw.is_empty() {
        return stops;
    }
    let mut color_stops = Vec::new();
    let mut alpha_stops = Vec::new();
    let color_data_len = color_count * 4;
    for chunk in raw
        .iter()
        .take(color_data_len)
        .collect::<Vec<_>>()
        .chunks(4)
    {
        if chunk.len() == 4 {
            color_stops.push(ColorStop {
                t: *chunk[0],
                r: *chunk[1],
                g: *chunk[2],
                b: *chunk[3],
            });
        }
    }
    if raw.len() > color_data_len {
        for chunk in raw[color_data_len..].chunks(2) {
            if chunk.len() == 2 {
                alpha_stops.push(AlphaStop {
                    t: chunk[0],
                    a: chunk[1],
                });
            }
        }
    }
    if alpha_stops.is_empty() {
        for c in color_stops {
            stops.push(GradientStop {
                offset: c.t,
                color: Vec4::new(c.r, c.g, c.b, 1.0),
            });
        }
        return stops;
    }
    let mut unique_t: Vec<f32> = Vec::new();
    for c in &color_stops {
        unique_t.push(c.t);
    }
    for a in &alpha_stops {
        unique_t.push(a.t);
    }
    unique_t.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    unique_t.dedup();
    for t in unique_t {
        let (r, g, b) = interpolate_color(&color_stops, t);
        let a = interpolate_alpha(&alpha_stops, t);
        stops.push(GradientStop {
            offset: t,
            color: Vec4::new(r, g, b, a),
        });
    }
    stops
}

fn interpolate_color(stops: &[ColorStop], t: f32) -> (f32, f32, f32) {
    if stops.is_empty() {
        return (1.0, 1.0, 1.0);
    }
    if t <= stops[0].t {
        return (stops[0].r, stops[0].g, stops[0].b);
    }
    if t >= stops.last().unwrap().t {
        let last = stops.last().unwrap();
        return (last.r, last.g, last.b);
    }
    for i in 0..stops.len() - 1 {
        let s1 = &stops[i];
        let s2 = &stops[i + 1];
        if t >= s1.t && t <= s2.t {
            let range = s2.t - s1.t;
            let ratio = if range == 0.0 {
                0.0
            } else {
                (t - s1.t) / range
            };
            return (
                s1.r + (s2.r - s1.r) * ratio,
                s1.g + (s2.g - s1.g) * ratio,
                s1.b + (s2.b - s1.b) * ratio,
            );
        }
    }
    (1.0, 1.0, 1.0)
}

fn interpolate_alpha(stops: &[AlphaStop], t: f32) -> f32 {
    if stops.is_empty() {
        return 1.0;
    }
    if t <= stops[0].t {
        return stops[0].a;
    }
    if t >= stops.last().unwrap().t {
        return stops.last().unwrap().a;
    }
    for i in 0..stops.len() - 1 {
        let s1 = &stops[i];
        let s2 = &stops[i + 1];
        if t >= s1.t && t <= s2.t {
            let range = s2.t - s1.t;
            let ratio = if range == 0.0 {
                0.0
            } else {
                (t - s1.t) / range
            };
            return s1.a + (s2.a - s1.a) * ratio;
        }
    }
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use lottie_data::model as data;
    use serde_json::json;

    fn first_shape_world_transform(tree: &RenderTree) -> Option<Mat4> {
        fn visit(node: &RenderNode, parent: Mat4, out: &mut Option<Mat4>) {
            if out.is_some() {
                return;
            }
            let world = parent * node.transform;
            match &node.content {
                NodeContent::Shape(_) => *out = Some(world),
                NodeContent::Group(children) => {
                    for child in children {
                        visit(child, world, out);
                        if out.is_some() {
                            break;
                        }
                    }
                }
                _ => {}
            }
        }

        let mut out = None;
        visit(&tree.root, Mat4::IDENTITY, &mut out);
        out
    }

    #[test]
    fn test_camera_transform() {
        let camera_layer = data::Layer {
            ty: 13,
            ind: Some(1),
            parent: None,
            nm: Some("Camera".to_string()),
            ip: 0.0,
            op: 60.0,
            st: 0.0,
            sr: 1.0,
            ks: data::Transform {
                p: data::PositionProperty::Unified(data::Property {
                    k: data::Value::Static(data::Vec3DefaultZero([0.0, 0.0, -500.0])),
                    ..Default::default()
                }),
                a: data::Property {
                    k: data::Value::Static(data::Vec3DefaultZero([0.0, 0.0, 0.0])),
                    ..Default::default()
                },
                ..Default::default()
            },
            pe: Some(data::Property {
                k: data::Value::Static(1000.0),
                ..Default::default()
            }),
            hd: None,
            ddd: Some(1),
            au: None,
            ao: None,
            tm: None,
            masks_properties: None,
            tt: None,
            ef: None,
            sy: None,
            bm: None,
            ref_id: None,
            w: None,
            h: None,
            color: None,
            sw: None,
            sh: None,
            shapes: None,
            t: None,
            match_name: None,
            tp: None,
            td: None,
            has_mask: None,
            mb: None,
            css_class: None,
            layer_id: None,
            xml_tag: None,
            collapse_transform_deprecated: None,
            collapse_transform: None,
        };

        let model = LottieJson {
            v: None,
            nm: None,
            ip: 0.0,
            op: 60.0,
            fr: 60.0,
            w: 1000,
            h: 1000,
            ddd: None,
            bg: None,
            layers: vec![camera_layer],
            assets: vec![],
            markers: vec![],
            metadata: None,
        };

        let mut player = LottiePlayer::new();
        player.load_json(model);
        let tree = player.render_tree();

        let vm = tree.view_matrix;
        // World point (0,0,0) -> View Space
        let p_world = Vec4::new(0.0, 0.0, 0.0, 1.0);
        let p_view = vm * p_world;

        // Camera at -500. Looking at 0.
        // In View Space (RH, -Z forward), the object at 0 (distance 500 in front)
        // should be at Z = -500.
        assert!(
            (p_view.z - (-500.0)).abs() < 1.0,
            "Expected Z=-500, got {}",
            p_view.z
        );
    }

    #[test]
    fn test_group_transform_applies_when_transform_follows_fill() {
        let model: LottieJson = serde_json::from_value(json!({
            "fr": 30.0,
            "ip": 0.0,
            "op": 2.0,
            "w": 100,
            "h": 100,
            "layers": [{
                "ty": 4,
                "ind": 1,
                "ip": 0.0,
                "op": 2.0,
                "st": 0.0,
                "sr": 1.0,
                "ks": {
                    "a": { "k": [0.0, 0.0, 0.0] },
                    "p": { "k": [0.0, 0.0, 0.0] },
                    "s": { "k": [100.0, 100.0, 100.0] },
                    "r": { "k": 0.0 },
                    "o": { "k": 100.0 }
                },
                "shapes": [{
                    "ty": "gr",
                    "nm": "Root Group",
                    "it": [
                        { "ty": "rc", "nm": "Rect", "p": { "k": [0.0, 0.0] }, "s": { "k": [20.0, 20.0] }, "r": { "k": 0.0 } },
                        { "ty": "fl", "nm": "Fill", "c": { "k": [1.0, 0.0, 0.0, 1.0] }, "o": { "k": 100.0 } },
                        { "ty": "tr", "nm": "Group Transform", "a": { "k": [0.0, 0.0, 0.0] }, "p": { "k": [50.0, 50.0, 0.0] }, "s": { "k": [100.0, 100.0, 100.0] }, "r": { "k": 0.0 }, "o": { "k": 100.0 } }
                    ]
                }]
            }]
        }))
        .expect("valid lottie json");

        let mut player = LottiePlayer::new();
        player.load_json(model);
        player.current_frame = 0.0;

        let tree = player.render_tree();
        let world = first_shape_world_transform(&tree).expect("shape node should exist");
        let m = world.to_cols_array();
        assert!(
            (m[12] - 50.0).abs() < 0.01 && (m[13] - 50.0).abs() < 0.01,
            "expected translated shape at (50, 50), got ({}, {})",
            m[12],
            m[13]
        );
    }

    #[test]
    fn test_layer_scale_keyframes_use_composition_time_not_layer_start_offset() {
        let model: LottieJson = serde_json::from_value(json!({
            "fr": 30.0,
            "ip": 0.0,
            "op": 120.0,
            "w": 200,
            "h": 200,
            "layers": [{
                "ty": 4,
                "ind": 1,
                "ip": 0.0,
                "op": 120.0,
                "st": 40.0,
                "sr": 1.0,
                "ks": {
                    "a": { "k": [0.0, 0.0, 0.0] },
                    "p": { "k": [0.0, 0.0, 0.0] },
                    "s": {
                        "a": 1,
                        "k": [
                            { "t": 40.0, "s": [100.0, 100.0, 100.0], "e": [200.0, 200.0, 100.0] },
                            { "t": 80.0, "s": [200.0, 200.0, 100.0] }
                        ]
                    },
                    "r": { "k": 0.0 },
                    "o": { "k": 100.0 }
                },
                "shapes": [{
                    "ty": "rc",
                    "p": { "k": [0.0, 0.0] },
                    "s": { "k": [20.0, 20.0] },
                    "r": { "k": 0.0 }
                }, {
                    "ty": "fl",
                    "c": { "k": [1.0, 0.0, 0.0, 1.0] },
                    "o": { "k": 100.0 }
                }]
            }]
        }))
        .expect("valid lottie json");

        let mut player = LottiePlayer::new();
        player.load_json(model);

        player.current_frame = 40.0;
        let start_tree = player.render_tree();
        let start_m = first_shape_world_transform(&start_tree)
            .expect("shape at frame 40")
            .to_cols_array();
        let start_scale_x = start_m[0];

        player.current_frame = 70.0;
        let mid_tree = player.render_tree();
        let mid_m = first_shape_world_transform(&mid_tree)
            .expect("shape at frame 70")
            .to_cols_array();
        let mid_scale_x = mid_m[0];

        assert!(
            (start_scale_x - 1.0).abs() < 0.01,
            "expected scale.x == 1.0 at frame 40, got {start_scale_x}"
        );
        assert!(
            mid_scale_x > 1.5,
            "expected animated scale.x > 1.5 at frame 70, got {mid_scale_x}"
        );
    }
}

