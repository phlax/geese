# geesed v0

`geesed` v0 is deliberately small: it boots, listens on a control socket, answers a `status` ping, and shuts down cleanly. Profile CRUD and ACP routing land in later issues; see phlax/geese#10 for the sequence.

`geese status` can be pointed at a non-default daemon socket by setting `GEESED_SOCKET`.

## Profile CRUD (phlax/geese#16)

Profile CRUD (`profile.list`, `profile.get`, `profile.create`, `profile.delete`, `profile.lock`, `profile.unlock`, `profile.copy`) is now dispatched through `geesed` via JSON-RPC 2.0. The `geese` CLI exposes each verb (`geese list`, `geese new <name>`, `geese path <name>`, `geese delete <name>`, `geese lock <name>`, `geese unlock <name>`, `geese copy <src> <dest>`), all of which call `geese-client::ensure_running()` to autospawn the daemon when it isn't running. The storage root defaults to `$GEESE_ROOT` (or the platform data dir); tests pass an explicit root via `RunOpts::geese_root`. See phlax/geese#16 for the full design.

## Goose process management (phlax/geese#19)

`geesed` gains four new control-socket methods for spawning and tracking `goose acp` child processes. One `goose acp` process is maintained per profile; stdio is captured for future ACP proxy use but is unused in this version.

### Control-socket methods

| Method | Params | Result | Behaviour |
|---|---|---|---|
| `goosed.start` | `{ "name": "<profile>" }` | `{ "pid": <u32> }` | Spawn `goose acp` for the named profile. Idempotent: returns existing pid if already running. |
| `goosed.stop` | `{ "name": "<profile>" }` | `null` | SIGTERM the child; escalates to SIGKILL after 5 s. No-op if not running. |
| `goosed.kill` | `{ "name": "<profile>" }` | `null` | SIGKILL immediately. No-op if not running. |
| `goosed.list_running` | `{}` | `[ { "name", "pid", "started_at" } ]` | Enumerate running goose processes. |

### CLI surface

```
geese start <profile>   # goosed.start
geese stop <profile>    # goosed.stop
geese kill <profile>    # goosed.kill
geese ps                # goosed.list_running
```

### Goose binary resolution

The `goose` binary is resolved once at daemon startup in this order:

1. `GEESE_GOOSE_BIN` env var (absolute path or bare name searched in `PATH`)
2. `which("goose")` from `PATH`
3. `None` — geesed starts normally but `goosed.start` returns `-32010 GooseBinaryUnavailable`

Tests pass an explicit binary via `RunOpts::with_goose_bin(path)`.

