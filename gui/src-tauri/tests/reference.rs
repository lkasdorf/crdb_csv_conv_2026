use std::path::Path;

#[test]
fn converts_reference_statement_byte_exact() {
    let xls = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../example/202601_Statement_TZS.xls");
    let expected_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../example/202601_Statement_TZS_ref99_trim.csv");

    let out = tempfile::NamedTempFile::new().unwrap();
    let result =
        crdb_csv_gui_lib::converter::convert_xls_to_csv(&xls, out.path()).unwrap();

    assert!(result.warnings.is_empty(), "warnings: {:?}", result.warnings);
    assert!(result.rows > 0);

    let produced = std::fs::read(out.path()).unwrap();
    let expected = std::fs::read(&expected_path).unwrap();
    assert_eq!(
        produced, expected,
        "output differs from reference CSV (byte comparison)"
    );
}
