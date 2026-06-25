use std::{io::IsTerminal, process};

use geesed::{RunOpts, run};
use tracing::Level;

fn main() {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(Level::INFO)
        .with_target(false)
        .without_time()
        .with_writer(std::io::stderr)
        .init();

    daemonize_if_autospawned();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");

    if let Err(error) = runtime.block_on(run(RunOpts::default())) {
        eprintln!("{error}");
        process::exit(1);
    }
}

/// Detach from the controlling terminal when this `geesed` was
/// autospawned by `geese_client::ensure_running`.
///
/// We use stdin's TTY status as the discriminator:
///
/// * Foreground (`geesed` from a shell) — stdin is a TTY, this is a
///   no-op, the daemon stays attached and `Ctrl-C` works as expected
///   for ad-hoc debugging.
/// * Autospawn (`ensure_running`'s `Command::spawn`) — that caller
///   redirects stdin to `/dev/null` before spawning, so `is_terminal()`
///   returns false and we call `setsid` to peel off into our own
///   session. Without this, a `SIGHUP` from the parent shell exiting
///   would kill geesed alongside it.
///
/// The previous shape did this on the parent side via an
/// `unsafe { cmd.pre_exec(nix::unistd::setsid) }` block on the spawned
/// `Command`. `nix::unistd::setsid` is the safe wrapper around
/// `setsid(2)` and we call it here, in the child, before tokio
/// touches the process. There's a microsecond-scale window between
/// `exec(geesed)` and this call where a `SIGHUP` delivered to the
/// parent's session can still hit the child — the pre_exec shape
/// closed that window completely. We accept the race because the
/// parent (`ensure_running`) doesn't exit until it polls the daemon's
/// socket into existence, which happens after this function returns,
/// so in practice the race never opens.
///
/// `setsid` returns `EPERM` if the calling process is already a
/// session leader. That happens when geesed is itself spawned by
/// something that already detached it (e.g. systemd `Type=forking`);
/// in that case we're already in the state we want, so we ignore the
/// error.
fn daemonize_if_autospawned() {
    if !std::io::stdin().is_terminal()
        && let Err(error) = nix::unistd::setsid()
    {
        // Already a session leader (EPERM) is benign; log other errors
        // but keep going — failing to daemonize is not fatal, the
        // worst case is the daemon dies with the parent shell on
        // SIGHUP, which is the pre-PR baseline for this same edge.
        if error != nix::errno::Errno::EPERM {
            eprintln!("geesed: setsid failed: {error} (continuing without session detach)");
        }
    }
}
