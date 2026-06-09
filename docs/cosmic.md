# COSMIC

`geese` ships an opt-in `--stack` mode that *attempts* to automatically combine the launched windows into a single COSMIC stack. It works often enough to be useful but has real caveats — read this whole page before relying on it.

## TL;DR

```
geese --stack
```

Add `stack: true` to `~/.config/geese/config.yml` to make it the default.

## How `--stack` works under the hood

COSMIC does not currently expose an IPC socket (cf. [cosmic-comp#280](https://github.com/pop-os/cosmic-comp/issues/280)) nor `for_window`-style rules (cf. [cosmic-comp#902](https://github.com/pop-os/cosmic-comp/issues/902)), so we can't ask the compositor to stack windows directly. Instead `geese --stack`:

1. Launches every profile as normal.
2. Sleeps `--stack-delay` ms (default 3000) to give the Electron windows time to map.
3. Uses [`wtype`](https://github.com/atx/wtype) to synthesise the **Super+S** keybinding (start/extend a COSMIC stack) once per launched window, with **Super+Tab** between presses to cycle focus.

That's the entire mechanism. It is keystroke automation, not real window management.

## Requirements

- A Wayland session (verified via `$WAYLAND_DISPLAY`). On X11 sessions `--stack` is a no-op with a warning.
- `wtype` installed and on `$PATH`. COSMIC supports the `virtual-keyboard-unstable-v1` Wayland protocol `wtype` needs.
- The workspace where the windows land must be in **tiling** mode. Super+S on a floating window is a no-op in COSMIC. Toggle with **Super+Y**.
- The default **Super+S** binding for *Stack Windows* and the default **Super+Tab** binding for *Switch Window* must still be in place. If you have rebound them, `--stack` will silently do the wrong thing. Check under Settings → Input → Keyboard Shortcuts → Window Management.

## Known failure modes

- **Race**: if the windows take longer than `--stack-delay` to appear, the keystrokes hit your terminal (or worse). Increase `--stack-delay` if you see this.
- **Focus theft**: if you focus another window during the delay, the keystrokes go there. Don't touch the keyboard for a few seconds after running `geese --stack`.
- **Workspace not tiling**: Super+S is silently ignored on floating workspaces. Toggle with Super+Y first.
- **One launch fails**: only successfully-launched profiles are counted; you may end up with fewer windows in the stack than expected. Re-run with `--verbose` to see which profile failed.
- **Profile count == 1**: nothing to stack, `--stack` is a no-op.

## Why this isn't more robust

A proper implementation would:

1. Bind the `ext-foreign-toplevel-list-v1` Wayland protocol to *wait* for exactly N windows with the expected app_id to appear, instead of sleeping.
2. Use `zcosmic-toplevel-management-v1` (COSMIC's window-control protocol) to focus each window explicitly, instead of relying on Super+Tab cycling through the right ones in the right order.
3. Still call the keybinding for stacking, because the actual "stack these toplevels" action is not exposed via any Wayland protocol — that's a COSMIC shell affair.

(1) and (2) are not yet implemented; tracked as future work. PRs welcome.

## Wayland app_id caveat

All Goose windows have `app_id="goose-app"` on Wayland, regardless of the per-profile symlink. Electron derives the Wayland app_id from `app.getName()` (which Goose hard-codes in `ui/desktop/package.json`), not from `argv[0]`. The symlink trick only affects `WM_CLASS` on X11. Even when COSMIC eventually ships matching rules, you won't be able to write `^goose-<name>` rules without an upstream Goose change.

## Fully manual workflow

If you'd rather not deal with `--stack`:

1. Run `geese` (no flags).
2. Make sure the workspace is in tiling mode (Super+Y).
3. Focus a Goose window. Press Super+S.
4. Focus the next Goose window. Press Super+S again to add to the stack.
5. Repeat.

Cycle through stacked windows with Super+Tab (or whatever you have bound).

