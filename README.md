# geese

`geese` is a small Rust library and CLI for managing named `GOOSE_PATH_ROOT` profiles for goose.

## Install

```bash
cargo install --path crates/geese
```

## Example

```bash
geese new work-stable
GOOSE_PATH_ROOT="$(geese path work-stable)" goose
```
