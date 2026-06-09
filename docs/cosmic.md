# COSMIC

COSMIC supports native stacked windows. Focus the windows spawned by `geese`, then use **Super+S** to stack the focused windows.

As of 2026-06, COSMIC does not ship `for_window`-style rules, so auto-stacking by `app_id` is not possible yet. Tracking issue: <https://github.com/pop-os/cosmic-comp/issues/902>.

Workflow:

1. Run `geese`
2. Focus the spawned Goose windows
3. Stack them with the shortcut shown in **Settings → Input → Keyboard Shortcuts**

When COSMIC adds matching rules, the rule should target the `app_id` regex `^goose-`.
