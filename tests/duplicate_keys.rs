use rune_cfg::{DiagnosticSeverity, RuneConfig, SchemaDocument};
#[test]
fn duplicate_scalars_fields_and_objects_warn_without_changing_lookup() {
    let config = RuneConfig::from_str(
        "value 1\nvalue 2\nr:\n name \"first\" name \"second\"\nend\nr:\n name \"third\"\nend\n",
    )
    .unwrap();
    assert_eq!(config.get::<f64>("value").unwrap(), 1.0);
    assert_eq!(config.get::<String>("r.name").unwrap(), "first");
    let diagnostics = config.duplicate_key_diagnostics();
    assert_eq!(diagnostics.len(), 3, "{diagnostics:?}");
    assert!(
        diagnostics
            .iter()
            .all(|d| d.severity == DiagnosticSeverity::Warning
                && d.hint.is_some()
                && d.range.is_some())
    );
    assert_eq!(
        diagnostics
            .iter()
            .map(|d| d.range.unwrap().start.line)
            .collect::<Vec<_>>(),
        vec![2, 4, 6]
    );
    assert_eq!(diagnostics[1].range.unwrap().start.column, 15);
    assert!(
        config
            .validate_schema(&SchemaDocument { blocks: vec![] })
            .iter()
            .all(|d| d.severity == DiagnosticSeverity::Error)
    );
}
#[test]
fn different_objects_and_exclusive_branches_are_not_duplicates() {
    let text =
        "r:\n if debug:\n  name \"a\"\n else:\n  name \"b\"\n endif\nend\ns:\n name \"c\"\nend\n";
    assert!(
        RuneConfig::from_str(text)
            .unwrap()
            .duplicate_key_diagnostics()
            .is_empty()
    );
}
#[test]
fn duplicates_inside_a_branch_point_at_the_repeated_key() {
    let text = "r:\n if debug:\n  name \"a\"\n  name \"b\"\n else:\n  name \"c\"\n endif\nend\n";
    let ds = RuneConfig::from_str(text)
        .unwrap()
        .duplicate_key_diagnostics();
    assert_eq!(ds.len(), 1);
    assert_eq!(ds[0].range.unwrap().start.line, 4);
}
#[test]
fn quoted_keys_crlf_and_utf16_ranges() {
    let text = "r:\r\n \"😀\" 1 \"😀\" 2\r\nend\r\n";
    let ds = RuneConfig::from_str(text)
        .unwrap()
        .duplicate_key_diagnostics();
    assert_eq!(ds.len(), 1);
    assert_eq!(ds[0].range.unwrap().start.column, 10);
    assert_eq!(ds[0].range.unwrap().end.column, 12);
}
