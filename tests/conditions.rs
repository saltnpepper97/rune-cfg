// Author: Dustin Pilgrim
// License: GPL-3.0-or-later

//! How `if` conditions evaluate, inline and in blocks.

use rune_cfg::{RuneConfig, RuneError};

fn load(source: &str) -> RuneConfig {
    RuneConfig::from_str(source).unwrap_or_else(|error| panic!("{source:?} should load: {error}"))
}

/// The value of top-level `r` for a config that ends by setting it.
fn r(source: &str) -> String {
    load(source)
        .get::<String>("r")
        .unwrap_or_else(|error| panic!("{source:?} should set r: {error}"))
}

fn parse_error(source: &str) -> RuneError {
    match RuneConfig::from_str(source) {
        Ok(_) => panic!("{source:?} should not parse"),
        Err(error) => error,
    }
}

#[test]
fn bare_condition_is_false_only_for_false_null_and_missing() {
    let cases = [
        ("f true", "yes"),
        ("f false", "no"),
        ("f None", "no"),
        ("f null", "no"),
        ("", "no"),
        ("f 0", "yes"),
        ("f \"\"", "yes"),
        ("f \"false\"", "yes"),
        ("f []", "yes"),
        ("f:\nend", "yes"),
    ];

    for (definition, expected) in cases {
        let source = format!("{definition}\nr if f \"yes\" else \"no\"\n");
        assert_eq!(r(&source), expected, "{source:?}");
    }
}

#[test]
fn bare_condition_in_a_block_if_takes_the_else_branch_when_false() {
    let config = load(
        "debug false\n\nserver:\n  if debug:\n    ping 8080\n  else:\n    ping 443\n  end\nend\n",
    );
    assert_eq!(config.get::<u16>("server.ping").unwrap(), 443);

    // The reported case, inline.
    for (debug, port) in [("true", 8080), ("false", 443)] {
        let config = load(&format!(
            "debug {debug}\n\nserver:\n  ping if debug 8080 else 443\nend\n"
        ));
        assert_eq!(
            config.get::<u16>("server.ping").unwrap(),
            port,
            "debug {debug}"
        );
    }
}

#[test]
fn elseif_chains_use_truthiness_too() {
    let source = "a false\nb None\nc 0\nr \"none\"\napp:\n  if a:\n    r \"a\"\n  elseif b:\n    r \"b\"\n  elseif c:\n    r \"c\"\n  else:\n    r \"else\"\n  end\nend\n";
    assert_eq!(load(source).get::<String>("app.r").unwrap(), "c");
}

#[test]
fn not_negates_any_condition() {
    assert_eq!(r("f false\nr if not f \"yes\" else \"no\"\n"), "yes");
    assert_eq!(r("f true\nr if not f \"yes\" else \"no\"\n"), "no");
    assert_eq!(r("r if not missing \"yes\" else \"no\"\n"), "yes");
    assert_eq!(r("f true\nr if not not f \"yes\" else \"no\"\n"), "yes");
    assert_eq!(
        r("e \"dev\"\nr if not e = \"prod\" \"yes\" else \"no\"\n"),
        "yes"
    );
    assert_eq!(
        r("e \"qa\"\nr if not e in [\"dev\", \"qa\"] \"yes\" else \"no\"\n"),
        "no"
    );

    let config = load("debug false\napp:\n  if not debug:\n    level \"info\"\n  end\nend\n");
    assert_eq!(config.get::<String>("app.level").unwrap(), "info");
}

#[test]
fn equality_and_inequality_compare_typed_values() {
    let cases = [
        ("e \"prod\"", "e = \"prod\"", "yes"),
        ("e \"dev\"", "e = \"prod\"", "no"),
        ("e \"dev\"", "e != \"prod\"", "yes"),
        ("e \"prod\"", "e != \"prod\"", "no"),
        ("n 8", "n = 8", "yes"),
        ("n 8", "n = \"8\"", "no"),
        ("n 8", "n != 9", "yes"),
        ("f false", "f = false", "yes"),
        ("f false", "f != true", "yes"),
        ("f None", "f = None", "yes"),
        // A missing key reads as null.
        ("", "missing = None", "yes"),
        ("", "missing != \"prod\"", "yes"),
        ("", "missing = \"prod\"", "no"),
    ];

    for (definition, condition, expected) in cases {
        let source = format!("{definition}\nr if {condition} \"yes\" else \"no\"\n");
        assert_eq!(r(&source), expected, "{source:?}");
    }
}

#[test]
fn comparisons_resolve_references_on_both_sides() {
    assert_eq!(
        r("mode \"prod\"\ndefault_mode \"prod\"\nr if mode = default_mode \"yes\" else \"no\"\n"),
        "yes"
    );
    assert_eq!(
        r(
            "mode \"dev\"\nfallback \"dev\"\nr if mode in [\"prod\", fallback] \"yes\" else \"no\"\n"
        ),
        "yes"
    );
}

#[test]
fn in_tests_membership_in_a_list() {
    let cases = [
        ("e \"staging\"", "yes"),
        ("e \"dev\"", "no"),
        ("e 3", "yes"),
        ("", "no"),
    ];

    for (definition, expected) in cases {
        let source =
            format!("{definition}\nr if e in [\"prod\", \"staging\", 3] \"yes\" else \"no\"\n");
        assert_eq!(r(&source), expected, "{source:?}");
    }

    let config = load(
        "environment \"staging\"\napp:\n  if environment in [\"prod\", \"staging\"]:\n    workers 8\n  else:\n    workers 2\n  end\nend\n",
    );
    assert_eq!(config.get::<u32>("app.workers").unwrap(), 8);
}

#[test]
fn dotted_paths_reach_into_objects() {
    let source = "debug true\napp:\n  debug false\n  level if debug \"trace\" else \"info\"\n  mode if app.debug \"trace\" else \"info\"\nend\n";
    let config = load(source);

    // A bare name is the top-level key, as in a value reference.
    assert_eq!(config.get::<String>("app.level").unwrap(), "trace");
    assert_eq!(config.get::<String>("app.mode").unwrap(), "info");

    assert_eq!(
        r(
            "app:\n  server:\n    tls:\n      verify false\n    end\n  end\nend\nr if app.server.tls.verify \"yes\" else \"no\"\n"
        ),
        "no"
    );
    assert_eq!(
        r("app:\n  name \"rune\"\nend\nr if app.name = \"rune\" \"yes\" else \"no\"\n"),
        "yes"
    );
    assert_eq!(r("app:\nend\nr if app.missing \"yes\" else \"no\"\n"), "no");
}

#[test]
fn paths_look_inside_the_active_if_block_branch() {
    let source = "prod false\napp:\n  if prod:\n    debug false\n  else:\n    debug true\n  end\nend\nr if app.debug \"yes\" else \"no\"\nlevel app.debug\n";
    let config = load(source);

    assert_eq!(config.get::<String>("r").unwrap(), "yes");
    // Value references see the same branch.
    assert!(config.get::<bool>("level").unwrap());
}

#[test]
fn conditions_see_resolved_values() {
    // A key set by another conditional.
    assert_eq!(
        r("prod true\ndebug if prod false else true\nr if debug \"yes\" else \"no\"\n"),
        "no"
    );
    // A key that is a reference to a false value.
    assert_eq!(
        r("base false\ndebug base\nr if debug \"yes\" else \"no\"\n"),
        "no"
    );
}

#[test]
fn namespaced_paths_read_the_environment_and_system() {
    assert_eq!(
        r("r if sys.os = \"never-an-os\" \"yes\" else \"no\"\n"),
        "no"
    );
    assert_eq!(r("r if sys.os \"yes\" else \"no\"\n"), "yes");
    assert_eq!(r("r if $sys.os \"yes\" else \"no\"\n"), "yes");
    assert_eq!(
        r("r if $sys.os != \"never-an-os\" \"yes\" else \"no\"\n"),
        "yes"
    );

    // PATH is set wherever tests run; the other variable is never set.
    assert_eq!(r("r if env.PATH \"yes\" else \"no\"\n"), "yes");
    assert_eq!(r("r if $env.PATH \"yes\" else \"no\"\n"), "yes");
    assert_eq!(
        r("r if $env.RUNE_CFG_TEST_SURELY_UNSET_VARIABLE \"yes\" else \"no\"\n"),
        "no"
    );

    assert_eq!(
        r("debug false\nr if $var.debug \"yes\" else \"no\"\n"),
        "no"
    );
    assert_eq!(r("debug true\nr if var.debug \"yes\" else \"no\"\n"), "yes");
}

#[test]
fn keys_named_not_and_in_still_work() {
    assert_eq!(r("not false\nr if not \"yes\" else \"no\"\n"), "no");
    assert_eq!(
        r("not \"x\"\nr if not = \"x\" \"yes\" else \"no\"\n"),
        "yes"
    );
    assert_eq!(r("in true\nr if in \"yes\" else \"no\"\n"), "yes");
}

#[test]
fn malformed_conditions_are_syntax_errors() {
    for source in [
        "e \"x\"\nr if e in \"x\" \"yes\" else \"no\"\n",
        "r if $nope.x \"yes\" else \"no\"\n",
        "app:\nend\nr if app. \"yes\" else \"no\"\n",
        "r if 5 \"yes\" else \"no\"\n",
        "r if e ! \"x\" \"yes\" else \"no\"\n",
    ] {
        let error = parse_error(source);
        assert!(
            !matches!(error, RuneError::RuntimeError { .. }),
            "{source:?} should fail to parse, got {error}"
        );
    }
}

#[test]
fn reference_cycles_are_errors_not_crashes() {
    for (source, key) in [
        // Two keys referring to each other.
        ("a b\nb a\n", "a"),
        // A condition reading the key it sets.
        ("a if a 1 else 2\n", "a"),
        // An if-block reading a key only that same block can set.
        ("app:\n  if app.x:\n    x true\n  end\nend\n", "app.x"),
    ] {
        let result = load(source).get::<String>(key);
        assert!(
            matches!(
                result,
                Err(RuneError::RuntimeError {
                    code: Some(310),
                    ..
                })
            ),
            "{source:?} should report a reference cycle, got {result:?}"
        );
    }
}
