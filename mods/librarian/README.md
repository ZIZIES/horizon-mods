# Librarian for Horizon

A native Horizon mod that does what the Fabric mod
[Librarian](https://github.com/VideoGameSmash12/Librarian) does: pages of saved
hotbars. Written from the mod's behaviour and file layout, not from its code.

## What it does

- **Pages.** The creative inventory's Saved Hotbars tab shows one page of nine
  rows; `]` and `[` (rebindable) or the `<` `>` buttons in the tab turn the
  page, as far as you like in either direction. The save and load toolbar keys
  work on the page that is open.
- **The same files.** Page 0 is the game's own `hotbar.nbt`; page `n` is
  `hotbars/hotbar.<n>.nbt` beside it, the Fabric mod's layout. Pages made with
  the Fabric mod open as they are: copy `hotbar.nbt` and the `hotbars/` folder
  from the old `.minecraft` into the instance folder (where `options.txt` is).
- **Names and metadata.** Click the tab's title to rename the page (Enter
  keeps it, Escape or a click elsewhere leaves it). `/librarian meta` sets the
  name, a description and a list of authors; they are kept in the page's file
  under `librarian`, where the Fabric mod keeps them, and survive saves.
- **Backups.** `B` (rebindable), the `B` button or `/librarian backup` copies
  the page's file to `hotbars/backups/` with the date in its name.
- **Overwrite protection.** Saving over a row that holds different items is
  refused once, with a line over the hotbar; the same keys again within five
  seconds save it. Can be turned off in the mod's settings.
- **Commands** (`/lb` is the same): `goto <page>`, `next`, `previous`,
  `backup`, `cache list`, `cache clear`, `meta name|description [set [text]]`,
  `meta authors list|clear|add <name>|remove <name>`, `meta delete`.
- **Settings** (Options > Mods > Librarian): remember the last page, where a
  page change is announced (action bar, toast, both, off), overwrite
  protection, and the three keys.

## Where it differs from the Fabric mod

- The overwrite prompt is a line over the hotbar and a second key press, not
  a confirmation screen.
- Names and descriptions are plain text: a formatted name written by the
  Fabric mod is shown without its colours, and renaming it here stores it
  plain. MiniMessage and `&` colour codes are not read.
- Page numbers are 64-bit, not unbounded.
- No background loading or page cache (Horizon reads the file when the tab
  opens, and it is fast): `cache list` lists the pages that have a file and
  `cache clear` reads the current page again.
- No "newer version" downgrade warning, no file compression option, no
  bookmarks, no add-on API.
- Backup dates are UTC.

## Hooks

Native hook points: `native_mods`, `native_settings`, `native_settings_click`,
`native_key`, `native_commands`, `native_command`, `native_frame`,
`native_screen_render_post`, `native_screen_mouse_button_pressed_pre`,
`native_screen_key_pressed_pre`, `native_screen_char_typed_pre`.

Game functions, so the mod is pinned to Horizon `9f42064` (Nightly
2026-11-13):

- `game::creative_hotbar::Game::refresh_saved_hotbars` and
  `Game::hotbar_load_or_save`: taken over at their head while the mod is on
  (they name `hotbar.nbt` themselves), with the same behaviour on the current
  page's file.
- `game::creative_screen::Game::creative_tab_title`: the hotbar tab's title.

## Checks

`cargo test` (the logic in `mixins/librarian/runtime.rs`), and the bundle
applied to the pinned source with horizon-mixins and `cargo check -p
horizon-client`. It has not been played.
