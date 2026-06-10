# geese design

## Storage layout

`geese` stores profiles under `$GEESE_ROOT/profiles`, where `GEESE_ROOT` defaults to `$XDG_DATA_HOME/geese` (typically `~/.local/share/geese`).

Each profile directory is a complete `GOOSE_PATH_ROOT`:

```text
$GEESE_ROOT/
└── profiles/
    └── <name>/
        ├── profile.toml
        ├── config/
        ├── data/
        └── state/
```

`geese` creates the empty `config/`, `data/`, and `state/` directories and only manages `profile.toml`. goose owns the contents underneath those directories.

## profile.toml

Version 0 metadata is:

```toml
name = "work-stable"
locked = false
parent = "default" # optional
```

`parent` is only recorded for profiles created with `geese copy`.

## goose integration

`geese path <name>` prints the profile directory, and `geese launch <name> -- ...` executes `goose` with `GOOSE_PATH_ROOT` set to that directory so goose uses that profile's `config/`, `data/`, and `state/` trees.
