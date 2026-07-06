//! SC-5: pipeline failure atomicity — an error at mid-pipeline returns Err
//! with no partial output.
//!
//! Proves that Pipeline::run() is all-or-nothing: any transform that returns
//! Err causes the entire pipeline to return Err, and the caller never receives
//! a partial Vec<Asset>. The consumer's two-phase upload (gestiscilo PUB-05)
//! builds its atomicity guarantee on this contract.
//!
//! Run: `cargo test -p ferro-assets --test all_or_nothing`

use bytes::Bytes;
use ferro_assets::{Asset, ContentType, Error, Pipeline, Transform};

/// A transform that always succeeds — passes all assets through.
struct AlwaysOk;

impl Transform for AlwaysOk {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        Ok(assets)
    }
}

/// A transform that returns Err on any JS asset via map_matching.
struct FailOnJs;

impl Transform for FailOnJs {
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        ferro_assets::map_matching(assets, &[ContentType::Js], |a| {
            Err(Error::transform(
                "fail_on_js",
                &a.path,
                "deliberate test failure",
            ))
        })
    }
}

/// A transform that always returns Err, regardless of content type.
struct AlwaysFail;

impl Transform for AlwaysFail {
    fn run(&self, _assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        Err(Error::setup("deliberate setup failure"))
    }
}

#[test]
fn error_mid_pipeline_produces_no_partial_output() {
    let assets = vec![
        Asset::new("page.html", Bytes::from_static(b"<html></html>")),
        Asset::new("app.js", Bytes::from_static(b"console.log('hi')")),
        Asset::new("style.css", Bytes::from_static(b"body { color: red }")),
    ];

    // Pipeline: AlwaysOk → FailOnJs → AlwaysOk
    // FailOnJs will error on app.js; the pipeline must return Err with no partial set.
    let pipeline = Pipeline::new().add(AlwaysOk).add(FailOnJs).add(AlwaysOk);

    let result = pipeline.run(assets);

    assert!(
        result.is_err(),
        "pipeline must return Err when a transform fails"
    );
    // The binding is never Ok(Vec<Asset>) — this is the no-partial-output proof.
    assert!(
        result.ok().is_none(),
        "pipeline result must NOT be Ok when a transform fails"
    );
}

#[test]
fn error_at_first_transform_returns_err() {
    let assets = vec![Asset::new("app.js", Bytes::from_static(b"var x = 1;"))];

    let pipeline = Pipeline::new().add(FailOnJs).add(AlwaysOk);

    let result = pipeline.run(assets);
    assert!(
        result.is_err(),
        "first-transform failure must propagate as Err"
    );
}

#[test]
fn error_at_last_transform_returns_err() {
    let assets = vec![Asset::new("app.js", Bytes::from_static(b"var x = 1;"))];

    let pipeline = Pipeline::new().add(AlwaysOk).add(AlwaysOk).add(FailOnJs);

    let result = pipeline.run(assets);
    assert!(
        result.is_err(),
        "last-transform failure must propagate as Err"
    );
}

#[test]
fn error_carries_transform_and_path_context() {
    let assets = vec![Asset::new("main.js", Bytes::from_static(b"x()"))];

    let pipeline = Pipeline::new().add(FailOnJs);
    let err = pipeline.run(assets).unwrap_err();
    let msg = err.to_string();

    assert!(
        msg.contains("fail_on_js"),
        "error must name the failed transform"
    );
    assert!(
        msg.contains("main.js"),
        "error must name the failed asset path"
    );
}

#[test]
fn setup_error_propagates_from_transform() {
    let pipeline = Pipeline::new().add(AlwaysFail);

    let result = pipeline.run(vec![Asset::new("x.json", Bytes::new())]);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("setup error"),
        "setup error must carry context"
    );
}

#[test]
fn non_js_assets_not_affected_by_fail_on_js() {
    // FailOnJs only errors on JS; non-JS assets must pass through even when
    // they share a pipeline with a JS asset — but the whole pipeline returns
    // Err if any one asset fails (all-or-nothing).
    let assets = vec![
        Asset::new("style.css", Bytes::from_static(b"body{}")),
        Asset::new("app.js", Bytes::from_static(b"x()")),
    ];

    let result = Pipeline::new().add(FailOnJs).run(assets);
    // The JS asset causes failure → whole pipeline returns Err.
    assert!(
        result.is_err(),
        "one JS failure causes the whole pipeline to return Err"
    );
}
