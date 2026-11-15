# horizon-mods

Native mods for [Horizon](https://github.com/horizon-rust/horizon) (the Rust
rewrite of Minecraft 26.3), mostly rewrites of Fabric mods I used on Java.
Each one is a horizon-mixins bundle pinned to Horizon `9f42064`.

Status is "attempted" unless it says otherwise: they compile into the game,
and the notes say what has actually been played.

| mod | what it does | status |
| --- | --- | --- |
| `librarian` | extra saved hotbar pages | played, works |
| `confirm_reset_keys` | confirm before resetting keybinds | played, works |
| `force_close_loading_screen` | terrain loading screen closes at once | played, works |
| `better_selection` | better text cursor/selection in edit boxes | played, works |
| `redaction` | blackbar hotbar, hand FOV, last-server button, inventory snow | played, works |
| `better_stats` | search box in Statistics | works for Items and Mobs, not General |
| `black_bar_concealer` | fixes skins drawn for the wrong arm model | untested |
| `tiertagger` | PvP tiers on name tags / tab list | broken: tiers don't show |
| `tree_feller` | sneak-break a log to fell and replant the tree | not played yet |

## Layout

- `mods/<id>/` — source: `mixins/<id>/horizon-mixin.toml`, `runtime.rs`
  (std-only logic with tests), one helper `.rs` per injection, and a README.
- `attempted/` — the same mods packed as `.hnmod`.
- `tools/check.sh` — stages every mod onto a pristine Horizon checkout and
  `cargo check`s the game. `tools/package.py <id>` packs one.

## Building

Native mods run with full access to your computer; read the source first.

```
horizon-launcher native-build <dir with the bundles> --instance <instance> --trust-native-code
```

`tiertagger` sends the UUIDs of players you see to mctiers.net / subtiers.net.
