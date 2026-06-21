use lumin_topology_scanner::scan_file_text;

pub fn assert_risk(file: &str, source: &str, expected: &[&str]) {
    let result = scan_file_text(file, source);
    assert!(!result.ok);
    assert_eq!(
        result
            .risk
            .iter()
            .map(|risk| risk.as_str())
            .collect::<Vec<_>>(),
        expected
    );
    assert!(result.edges.is_empty());
}
