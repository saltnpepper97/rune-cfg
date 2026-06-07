use rune_cfg::{DiagnosticSeverity, RuneConfig, SchemaDocument};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RuneConfig::from_file("examples/schema_config_invalid.rune")?;
    let schema = SchemaDocument::from_file("examples/schema.rune")?;
    let diagnostics = config.validate_schema(&schema);

    if diagnostics.is_empty() {
        println!("config satisfies schema");
        return Ok(());
    }

    for diagnostic in diagnostics {
        let severity = match diagnostic.severity {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Information => "info",
            DiagnosticSeverity::Hint => "hint",
        };

        if let Some(range) = diagnostic.range {
            println!(
                "{}:{}:{}: {}",
                severity, range.start.line, range.start.column, diagnostic.message
            );
        } else {
            println!("{}: {}", severity, diagnostic.message);
        }

        if let Some(hint) = diagnostic.hint {
            println!("  hint: {}", hint);
        }
    }

    Ok(())
}
