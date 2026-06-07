//! SC-1: passthrough proof — a JSON (ContentType::Other) file run through a
//! pipeline of always-passthrough transforms exits byte-identical.
//!
//! Proves the crate's core contract: files whose ContentType is not in a
//! transform's accepted set are never touched. The pipeline returns the same
//! bytes it received.
//!
//! Includes the real-transform version (criterion 1): a JSON file through all
//! seven built-in transforms exits byte-identical. This is the artifact-agnostic
//! guarantee with the actual production transforms, not local stubs.
//!
//! Run: `cargo test -p ferro-assets --test passthrough_proof`

use bytes::Bytes;
use ferro_assets::transforms::{
    CssMinify, HtmlMinify, ImageTranscode, InjectBeforeTag, JsMinify, ReplaceTokens,
    ResponsiveImages,
};
use ferro_assets::{Asset, ContentType, Error, Pipeline, Transform};
use std::collections::HashMap;

/// A transform that only accepts HTML. Everything else passes through.
struct HtmlOnlyNoOp;

impl Transform for HtmlOnlyNoOp {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        ferro_assets::map_matching(assets, &[ContentType::Html], Ok)
    }
}

/// A transform that only accepts CSS. Everything else passes through.
struct CssOnlyNoOp;

impl Transform for CssOnlyNoOp {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        ferro_assets::map_matching(assets, &[ContentType::Css], Ok)
    }
}

/// A transform that only accepts JS. Everything else passes through.
struct JsOnlyNoOp;

impl Transform for JsOnlyNoOp {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        ferro_assets::map_matching(assets, &[ContentType::Js], Ok)
    }
}

/// A transform that only accepts images. Everything else passes through.
struct ImageOnlyNoOp;

impl Transform for ImageOnlyNoOp {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        ferro_assets::map_matching(
            assets,
            &[ContentType::Jpeg, ContentType::Png, ContentType::Avif],
            Ok,
        )
    }
}

/// A passthrough that accepts ALL content types — used to test that even a
/// no-op full-coverage transform does not alter bytes.
struct FullPassThrough;

impl Transform for FullPassThrough {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        Ok(assets)
    }
}

#[test]
fn json_file_passes_through_type_gated_pipeline_byte_identical() {
    let json_bytes = Bytes::from_static(br#"{"intent":"browse","fields":[]}"#);
    let assets = vec![Asset::new("spec.json", json_bytes.clone())];

    // Pipeline of transforms that each only accept specific content types.
    // A JSON file (ContentType::Other) must pass all of them unchanged.
    let pipeline = Pipeline::new()
        .add(HtmlOnlyNoOp)
        .add(CssOnlyNoOp)
        .add(JsOnlyNoOp)
        .add(ImageOnlyNoOp);

    let result = pipeline
        .run(assets)
        .expect("pipeline must succeed on JSON input");

    assert_eq!(result.len(), 1, "output must still have exactly one asset");
    assert_eq!(
        result[0].bytes, json_bytes,
        "JSON bytes must be byte-identical after passing through all transforms"
    );
    assert_eq!(
        result[0].content_type,
        ContentType::Other,
        "content type must remain Other"
    );
}

#[test]
fn other_type_asset_passes_through_full_coverage_transform() {
    let data = Bytes::from_static(b"arbitrary binary data \x00\x01\x02");
    let asset = Asset::new("data.bin", data.clone());

    let pipeline = Pipeline::new().add(FullPassThrough);

    let result = pipeline.run(vec![asset]).expect("pipeline must succeed");
    assert_eq!(result[0].bytes, data, "binary data must be byte-identical");
}

#[test]
fn empty_pipeline_returns_assets_unchanged() {
    let bytes = Bytes::from_static(b"untouched");
    let assets = vec![Asset::new("file.json", bytes.clone())];

    let result = Pipeline::new()
        .run(assets)
        .expect("empty pipeline must succeed");

    assert_eq!(result[0].bytes, bytes);
}

#[test]
fn pipeline_applies_transforms_in_insertion_order() {
    // Track order by appending a byte in each transform.
    use std::sync::{Arc, Mutex};

    let log: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(vec![]));

    struct OrderProbe {
        id: u8,
        log: Arc<Mutex<Vec<u8>>>,
    }

    impl Transform for OrderProbe {
        fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
            self.log.lock().unwrap().push(self.id);
            Ok(assets)
        }
    }

    let pipeline = Pipeline::new()
        .add(OrderProbe {
            id: 1,
            log: log.clone(),
        })
        .add(OrderProbe {
            id: 2,
            log: log.clone(),
        })
        .add(OrderProbe {
            id: 3,
            log: log.clone(),
        });

    pipeline
        .run(vec![Asset::new("x.json", Bytes::new())])
        .unwrap();

    assert_eq!(
        *log.lock().unwrap(),
        vec![1, 2, 3],
        "transforms must run in insertion order"
    );
}

/// SC-1 (real-transform version, criterion 1): a JSON file run through all
/// seven production built-in transforms exits byte-identical.
///
/// This is the artifact-agnostic guarantee: ContentType::Other files are
/// never touched by any built-in transform, regardless of which transforms
/// are in the pipeline.
#[test]
fn json_file_unchanged_by_all_seven_real_transforms() {
    let json_bytes = Bytes::from_static(br#"{"intent":"browse","fields":[],"version":"1.0"}"#);
    let assets = vec![Asset::new("spec.json", json_bytes.clone())];

    let pipeline = Pipeline::new()
        .add(HtmlMinify::new())
        .add(CssMinify::new())
        .add(JsMinify::new())
        .add(ImageTranscode::new())
        .add(ResponsiveImages::new())
        .add(InjectBeforeTag::new("</body>", "<script></script>"))
        .add(ReplaceTokens::new(HashMap::new()));

    let result = pipeline
        .run(assets)
        .expect("full real-transform pipeline must succeed on JSON input");

    assert_eq!(
        result.len(),
        1,
        "output must have exactly one asset (no variants emitted for non-image)"
    );
    assert_eq!(
        result[0].bytes, json_bytes,
        "JSON bytes must be byte-identical after all seven real transforms"
    );
    assert_eq!(
        result[0].content_type,
        ContentType::Other,
        "content type must remain Other after all seven real transforms"
    );
}
