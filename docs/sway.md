# sway

```ini
# Send every geese-launched goose to a dedicated workspace and tab them
set $ws_goose "9: goose"
for_window [app_id="^goose-"] move container to workspace $ws_goose
workspace $ws_goose layout tabbed
```
