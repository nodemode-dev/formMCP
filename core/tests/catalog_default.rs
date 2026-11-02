use formmcp_core::catalog::DEFAULT_CATALOG;

#[test]
fn the_default_catalogue_is_the_public_download_store() {
    assert_eq!(DEFAULT_CATALOG, "https://dl.formai.studio/catalog/apps.json");
    assert_eq!(format!("{DEFAULT_CATALOG}.sig"), "https://dl.formai.studio/catalog/apps.json.sig");
    assert!(!DEFAULT_CATALOG.contains("github.com"), "the private release page never serves the public");
}
