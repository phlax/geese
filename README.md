# geese

`geese` is a small Rust CLI that launches multiple isolated [Goose](https://github.com/aaif-goose/goose) desktop profiles, gives each one a distinct Wayland `app_id`, and leaves tabbing or stacking to your compositor.

## Install

```bash
cargo install --path .
```

`geese` is not on crates.io yet.

## Quick start

1. Copy `config.example.yml` to `~/.config/geese/config.yml`
2. Edit the profiles, arguments, and environment variables to match your setup
3. Run `geese`

## Commands

| Command | Behavior |
| --- | --- |
| `geese` | Launch all configured profiles |
| `geese --get-gander` | Alias for `launch-all` |
| `geese launch-all` | Launch all configured profiles |
| `geese launch <name>` | Launch one named profile |
| `geese list` | Print configured profiles with their app IDs, data directories, and resolved binaries |
| `geese paths` | Print the resolved config file path and data root |
| `geese --foreground` / `-f` | Keep the parent attached, forward `SIGINT`/`SIGTERM`, and wait for launched children |
| `geese --verbose` / `-v` | Print resolved paths, command lines, and environment differences |
| `geese --help` / `--version` | Standard clap output |

## Compositor setup

See `docs/README.md` for per-compositor tabbing and stacking notes:

- `docs/cosmic.md`
- `docs/sway.md`
- `docs/hyprland.md`
- `docs/kwin.md`

## Limitations

- `geese` does not embed Goose windows. Tabbing or stacking is done by your compositor, because Wayland clients cannot reparent other clients' surfaces by design.
- The symlink and `argv[0]` trick relies on Chromium or Electron deriving the Wayland `app_id` from `argv[0]`. If Goose ever sets `app_id` explicitly, the workaround would need an upstream Goose change instead.
- `geese` sets both `GOOSE_CONFIG_DIR` and the four `XDG_*` variables so profile isolation still works if only one of those mechanisms is respected upstream.
- On X11 sessions, `geese` appends `--class=goose-<name>` so `WM_CLASS` is distinct too. Chromium ignores that flag on Wayland.

## Credit

`geese` exists to make [Goose](https://github.com/aaif-goose/goose) profile launching less awkward on Linux desktops.
