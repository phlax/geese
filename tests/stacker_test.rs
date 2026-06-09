#[allow(dead_code)]
#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/error.rs"]
mod error;
#[allow(dead_code)]
#[path = "../src/paths.rs"]
mod paths;
#[allow(dead_code)]
#[path = "../src/stacker.rs"]
mod stacker;

use config::Config;
use stacker::{stack_after_launch, StackOptions};

#[test]
fn config_stack_fields_round_trip() {
    let yaml = r#"
stack: true
stack_delay_ms: 5000
profiles:
  work: {}
"#;

    let config = Config::from_yaml_str(yaml).expect("config should parse");
    assert!(config.stack);
    assert_eq!(config.stack_delay_ms, 5000);

    let emitted = serde_yaml::to_string(&config).expect("config should serialize");
    let reparsed = Config::from_yaml_str(&emitted).expect("config should round-trip");
    assert_eq!(config, reparsed);
}

#[test]
fn config_stack_defaults_to_false() {
    let yaml = r#"
profiles:
  work: {}
"#;

    let config = Config::from_yaml_str(yaml).expect("config should parse");
    assert!(!config.stack);
    assert_eq!(config.stack_delay_ms, 3000);
}

#[test]
fn no_stack_overrides_config_stack_true() {
    // Simulate: config has stack: true, but --no-stack is passed.
    let stack_opts = StackOptions {
        enabled: false, // --no-stack wins
        delay_ms: 3000,
        verbose: false,
    };
    // profile_count >= 2, but enabled=false → should return Ok immediately
    // (without needing wtype on $PATH)
    let result = stack_after_launch(3, &stack_opts);
    assert!(result.is_ok(), "no-stack should always succeed: {result:?}");
}

#[test]
fn single_profile_short_circuits_without_wtype() {
    // Even with enabled=true and no wtype present, profile_count < 2 → Ok(()).
    let stack_opts = StackOptions {
        enabled: true,
        delay_ms: 0,
        verbose: false,
    };
    let result = stack_after_launch(1, &stack_opts);
    assert!(
        result.is_ok(),
        "single profile should be a no-op: {result:?}"
    );
}

#[test]
fn non_wayland_env_short_circuits_without_wtype() {
    // Clear WAYLAND_DISPLAY to simulate a non-Wayland (X11) session.
    // With profile_count >= 2 and no WAYLAND_DISPLAY, stacker should return Ok(())
    // with a warning instead of erroring about wtype.
    let prev_wayland = std::env::var_os("WAYLAND_DISPLAY");
    std::env::remove_var("WAYLAND_DISPLAY");

    let stack_opts = StackOptions {
        enabled: true,
        delay_ms: 0,
        verbose: false,
    };
    let result = stack_after_launch(3, &stack_opts);

    // Restore env var before asserting so we don't leave env dirty.
    if let Some(val) = prev_wayland {
        std::env::set_var("WAYLAND_DISPLAY", val);
    }

    assert!(
        result.is_ok(),
        "non-Wayland session should be a no-op, not an error: {result:?}"
    );
}
