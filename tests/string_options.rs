use rune_cfg::{ParseOptions, RuneConfig, RuneError, Value};

const STRICT: ParseOptions = ParseOptions {
    quoted_references: false,
    strict_env: true,
};
fn command(text: &str, options: ParseOptions) -> Result<String, RuneError> {
    RuneConfig::from_str_with_options(text, options)?.get("r.run")
}

#[test]
fn shell_strings_keep_punctuation_and_shell_syntax() {
    for shell in [
        "echo saved $VEKJA_NAME.",
        "$EDITOR",
        "${X}",
        "$(date)",
        "$?",
        "$1",
        "$$",
        "mv $VEKJA_PATH $VEKJA_PATH.bak",
        "echo $NAME.! $NAME.. $NAME./x",
        "écho $X.",
    ] {
        let text = format!("r:\n run {shell:?}\nend\n");
        assert_eq!(command(&text, STRICT).unwrap(), shell);
    }
}

#[test]
fn legacy_quoted_references_and_bare_references_still_work() {
    let text = "editor \"vim\"\nr:\n run \"$editor\"\nend\n";
    assert_eq!(command(text, ParseOptions::default()).unwrap(), "vim");
    assert_eq!(command(text, STRICT).unwrap(), "$editor");
    assert_eq!(
        command("editor \"vim\"\nr:\n run editor\nend\n", STRICT).unwrap(),
        "vim"
    );
    assert_eq!(
        command("editor \"vim\"\nr:\n run $var.editor\nend\n", STRICT).unwrap(),
        "vim"
    );
}

#[test]
fn punctuation_is_fixed_in_default_mode_too() {
    assert_eq!(
        command("r:\n run \"echo $NAME.\"\nend", ParseOptions::default()).unwrap(),
        "echo $NAME."
    );
}

#[test]
fn strict_unset_env_reports_real_coordinates() {
    let missing = "VEKJA_RUNE_ENV_THAT_MUST_NOT_EXIST_879DA357";
    assert!(std::env::var_os(missing).is_none());
    for value in [
        format!("\"cd $env.{missing}/x\""),
        format!("$env.{missing}"),
    ] {
        let text = format!("# header\nr:\n  run {value}\nend\n");
        let error = RuneConfig::from_str_with_options(&text, STRICT)
            .err()
            .unwrap();
        match error {
            RuneError::SyntaxError {
                line,
                column,
                message,
                hint,
                ..
            } => {
                assert_eq!(line, 3);
                assert_eq!(column, 7);
                assert!(message.contains(missing));
                assert!(hint.unwrap().contains("Set"));
            }
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(
        command(
            &format!("r:\n run \"cd $env.{missing}/x\"\nend"),
            ParseOptions::default()
        )
        .unwrap(),
        "cd /x"
    );
}

#[test]
fn strict_env_preserves_hyphen_suffix() {
    let home = std::env::var("HOME").unwrap();
    assert_eq!(
        command("r:\n run \"cp x $env.HOME-backup\"\nend", STRICT).unwrap(),
        format!("cp x {home}-backup")
    );
    assert_eq!(
        command("r:\n run \"$env.HOME\"\nend", STRICT).unwrap(),
        home
    );
}

#[test]
fn invalid_namespace_paths_have_string_start_coordinates() {
    let error =
        RuneConfig::from_str_with_options("r:\n  run \"prefix $sys.unknown_key\"\nend", STRICT)
            .err()
            .unwrap();
    assert!(matches!(
        error,
        RuneError::SyntaxError {
            line: 2,
            column: 7,
            ..
        }
    ));
}

#[test]
fn options_apply_to_nested_gathers() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("main.rune"),
        "gather \"one.rune\" as one\nr:\n run one.shell\nend",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("one.rune"),
        "gather \"two.rune\" as two\nshell \"$EDITOR\"",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("two.rune"),
        "bad \"$env.VEKJA_RUNE_ENV_THAT_MUST_NOT_EXIST_879DA357\"",
    )
    .unwrap();
    assert!(RuneConfig::from_file_with_options(dir.path().join("main.rune"), STRICT).is_err());
    std::fs::write(dir.path().join("two.rune"), "okay \"hello\"").unwrap();
    let config = RuneConfig::from_file_with_options(dir.path().join("main.rune"), STRICT).unwrap();
    assert_eq!(config.get::<String>("r.run").unwrap(), "$EDITOR");
    assert!(matches!(
        config.get_document("one").unwrap().globals[0].1,
        Value::String(_)
    ));
}

#[test]
fn literal_dollar_escape_in_both_quote_styles_and_arrays() {
    for literal in [
        r#""\$env.HOME""#,
        r#"'\$env.HOME'"#,
        r#""\$var.name""#,
        r#""\$EDITOR""#,
    ] {
        let text = format!("r:\n run {literal}\nend\n");
        let got = command(&text, STRICT).unwrap();
        assert!(got.starts_with('$'), "{got}");
        assert!(!got.contains('\\'), "{got}");
    }
    let home = std::env::var("HOME").unwrap();
    assert_eq!(
        command(
            r#"r:
 run "\$env.HOME $env.HOME"
end"#,
            STRICT
        )
        .unwrap(),
        format!("$env.HOME {home}")
    );
    assert_eq!(
        command(
            r#"r:
 run "\\$env.HOME"
end"#,
            STRICT
        )
        .unwrap(),
        format!("\\{home}")
    );
    let config = RuneConfig::from_str_with_options(
        r#"r:
 run ["\$env.HOME", "é \$var.name", "\$env.VEKJA_RUNE_ENV_THAT_MUST_NOT_EXIST_879DA357"]
end"#,
        STRICT,
    )
    .unwrap();
    assert_eq!(
        config.get::<Vec<String>>("r.run").unwrap(),
        vec![
            "$env.HOME",
            "é $var.name",
            "$env.VEKJA_RUNE_ENV_THAT_MUST_NOT_EXIST_879DA357"
        ]
    );
    assert_eq!(
        command(
            r#"r:
 run "\$env.HOME"
end"#,
            ParseOptions::default()
        )
        .unwrap(),
        home
    );
}

#[test]
fn literal_mode_keeps_var_namespace_in_quotes_but_resolves_bare_var() {
    assert_eq!(
        command("name \"hello\"\nr:\n run \"$var.name\"\nend", STRICT).unwrap(),
        "$var.name"
    );
    assert_eq!(
        command("name \"hello\"\nr:\n run $var.name\nend", STRICT).unwrap(),
        "hello"
    );
}
