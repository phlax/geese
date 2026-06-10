use std::fs;

use assert_cmd::Command;
use tempfile::tempdir;

fn geese() -> Command {
    Command::cargo_bin("geese").unwrap()
}

#[test]
fn happy_path_uses_geese_root_override() {
    let tempdir = tempdir().unwrap();
    let root = tempdir.path().join("geese-root");

    geese()
        .env("GEESE_ROOT", &root)
        .args(["new", "source"])
        .assert()
        .success();

    geese()
        .env("GEESE_ROOT", &root)
        .args(["list"])
        .assert()
        .success()
        .stdout("source\n");

    let source_path = root.join("profiles").join("source");
    fs::write(
        source_path.join("config").join("config.yaml"),
        "model = \"gpt\"\n",
    )
    .unwrap();
    fs::write(
        source_path.join("config").join("extra.toml"),
        "ignored = true\n",
    )
    .unwrap();
    fs::write(source_path.join("data").join("state.bin"), "do not copy\n").unwrap();

    geese()
        .env("GEESE_ROOT", &root)
        .args(["copy", "source", "target"])
        .assert()
        .success();

    let target_path = root.join("profiles").join("target");
    assert_eq!(
        fs::read_to_string(target_path.join("config").join("config.yaml")).unwrap(),
        "model = \"gpt\"\n"
    );
    assert!(!target_path.join("config").join("extra.toml").exists());
    assert!(!target_path.join("data").join("state.bin").exists());

    geese()
        .env("GEESE_ROOT", &root)
        .args(["lock", "target"])
        .assert()
        .success();

    geese()
        .env("GEESE_ROOT", &root)
        .args(["delete", "target"])
        .assert()
        .failure()
        .stderr("profile is locked: target\n");

    geese()
        .env("GEESE_ROOT", &root)
        .args(["unlock", "target"])
        .assert()
        .success();

    geese()
        .env("GEESE_ROOT", &root)
        .args(["delete", "target"])
        .assert()
        .success();

    geese()
        .env("GEESE_ROOT", &root)
        .args(["list"])
        .assert()
        .success()
        .stdout("source\n");
}
