// Librarian's side of the game: everything that names a game type. The
// helpers beside this file are one call each into here; the logic with no
// game types in it is `runtime.rs` (`super::librarian`).
use super::librarian as rt;
use super::{Game, Screen};
use horizon_client::{
    hotbar, hud::HudBatch, items::ItemStack, keybinds, mod_api::NativeScreenView, save, text::Text,
};
use horizon_world::nbt::{Compound, JavaString, Tag};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

/// The file's own key for the metadata, as the Fabric mod names it.
const META: &str = "librarian";

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

/// The folder `hotbar.nbt` is in: beside `options.txt`.
fn base() -> PathBuf {
    Game::options_path()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

fn settings(game: &Game<'_>) -> rt::Settings {
    rt::Settings::read(|key| game.native_setting(rt::ID, key))
}

fn string(tag: Option<&Tag>) -> Option<String> {
    match tag {
        Some(Tag::String(s)) => Some(s.to_string_lossy()),
        _ => None,
    }
}

/// The `librarian` compound of the page file at `path`.
fn read_meta(path: &Path) -> rt::Meta {
    let Some(file) = save::read_dat(path) else {
        return rt::Meta::default();
    };
    let Some(meta) = file.get(META) else {
        return rt::Meta::default();
    };
    let text = |key: &str| {
        string(meta.get(key))
            .filter(|s| !s.trim().is_empty())
            .map(|s| rt::component_plain(&s))
    };
    let authors = match meta.get("authors") {
        Some(Tag::List(list)) => list.iter().filter_map(|t| string(Some(t))).collect(),
        _ => Vec::new(),
    };
    rt::Meta {
        name: text("name"),
        description: text("description"),
        authors,
    }
}

/// Puts `meta` in the page file at `path` (and takes the compound out when
/// there is none), leaving the rows as they are. A page with no file yet
/// gets one that holds only the metadata: its rows read as empty.
fn write_meta(path: &Path, meta: &rt::Meta) -> Result<(), String> {
    let mut file = match save::read_dat(path) {
        Some(Tag::Compound(file)) => file,
        _ if meta.is_empty() => return Ok(()),
        _ => {
            let mut file = Compound::new();
            file.insert(
                JavaString::from("DataVersion"),
                Tag::Int(horizon_world::DATA_VERSION),
            );
            file
        }
    };
    if meta.is_empty() {
        if file.remove(&JavaString::from(META)).is_none() {
            return Ok(());
        }
    } else {
        let mut entries = vec![("version", Tag::Int(0))];
        if let Some(name) = &meta.name {
            let name = rt::plain_component(name);
            entries.push(("name", Tag::String(name.as_str().into())));
        }
        if let Some(description) = &meta.description {
            let description = rt::plain_component(description);
            entries.push(("description", Tag::String(description.as_str().into())));
        }
        if !meta.authors.is_empty() {
            let authors = meta.authors.iter().map(|a| Tag::String(a.as_str().into()));
            entries.push(("authors", Tag::List(authors.collect())));
        }
        file.insert(JavaString::from(META), Tag::compound(entries));
    }
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    }
    save::write_dat(path, Tag::Compound(file)).map_err(|e| e.to_string())
}

/// Reads the page to start on and its metadata, once.
fn ready(game: &Game<'_>) {
    if rt::with(|s| s.ready) {
        return;
    }
    let page = if settings(game).remember {
        game.native_setting(rt::ID, "page")
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(0)
    } else {
        0
    };
    let meta = read_meta(&rt::page_path(&base(), page));
    rt::with(|s| {
        s.ready = true;
        s.page = page;
        s.meta = meta;
    });
}

fn page(game: &Game<'_>) -> i64 {
    ready(game);
    rt::with(|s| s.page)
}

/// A line over the hotbar.
fn overlay(game: &mut Game<'_>, text: String) {
    game.titles.set_overlay(Text::literal(text), false);
}

/// What a page is called in a notice: its name, else its file.
fn page_label(page: i64) -> String {
    rt::with(|s| s.meta.name.clone()).unwrap_or_else(|| rt::page_file_name(page))
}

/// Turns to `page`: the hotbar tab shows it and the save and load keys use it.
fn set_page(game: &mut Game<'_>, page: i64) {
    ready(game);
    let meta = read_meta(&rt::page_path(&base(), page));
    rt::with(|s| {
        s.page = page;
        s.meta = meta;
        s.rename = None;
        s.confirm = None;
    });
    let settings = settings(game);
    if settings.remember {
        game.native_set_setting(rt::ID, "page", &page.to_string());
    }
    game.refresh_saved_hotbars();
    let label = page_label(page);
    if matches!(settings.notify, rt::Notify::ActionBar | rt::Notify::Both) {
        overlay(game, format!("Hotbar selected: {label}"));
    }
    if matches!(settings.notify, rt::Notify::Toast | rt::Notify::Both) {
        game.native_add_toast(
            "librarian:page",
            "minecraft:bookshelf",
            Text::literal("Hotbar selected"),
            Text::literal(label),
            false,
        );
    }
}

fn step(game: &mut Game<'_>, by: i64) {
    let page = rt::stepped(page(game), by);
    set_page(game, page);
}

/// Copies the current page's file to `hotbars/backups/`.
fn backup(game: &mut Game<'_>) -> String {
    let page = page(game);
    let from = rt::page_path(&base(), page);
    let name = rt::page_file_name(page);
    if !from.is_file() {
        return format!("Failed to back up hotbar page: {name} has nothing saved yet");
    }
    let to = rt::backup_path(&base(), page, now_millis());
    let copied = to
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|()| std::fs::copy(&from, &to));
    match copied {
        Ok(_) => format!(
            "{name} backed up to {}",
            to.file_name().unwrap_or_default().to_string_lossy()
        ),
        Err(e) => format!("Failed to back up hotbar page: {e}"),
    }
}

/// Changes the current page's metadata and writes it to its file.
fn edit_meta(game: &mut Game<'_>, change: impl FnOnce(&mut rt::Meta)) -> Result<(), String> {
    let page = page(game);
    let meta = rt::with(|s| {
        change(&mut s.meta);
        s.meta.clone()
    });
    write_meta(&rt::page_path(&base(), page), &meta)?;
    // The title follows the name.
    game.refresh_saved_hotbars();
    Ok(())
}

/// The pages that have a file, in order.
fn pages_on_disk() -> Vec<i64> {
    let base = base();
    let mut pages: Vec<i64> = std::fs::read_dir(base.join("hotbars"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| rt::page_of_file_name(&entry.file_name().to_string_lossy()))
        .collect();
    if rt::page_path(&base, 0).is_file() {
        pages.push(0);
    }
    pages.sort_unstable();
    pages.dedup();
    pages
}

fn run(game: &mut Game<'_>, command: rt::Command) -> Vec<String> {
    use rt::Command::*;
    let done = |result: Result<(), String>, said: String| match result {
        Ok(()) => vec![said],
        Err(e) => vec![format!("Failed to save hotbar page: {e}")],
    };
    match command {
        Goto(page) => {
            set_page(game, page);
            Vec::new()
        }
        Next => {
            step(game, 1);
            Vec::new()
        }
        Previous => {
            step(game, -1);
            Vec::new()
        }
        Backup => vec![backup(game)],
        Pages => {
            let pages = pages_on_disk();
            if pages.is_empty() {
                vec!["No pages have been saved yet.".to_owned()]
            } else {
                let list: Vec<String> = pages.iter().map(i64::to_string).collect();
                vec![
                    format!("Saved page(s): {}", list.join(", ")),
                    format!("Current page: {}", page(game)),
                ]
            }
        }
        Reload => {
            let page = page(game);
            set_page(game, page);
            vec!["The current page has been read again.".to_owned()]
        }
        MetaDelete => {
            if rt::with(|s| s.meta.is_empty()) {
                return vec!["The current page has no metadata to delete.".to_owned()];
            }
            let result = edit_meta(game, |m| *m = rt::Meta::default());
            done(result, "The current page's metadata has been deleted.".to_owned())
        }
        NameShow => vec![match rt::with(|s| s.meta.name.clone()) {
            Some(name) => format!("The current page's name is currently set to {name}."),
            None => "The current page currently doesn't have a name.".to_owned(),
        }],
        NameSet(name) => {
            let said = match &name {
                Some(name) => format!("The current page has been renamed to {name}."),
                None => "The current page's name has been reset to defaults.".to_owned(),
            };
            let result = edit_meta(game, |m| m.name = name);
            done(result, said)
        }
        DescriptionShow => vec![match rt::with(|s| s.meta.description.clone()) {
            Some(text) => format!("The current page's description is currently set to {text}."),
            None => "The current page currently doesn't have a description.".to_owned(),
        }],
        DescriptionSet(text) => {
            let said = match &text {
                Some(text) => format!("The current page's description has been set to {text}."),
                None => "The current page's description has been reset to defaults.".to_owned(),
            };
            let result = edit_meta(game, |m| m.description = text);
            done(result, said)
        }
        AuthorsList => {
            let authors = rt::with(|s| s.meta.authors.clone());
            vec![if authors.is_empty() {
                "No authors are listed in the current page.".to_owned()
            } else {
                format!("Current page author(s): {}", authors.join(", "))
            }]
        }
        AuthorsClear => {
            let result = edit_meta(game, |m| m.authors.clear());
            done(
                result,
                "All authors have been removed from this page.".to_owned(),
            )
        }
        AuthorsAdd(name) => {
            if rt::with(|s| s.meta.authors.contains(&name)) {
                return vec![format!(
                    "{name} is already in the list of authors for this page."
                )];
            }
            let said = format!("{name} has been added to the list of authors for this page.");
            let result = edit_meta(game, |m| m.authors.push(name));
            done(result, said)
        }
        AuthorsRemove(name) => {
            if !rt::with(|s| s.meta.authors.contains(&name)) {
                return vec![format!("{name} isn't listed as an author for this page.")];
            }
            let said = format!("{name} has been removed from the list of authors for this page.");
            let result = edit_meta(game, |m| m.authors.retain(|a| *a != name));
            done(result, said)
        }
        Help => rt::USAGE.iter().map(|line| (*line).to_owned()).collect(),
    }
}

/// Whether the hotbar tab of the creative screen is what is open.
fn hotbar_tab_open(game: &Game<'_>) -> bool {
    game.screen == Screen::Creative && game.creative_screen.tab().hotbar
}

// ------------------------------------------------------------ the hooks

/// Librarian's row in Options > Mods, with its keys.
pub(super) fn listed(mods: &mut Vec<super::native_hooks::NativeMod>) {
    let key = |k: [&str; 3]| k.map(str::to_owned);
    mods.push(super::native_hooks::NativeMod {
        id: rt::ID.into(),
        title: format!("Librarian {}", rt::VERSION),
        description: rt::DESCRIPTION.into(),
        keys: vec![
            key(rt::KEY_NEXT),
            key(rt::KEY_PREVIOUS),
            key(rt::KEY_BACKUP),
        ],
    });
}

pub(super) fn settings_page(game: &Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id == rt::ID {
        buttons.extend(settings(game).labels());
    }
}

pub(super) fn settings_click(game: &mut Game<'_>, mod_id: &str, button: usize) {
    if mod_id != rt::ID {
        return;
    }
    let mut settings = settings(game);
    settings.click(button);
    for (key, value) in settings.stored() {
        game.native_set_setting(rt::ID, key, &value);
    }
}

/// The page keys, while playing in creative or over the creative screen
/// (not while its search box or the rename box takes the typing).
pub(super) fn key(game: &mut Game<'_>, key: &str, pressed: bool) {
    if !pressed || !game.native_on(rt::ID) || !game.creative {
        return;
    }
    let over_screen = game.screen == Screen::Creative && !game.creative_screen.tab().search;
    if !(game.native_playing() || over_screen) || rt::with(|s| s.rename.is_some()) {
        return;
    }
    let bound = |k: [&str; 3]| game.native_mod_key(rt::ID, k[0], k[2]) == key;
    if bound(rt::KEY_NEXT) {
        step(game, 1);
    } else if bound(rt::KEY_PREVIOUS) {
        step(game, -1);
    } else if bound(rt::KEY_BACKUP) {
        let said = backup(game);
        overlay(game, said);
    }
}

pub(super) fn commands(commands: &mut Vec<&'static str>) {
    commands.extend(rt::COMMANDS);
}

pub(super) fn command(game: &mut Game<'_>, line: &str, ran: &mut bool) {
    if !game.native_on(rt::ID) {
        return;
    }
    let Some(command) = rt::parse_command(line) else {
        return;
    };
    *ran = true;
    for line in run(game, command) {
        game.native_chat(&line);
    }
}

/// Once a frame: what the read-only hooks asked for.
pub(super) fn frame(game: &mut Game<'_>) {
    if !game.native_on(rt::ID) || !rt::with(|s| s.ready) {
        return;
    }
    if game.screen != Screen::Creative {
        rt::with(|s| s.rename = None);
    }
    for action in rt::with(|s| std::mem::take(&mut s.actions)) {
        match action {
            rt::Action::Click => game.audio.play_button_click(),
            rt::Action::Step(by) => step(game, by),
            rt::Action::Backup => {
                let said = backup(game);
                overlay(game, said);
            }
            rt::Action::Rename(name) => {
                if let Err(e) = edit_meta(game, |m| m.name = name) {
                    overlay(game, format!("Failed to save hotbar page: {e}"));
                }
            }
        }
    }
}

/// The current page's file. Page 0 is the game's own `hotbar.nbt`.
fn current_path(game: &Game<'_>) -> PathBuf {
    let path = Game::options_path().with_file_name("hotbar.nbt");
    if !game.native_on(rt::ID) {
        return path;
    }
    match page(game) {
        0 => path,
        page => rt::page_path(path.parent().unwrap_or(Path::new("")), page),
    }
}

/// In place of the game's `refresh_saved_hotbars`
/// (`CreativeModeInventoryScreen.selectTab` on the HOTBAR tab): the same
/// rows and hints, read from the current page's file.
pub(super) fn refresh_saved_hotbars(game: &mut Game<'_>) {
    if !game.creative_screen.tab().hotbar {
        return;
    }
    let path = current_path(game);
    let known = |id: &str| game.res.items().get(id).is_some();
    let rows = hotbar::load(&path, &known);
    let key = |mapping: &str| keybinds::display_name(game.settings.keys.get(mapping)).plain();
    let empty = || ItemStack::new("minecraft:air", 0);
    let mut items = Vec::with_capacity(hotbar::GROUPS * hotbar::SIZE);
    for (i, row) in rows.iter().enumerate().take(hotbar::GROUPS) {
        if row.iter().any(Option::is_some) {
            items.extend(row.iter().map(|s| s.clone().unwrap_or_else(empty)));
            continue;
        }
        for j in 0..hotbar::SIZE {
            if j != i {
                items.push(empty());
                continue;
            }
            let with = [
                key("key.saveToolbarActivator"),
                key(&format!("key.hotbar.{}", i + 1)),
            ];
            let mut paper = ItemStack::new("minecraft:paper", 1);
            let data = paper.data_mut();
            data.set_other(
                "minecraft:creative_slot_lock",
                Some(Tag::compound(Vec::<(&str, Tag)>::new())),
            );
            data.set_other(
                "minecraft:item_name",
                Some(Tag::compound([
                    ("translate", Tag::String("inventory.hotbarInfo".into())),
                    (
                        "with",
                        Tag::List(
                            with.iter()
                                .map(|s| Tag::String(s.as_str().into()))
                                .collect(),
                        ),
                    ),
                ])),
            );
            items.push(paper);
        }
    }
    game.creative_screen.set_saved_hotbars(items);
}

/// In place of the game's `hotbar_load_or_save`
/// (`CreativeModeInventoryScreen.handleHotbarLoadOrSave`), on the current
/// page's file. A save over a row that holds other items is refused once,
/// with a notice; the same keys again within a few seconds save it. A save
/// keeps the page's metadata, which the game's writer leaves out.
pub(super) fn hotbar_load_or_save(game: &mut Game<'_>, index: usize, load: bool) {
    let path = current_path(game);
    let known = |id: &str| game.items().get(id).is_some();
    let mut rows = hotbar::load(&path, &known);
    if load {
        for (slot, stack) in rows[index].iter().enumerate() {
            game.inventory[slot] = stack.clone();
        }
        return;
    }
    let on = game.native_on(rt::ID);
    if on {
        let differs = settings(game).protect
            && rows[index].iter().any(Option::is_some)
            && rows[index][..] != game.inventory[..hotbar::SIZE];
        if !rt::with(|s| rt::save_allowed(s, index, differs, now_millis())) {
            overlay(
                game,
                format!(
                    "Row {} holds different items. Press the same keys again to overwrite it.",
                    index + 1
                ),
            );
            return;
        }
    }
    rows[index] = game.inventory[..hotbar::SIZE].to_vec();
    if let Some(folder) = path.parent()
        && let Err(e) = std::fs::create_dir_all(folder)
    {
        log::warn!("librarian: {} was not created: {e}", folder.display());
    }
    if let Err(e) = hotbar::save(&path, &rows) {
        // saveHotbar sets the message inside its try, so a failed save says nothing.
        log::warn!("the creative hotbar was not saved: {e}");
        return;
    }
    if on {
        let meta = rt::with(|s| s.meta.clone());
        if !meta.is_empty()
            && let Err(e) = write_meta(&path, &meta)
        {
            log::warn!("librarian: the page's metadata was not written back: {e}");
        }
    }
    let key = |mapping: &str| keybinds::display_name(game.settings.keys.get(mapping));
    let message = Text::translate(
        "inventory.hotbarSaved",
        &[
            key("key.loadToolbarActivator"),
            key(&format!("key.hotbar.{}", index + 1)),
        ],
    );
    // handleHotbarLoadOrSave: the overlay, said at once.
    game.say_system_now(&message);
    game.titles.set_overlay(message, false);
}

/// The hotbar tab's title: the page's name or number.
pub(super) fn tab_title(
    game: &Game<'_>,
    tab: &horizon_client::creative::Tab,
) -> ControlFlow<String> {
    if !tab.hotbar || !game.native_on(rt::ID) {
        return ControlFlow::Continue(());
    }
    let page = page(game);
    ControlFlow::Break(rt::with(|s| rt::title(page, &s.meta)))
}

/// Over the creative screen's hotbar tab: the page buttons, the rename box
/// and their tooltips.
pub(super) fn draw(game: &Game<'_>, batch: &mut HudBatch, screen: &NativeScreenView<'_>) {
    if screen.id != "creative" || !game.native_on(rt::ID) || !hotbar_tab_open(game) {
        return;
    }
    let page = page(game);
    let origin = [screen.rect[0], screen.rect[1]];
    let mouse = screen.mouse;
    let saved = rt::page_path(&base(), page).is_file();
    let hovered = rt::button_at(origin, mouse);
    for button in rt::BUTTONS {
        let [x, y, w, h] = rt::button_rect(origin, button);
        let active = button != rt::Button::Backup || saved;
        let sprite = match (active, hovered == Some(button)) {
            (false, _) => "gui/sprites/widget/button_disabled",
            (true, true) => "gui/sprites/widget/button_highlighted",
            (true, false) => "gui/sprites/widget/button",
        };
        horizon_client::ui::blit_sprite(batch, &game.gui.atlas, sprite, x, y, w, h, u32::MAX);
        let label = match button {
            rt::Button::Previous => "<",
            rt::Button::Backup => "B",
            rt::Button::Next => ">",
        };
        let width = game.native_text_width(label);
        let color = if active { 0xFF_FFFF } else { 0xA0_A0A0 };
        game.native_text(batch, label, [x + (w - width) / 2.0, y + 2.0], color, true);
    }
    let rename = rt::with(|s| s.rename.clone());
    if let Some(rename) = &rename {
        let [x, y, w, h] = rt::title_rect(origin);
        game.native_fill(batch, [x, y, w, h], 0xFF00_0000);
        game.native_fill(batch, [x + 1.0, y + 1.0, w - 2.0, h - 2.0], 0xFF2B_2B2B);
        game.native_text(batch, &rename.text, [x + 2.0, y + 2.0], 0xFF_FFFF, false);
        // The caret blinks twice a second.
        if now_millis() / 300 % 2 == 0 {
            let at = x + 2.0 + game.native_text_width(rename.before_caret());
            game.native_fill(batch, [at, y + 1.0, 1.0, h - 2.0], 0xFFD0_D0D0);
        }
    }
    let tip = match hovered {
        Some(rt::Button::Previous) => Some("Previous page".to_owned()),
        Some(rt::Button::Next) => Some("Next page".to_owned()),
        Some(rt::Button::Backup) if saved => Some("Make a backup of this page".to_owned()),
        Some(rt::Button::Backup) => Some("Nothing saved on this page yet".to_owned()),
        None if rename.is_none() && rt::inside(rt::title_rect(origin), mouse) => {
            Some(rt::with(|s| {
                let mut tip = String::from("Click to Rename");
                if let Some(description) = &s.meta.description {
                    tip = format!("{description} ({tip})");
                }
                if !s.meta.authors.is_empty() {
                    tip = format!("{tip} - Authors: {}", s.meta.authors.join(", "));
                }
                tip
            }))
        }
        None => None,
    };
    if let Some(tip) = tip {
        game.tooltip(batch, &tip, mouse[0], mouse[1]);
    }
}

/// A click over the creative screen's hotbar tab: a page button, or the
/// title to rename the page.
pub(super) fn mouse_pressed(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    if screen.id != "creative" || !game.native_on(rt::ID) || !hotbar_tab_open(game) {
        return;
    }
    let page = page(game);
    let origin = [screen.rect[0], screen.rect[1]];
    let renaming = rt::with(|s| s.rename.is_some());
    if button != 0 {
        rt::with(|s| s.rename = None);
        return;
    }
    if let Some(hit) = rt::button_at(origin, screen.mouse) {
        *canceled = true;
        let action = match hit {
            rt::Button::Previous => rt::Action::Step(-1),
            rt::Button::Next => rt::Action::Step(1),
            rt::Button::Backup if rt::page_path(&base(), page).is_file() => rt::Action::Backup,
            rt::Button::Backup => return,
        };
        rt::with(|s| {
            s.rename = None;
            s.actions.push(rt::Action::Click);
            s.actions.push(action);
        });
    } else if rt::inside(rt::title_rect(origin), screen.mouse) {
        *canceled = true;
        if !renaming {
            rt::with(|s| {
                let text = s.meta.name.clone().unwrap_or_default();
                let caret = text.chars().count();
                s.rename = Some(rt::Rename { text, caret });
            });
        }
    } else {
        // A click anywhere else leaves the box without renaming.
        rt::with(|s| s.rename = None);
    }
}

/// A key over the screen while the rename box is open: the box takes it.
pub(super) fn key_pressed(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    key: &str,
    canceled: &mut bool,
) {
    if screen.id != "creative" || !game.native_on(rt::ID) {
        return;
    }
    rt::with(|s| {
        let Some(rename) = &mut s.rename else {
            return;
        };
        *canceled = true;
        if key == "key.keyboard.escape" {
            s.rename = None;
        } else if let Some(action) = rename.key(key) {
            s.rename = None;
            s.actions.push(action);
        }
    });
}

/// A character typed while the rename box is open.
pub(super) fn char_typed(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    c: char,
    canceled: &mut bool,
) {
    if screen.id != "creative" || !game.native_on(rt::ID) {
        return;
    }
    rt::with(|s| {
        if let Some(rename) = &mut s.rename {
            *canceled = true;
            rename.insert(c);
        }
    });
}
