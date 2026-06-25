use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use geese_client::{ClientError, GeesedClient, ensure_running};
use nix::{
    sys::signal::{Signal, kill},
    unistd::Pid,
};
use tempfile::tempdir;

// `temp_env::async_with_vars` replaces the hand-rolled `ENV_LOCK`
// Mutex this file used to carry. temp-env wraps the underlying
// `unsafe { std::env::set_var }` (so this test stays inside the
// workspace `unsafe_code = "forbid"` lint), serialises concurrent
// env mutations via its own crate-level mutex, and restores the
// previous env state on drop *even if the test body panics* — which
// the old manual `set_var` / `remove_var` pair did not.

fn spawn_geesed(runtime_root: &Path) -> Child {
    Command::new(assert_cmd::cargo::cargo_bin("geesed"))
        .env("XDG_RUNTIME_DIR", runtime_root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}

fn socket_path(runtime_root: &Path) -> PathBuf {
    runtime_root.join("geese").join("control.sock")
}

fn wait_for_socket(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if path.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }

    panic!("socket did not appear at {}", path.display());
}

#[tokio::test(flavor = "current_thread")]
async fn status_reads_from_real_daemon() {
    let tempdir = tempdir().unwrap();
    let socket = socket_path(tempdir.path());
    let mut daemon = spawn_geesed(tempdir.path());
    wait_for_socket(&socket);

    let mut client = GeesedClient::connect_at(&socket).await.unwrap();
    let status = client.status().await.unwrap();

    assert_eq!(status.pid, daemon.id());
    assert_eq!(status.version, env!("CARGO_PKG_VERSION"));
    assert!(!status.started_at.is_empty());

    kill(Pid::from_raw(daemon.id() as i32), Signal::SIGTERM).unwrap();
    assert!(daemon.wait().unwrap().success());
}

#[tokio::test(flavor = "current_thread")]
async fn connect_at_reports_not_running() {
    let path = PathBuf::from("/nonexistent/path");
    let error = GeesedClient::connect_at(&path).await.unwrap_err();

    assert!(matches!(error, ClientError::NotRunning(actual) if actual == path));
}

#[tokio::test(flavor = "current_thread")]
async fn ensure_running_connects_to_existing_daemon() {
    let tempdir = tempdir().unwrap();
    let socket = socket_path(tempdir.path());
    let mut daemon = spawn_geesed(tempdir.path());
    wait_for_socket(&socket);

    // Point ensure_running at the running daemon via XDG_RUNTIME_DIR.
    let result = temp_env::async_with_vars([("XDG_RUNTIME_DIR", Some(tempdir.path()))], async {
        ensure_running().await
    })
    .await;

    assert!(
        result.is_ok(),
        "ensure_running should connect to running daemon"
    );

    kill(Pid::from_raw(daemon.id() as i32), Signal::SIGTERM).unwrap();
    assert!(daemon.wait().unwrap().success());
}

#[tokio::test(flavor = "current_thread")]
async fn ensure_running_autospawns_daemon() {
    let geesed_bin = assert_cmd::cargo::cargo_bin("geesed");
    if !geesed_bin.exists() {
        eprintln!(
            "skipping ensure_running_autospawns_daemon: geesed binary not found at {}",
            geesed_bin.display()
        );
        return;
    }

    // find_geesed_binary() searches PATH; prepend the dir where cargo placed
    // the geesed binary (target/debug/) so it is found at runtime.
    let geesed_dir = geesed_bin.parent().unwrap();
    let orig_path = std::env::var_os("PATH").unwrap_or_default();
    let mut dirs: Vec<std::path::PathBuf> = vec![geesed_dir.to_path_buf()];
    dirs.extend(std::env::split_paths(&orig_path));
    let new_path = std::env::join_paths(&dirs).unwrap();

    let tempdir = tempdir().unwrap();

    // Use a fresh runtime dir so no daemon is running there, and a
    // PATH that puts the just-built geesed binary first.
    let result = temp_env::async_with_vars(
        [
            ("XDG_RUNTIME_DIR", Some(tempdir.path().as_os_str())),
            ("PATH", Some(new_path.as_os_str())),
        ],
        async { ensure_running().await },
    )
    .await;

    assert!(
        result.is_ok(),
        "ensure_running should autospawn geesed: {:?}",
        result.err()
    );

    // Clean up: kill the autospawned daemon
    let socket = socket_path(tempdir.path());
    if let Ok(mut client) = GeesedClient::connect_at(&socket).await
        && let Ok(status) = client.status().await
    {
        let _ = kill(Pid::from_raw(status.pid as i32), Signal::SIGTERM);
    }
}
