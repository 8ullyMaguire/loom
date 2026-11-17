#[test]
fn openapi_documents_typed_bundle_routes() {
    let doc = loom_api::openapi::document();
    let json = serde_json::to_value(&doc).unwrap();
    let paths = json["paths"].as_object().expect("paths");
    for p in ["/api/v1/bundles", "/api/v1/bundles/{id}"] {
        assert!(paths.contains_key(p), "missing path {p}");
    }
    let schemas = json["components"]["schemas"].as_object().expect("schemas");
    assert!(schemas.contains_key("BundleDto"));
    assert!(schemas.contains_key("ErrorBody"));
    println!("openapi paths {} schemas {}", paths.len(), schemas.len());
}
