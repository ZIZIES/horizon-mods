// Librarian for Horizon: saved hotbar pages. A port of what the Fabric mod
// Librarian does (pages of saved hotbars, page names and other metadata kept
// in the same files, backups, overwrite protection), written against
// Horizon's creative toolbars. This file is the mod's logic with no game
// types in it, so it is both the Cargo library the tests run on and the
// module the bundle adds to the game; `glue.rs` is the part that touches the
// game.
use std::cell::RefCell;
use std::path::{Path, PathBuf};

pub const ID: &str = "librarian";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "Saved hotbar pages: flip through as many pages of saved hotbars as you like, name them, back them up, and get asked before a saved row is overwritten.";

/// The keys: name, label, default.
pub const KEY_NEXT: [&str; 3] = ["next_page", "Next page", "key.keyboard.right.bracket"];
pub const KEY_PREVIOUS: [&str; 3] = [
    "previous_page",
    "Previous page",
    "key.keyboard.left.bracket",
];
pub const KEY_BACKUP: [&str; 3] = ["backup_page", "Backup current page", "key.keyboard.b"];

/// The commands, without their slash.
pub const COMMANDS: [&str; 2] = ["librarian", "lb"];

/// How long a refused save waits for the second press, in milliseconds.
pub const CONFIRM_MS: u64 = 5000;
/// The longest name the rename box takes.
pub const NAME_CHARS: usize = 64;

// ---------------------------------------------------------------- files

/// A page's file name, as Librarian names it: page 0 is vanilla's own file.
pub fn page_file_name(page: i64) -> String {
    if page == 0 {
        "hotbar.nbt".to_owned()
    } else {
        format!("hotbar.{page}.nbt")
    }
}

/// Where a page's file is: page 0 beside `options.txt` (vanilla's
/// `hotbar.nbt`), every other page in `hotbars/` beside it.
pub fn page_path(base: &Path, page: i64) -> PathBuf {
    if page == 0 {
        base.join(page_file_name(0))
    } else {
        base.join("hotbars").join(page_file_name(page))
    }
}

/// The page a file in `hotbars/` holds (`hotbar.<n>.nbt`).
pub fn page_of_file_name(name: &str) -> Option<i64> {
    let rest = name.strip_prefix("hotbar.")?;
    let number = rest.strip_suffix(".nbt")?;
    number.parse().ok()
}

/// `hotbars/backups/<file> [<date>].nbt`, the date in UTC.
pub fn backup_path(base: &Path, page: i64, unix_millis: u64) -> PathBuf {
    base.join("hotbars").join("backups").join(format!(
        "{} [{}].nbt",
        page_file_name(page),
        date(unix_millis)
    ))
}

/// `yyyy-MM-dd at HH.mm.ss.SSS +0000` (no colons: Windows file names).
pub fn date(unix_millis: u64) -> String {
    let millis = unix_millis % 1000;
    let seconds = unix_millis / 1000;
    let (h, m, s) = (
        (seconds / 3600) % 24,
        (seconds / 60) % 60,
        seconds % 60,
    );
    let (year, month, day) = civil((seconds / 86400) as i64);
    format!("{year:04}-{month:02}-{day:02} at {h:02}.{m:02}.{s:02}.{millis:03} +0000")
}

/// Days since 1970-01-01 to a calendar date (Howard Hinnant's algorithm).
fn civil(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(month <= 2), month, day)
}

// ------------------------------------------------------------- metadata

/// A page's metadata, the `librarian` compound of its file: plain text here,
/// JSON text components in the file (as the Fabric mod writes them).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Meta {
    pub name: Option<String>,
    pub description: Option<String>,
    pub authors: Vec<String>,
}

impl Meta {
    pub fn is_empty(&self) -> bool {
        self.name.is_none() && self.description.is_none() && self.authors.is_empty()
    }
}

/// The plain text of a JSON text component (`"x"`, `{"text":"x","extra":[…]}`,
/// `[…]`); text that is no JSON is taken as it is.
pub fn component_plain(json: &str) -> String {
    let mut parser = Json {
        s: json.as_bytes(),
        at: 0,
    };
    let mut out = String::new();
    parser.space();
    if parser.value(&mut out, true).is_some() {
        parser.space();
        if parser.at == parser.s.len() {
            return out;
        }
    }
    json.to_owned()
}

/// Plain text as a JSON text component.
pub fn plain_component(text: &str) -> String {
    let mut out = String::from("{\"text\":\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push_str("\"}");
    out
}

/// Just enough JSON to read a text component's text.
struct Json<'a> {
    s: &'a [u8],
    at: usize,
}

impl Json<'_> {
    fn space(&mut self) {
        while self.s.get(self.at).is_some_and(u8::is_ascii_whitespace) {
            self.at += 1;
        }
    }
    fn eat(&mut self, c: u8) -> Option<()> {
        self.space();
        (self.s.get(self.at) == Some(&c)).then(|| self.at += 1)
    }
    /// One value; its text goes to `out` when `shown`.
    fn value(&mut self, out: &mut String, shown: bool) -> Option<()> {
        self.space();
        match *self.s.get(self.at)? {
            b'"' => {
                let text = self.string()?;
                if shown {
                    out.push_str(&text);
                }
                Some(())
            }
            b'[' => {
                self.at += 1;
                if self.eat(b']').is_some() {
                    return Some(());
                }
                loop {
                    self.value(out, shown)?;
                    if self.eat(b',').is_none() {
                        return self.eat(b']');
                    }
                }
            }
            b'{' => {
                self.at += 1;
                if self.eat(b'}').is_some() {
                    return Some(());
                }
                // `text` before `extra`, whatever order the file has them in.
                let (mut text, mut extra) = (String::new(), String::new());
                loop {
                    self.space();
                    let key = self.string()?;
                    self.eat(b':')?;
                    match key.as_str() {
                        "text" => self.value(&mut text, shown)?,
                        "extra" => self.value(&mut extra, shown)?,
                        _ => self.value(&mut String::new(), false)?,
                    }
                    if self.eat(b',').is_none() {
                        self.eat(b'}')?;
                        out.push_str(&text);
                        out.push_str(&extra);
                        return Some(());
                    }
                }
            }
            _ => {
                // A number, true, false or null: no text.
                let start = self.at;
                while self
                    .s
                    .get(self.at)
                    .is_some_and(|c| c.is_ascii_alphanumeric() || b"+-.".contains(c))
                {
                    self.at += 1;
                }
                (self.at > start).then_some(())
            }
        }
    }
    fn string(&mut self) -> Option<String> {
        if self.s.get(self.at) != Some(&b'"') {
            return None;
        }
        self.at += 1;
        let mut bytes = Vec::new();
        loop {
            let c = *self.s.get(self.at)?;
            self.at += 1;
            match c {
                b'"' => return String::from_utf8(bytes).ok(),
                b'\\' => {
                    let e = *self.s.get(self.at)?;
                    self.at += 1;
                    let c = match e {
                        b'n' => '\n',
                        b't' => '\t',
                        b'r' => '\r',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        b'u' => {
                            let hex = self.s.get(self.at..self.at + 4)?;
                            self.at += 4;
                            let code =
                                u32::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok()?;
                            char::from_u32(code).unwrap_or('\u{fffd}')
                        }
                        other => other as char,
                    };
                    let mut buffer = [0; 4];
                    bytes.extend_from_slice(c.encode_utf8(&mut buffer).as_bytes());
                }
                c => bytes.push(c),
            }
        }
    }
}

/// The hotbar tab's title: the page's name, else `Saved Hotbars (Page n)`.
pub fn title(page: i64, meta: &Meta) -> String {
    match &meta.name {
        Some(name) if !name.trim().is_empty() => name.clone(),
        _ => format!("Saved Hotbars (Page {page})"),
    }
}

// ------------------------------------------------------------- commands

/// What `/librarian …` (or `/lb …`) asks for.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Goto(i64),
    Next,
    Previous,
    Backup,
    /// `cache list`: here, the pages that have a file.
    Pages,
    /// `cache clear`: read the current page's metadata again.
    Reload,
    MetaDelete,
    NameShow,
    NameSet(Option<String>),
    DescriptionShow,
    DescriptionSet(Option<String>),
    AuthorsList,
    AuthorsClear,
    AuthorsAdd(String),
    AuthorsRemove(String),
    /// Anything else: the usage lines.
    Help,
}

/// `line` is the command without its slash. None when it is not ours.
pub fn parse_command(line: &str) -> Option<Command> {
    let line = line.trim();
    let (head, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
    if !COMMANDS.contains(&head) {
        return None;
    }
    let rest = rest.trim();
    let (verb, args) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    let args = args.trim();
    Some(match verb {
        "goto" => args.parse().map_or(Command::Help, Command::Goto),
        "next" => Command::Next,
        "previous" | "prev" => Command::Previous,
        "backup" => Command::Backup,
        "cache" => match args {
            "list" => Command::Pages,
            "clear" => Command::Reload,
            _ => Command::Help,
        },
        "pages" => Command::Pages,
        "meta" => parse_meta(args),
        _ => Command::Help,
    })
}

fn parse_meta(args: &str) -> Command {
    let (what, rest) = args.split_once(char::is_whitespace).unwrap_or((args, ""));
    let rest = rest.trim();
    // `set` alone resets, `set <value>` sets; nothing shows.
    let value = |rest: &str| -> Option<Option<String>> {
        if rest.is_empty() {
            return None;
        }
        let (set, value) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
        (set == "set").then(|| Some(value.trim().to_owned()).filter(|v| !v.is_empty()))
    };
    match what {
        "delete" => Command::MetaDelete,
        "name" if rest.is_empty() => Command::NameShow,
        "name" => value(rest).map_or(Command::Help, Command::NameSet),
        "description" if rest.is_empty() => Command::DescriptionShow,
        "description" => value(rest).map_or(Command::Help, Command::DescriptionSet),
        "authors" => {
            let (verb, name) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
            let name = name.trim();
            match verb {
                "list" => Command::AuthorsList,
                "clear" => Command::AuthorsClear,
                "add" if !name.is_empty() => Command::AuthorsAdd(name.to_owned()),
                "remove" if !name.is_empty() => Command::AuthorsRemove(name.to_owned()),
                _ => Command::Help,
            }
        }
        _ => Command::Help,
    }
}

pub const USAGE: [&str; 9] = [
    "/librarian goto <page> | next | previous",
    "/librarian backup",
    "/librarian cache list | clear",
    "/librarian meta name [set [<name>]]",
    "/librarian meta description [set [<text>]]",
    "/librarian meta authors list | clear",
    "/librarian meta authors add <name> | remove <name>",
    "/librarian meta delete",
    "(/lb is the same command)",
];

// ------------------------------------------------------------- settings

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Notify {
    ActionBar,
    Toast,
    Both,
    Off,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Start on the page that was open last.
    pub remember: bool,
    pub notify: Notify,
    /// Ask before a saved row is overwritten with different items.
    pub protect: bool,
}

impl Settings {
    pub fn read(get: impl Fn(&str) -> Option<String>) -> Self {
        let flag = |key: &str| get(key).is_none_or(|v| v.trim() != "false");
        let notify = match get("notify").as_deref().map(str::trim) {
            Some("toast") => Notify::Toast,
            Some("both") => Notify::Both,
            Some("off") => Notify::Off,
            _ => Notify::ActionBar,
        };
        Self {
            remember: flag("remember"),
            notify,
            protect: flag("protect"),
        }
    }
    pub fn labels(&self) -> Vec<String> {
        let on = |b: bool| if b { "ON" } else { "OFF" };
        let notify = match self.notify {
            Notify::ActionBar => "Action bar",
            Notify::Toast => "Toast",
            Notify::Both => "Action bar and toast",
            Notify::Off => "OFF",
        };
        vec![
            format!("Remember last page: {}", on(self.remember)),
            format!("Page change notice: {notify}"),
            format!("Overwrite protection: {}", on(self.protect)),
        ]
    }
    pub fn click(&mut self, button: usize) {
        match button {
            0 => self.remember = !self.remember,
            1 => {
                self.notify = match self.notify {
                    Notify::ActionBar => Notify::Toast,
                    Notify::Toast => Notify::Both,
                    Notify::Both => Notify::Off,
                    Notify::Off => Notify::ActionBar,
                }
            }
            2 => self.protect = !self.protect,
            _ => {}
        }
    }
    pub fn stored(&self) -> [(&'static str, String); 3] {
        let flag = |b: bool| if b { "true" } else { "false" }.to_owned();
        let notify = match self.notify {
            Notify::ActionBar => "action_bar",
            Notify::Toast => "toast",
            Notify::Both => "both",
            Notify::Off => "off",
        };
        [
            ("remember", flag(self.remember)),
            ("notify", notify.to_owned()),
            ("protect", flag(self.protect)),
        ]
    }
}

// --------------------------------------------------------------- layout

/// The creative panel's buttons, left to right.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Button {
    Previous,
    Backup,
    Next,
}

pub const BUTTONS: [Button; 3] = [Button::Previous, Button::Backup, Button::Next];
/// A button's side, in GUI pixels.
pub const BUTTON: f32 = 12.0;

/// A button's rectangle `[x, y, w, h]` for the creative panel at `origin`:
/// the three of them end where the scroll bar's column does, in the title's
/// row.
pub fn button_rect(origin: [f32; 2], button: Button) -> [f32; 4] {
    let i = match button {
        Button::Previous => 0.0,
        Button::Backup => 1.0,
        Button::Next => 2.0,
    };
    [origin[0] + 151.0 + i * BUTTON, origin[1] + 4.0, BUTTON, BUTTON]
}

/// The title's rectangle: a click on it renames the page.
pub fn title_rect(origin: [f32; 2]) -> [f32; 4] {
    [origin[0] + 7.0, origin[1] + 4.0, 142.0, 11.0]
}

pub fn inside(rect: [f32; 4], at: [f32; 2]) -> bool {
    at[0] >= rect[0] && at[0] < rect[0] + rect[2] && at[1] >= rect[1] && at[1] < rect[1] + rect[3]
}

pub fn button_at(origin: [f32; 2], at: [f32; 2]) -> Option<Button> {
    BUTTONS
        .into_iter()
        .find(|b| inside(button_rect(origin, *b), at))
}

// ---------------------------------------------------------------- state

/// What a read-only hook (a click, a key over the screen) asks the next
/// frame to do, where the game can be changed.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Step(i64),
    Backup,
    Rename(Option<String>),
    Click,
}

/// The rename box: the page's name being typed over the title.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rename {
    pub text: String,
    /// The caret, in characters.
    pub caret: usize,
}

impl Rename {
    pub fn insert(&mut self, c: char) {
        if c.is_control() || self.text.chars().count() >= NAME_CHARS {
            return;
        }
        let at = self.byte(self.caret);
        self.text.insert(at, c);
        self.caret += 1;
    }
    /// A key by its vanilla name; whether it ends the box, and how.
    pub fn key(&mut self, key: &str) -> Option<Action> {
        let len = self.text.chars().count();
        match key {
            "key.keyboard.enter" | "key.keyboard.keypad.enter" => {
                let name = self.text.trim().to_owned();
                return Some(Action::Rename(Some(name).filter(|n| !n.is_empty())));
            }
            "key.keyboard.backspace" if self.caret > 0 => {
                let at = self.byte(self.caret - 1);
                self.text.remove(at);
                self.caret -= 1;
            }
            "key.keyboard.delete" if self.caret < len => {
                let at = self.byte(self.caret);
                self.text.remove(at);
            }
            "key.keyboard.left" => self.caret = self.caret.saturating_sub(1),
            "key.keyboard.right" => self.caret = (self.caret + 1).min(len),
            "key.keyboard.home" => self.caret = 0,
            "key.keyboard.end" => self.caret = len,
            _ => {}
        }
        None
    }
    fn byte(&self, chars: usize) -> usize {
        self.text
            .char_indices()
            .nth(chars)
            .map_or(self.text.len(), |(i, _)| i)
    }
    /// The text before the caret, to measure where the caret is drawn.
    pub fn before_caret(&self) -> &str {
        &self.text[..self.byte(self.caret)]
    }
}

#[derive(Default)]
pub struct State {
    /// Read from the settings on first use.
    pub ready: bool,
    pub page: i64,
    /// The current page's metadata, read from its file.
    pub meta: Meta,
    pub rename: Option<Rename>,
    pub actions: Vec<Action>,
    /// The row a save was refused for, and until when a second press saves.
    pub confirm: Option<(usize, u64)>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

/// The mod's state, one per thread (the window's game and its server thread
/// each have their own; only the window's is ever used).
pub fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with_borrow_mut(f)
}

/// Whether a save of `row` goes through: always when the saved row is empty
/// or holds the same items; otherwise only on a second press within
/// `CONFIRM_MS` of the refused one.
pub fn save_allowed(state: &mut State, row: usize, differs: bool, now_millis: u64) -> bool {
    if !differs {
        state.confirm = None;
        return true;
    }
    match state.confirm.take() {
        Some((asked, until)) if asked == row && now_millis <= until => true,
        _ => {
            state.confirm = Some((row, now_millis + CONFIRM_MS));
            false
        }
    }
}

/// The page `step` pages from `page`.
pub fn stepped(page: i64, step: i64) -> i64 {
    page.saturating_add(step)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_files_are_librarians() {
        let base = Path::new("/p");
        assert_eq!(page_path(base, 0), Path::new("/p/hotbar.nbt"));
        assert_eq!(page_path(base, 7), Path::new("/p/hotbars/hotbar.7.nbt"));
        assert_eq!(page_path(base, -2), Path::new("/p/hotbars/hotbar.-2.nbt"));
        assert_eq!(page_of_file_name("hotbar.7.nbt"), Some(7));
        assert_eq!(page_of_file_name("hotbar.-2.nbt"), Some(-2));
        assert_eq!(page_of_file_name("hotbar.nbt"), None);
        assert_eq!(page_of_file_name("hotbar.7.nbt_old"), None);
    }

    #[test]
    fn backup_names_carry_the_date() {
        // 2026-11-14 18:19:05.123 UTC.
        let millis = 1_794_680_345_123;
        assert_eq!(date(millis), "2026-11-14 at 18.19.05.123 +0000");
        assert_eq!(date(0), "1970-01-01 at 00.00.00.000 +0000");
        // A leap day.
        assert_eq!(date(1_709_164_800_000), "2024-02-29 at 00.00.00.000 +0000");
        assert_eq!(
            backup_path(Path::new("/p"), 3, 0),
            Path::new("/p/hotbars/backups/hotbar.3.nbt [1970-01-01 at 00.00.00.000 +0000].nbt")
        );
    }

    #[test]
    fn components_read_as_plain_text() {
        assert_eq!(component_plain("\"PvP kits\""), "PvP kits");
        assert_eq!(component_plain("{\"text\":\"PvP kits\"}"), "PvP kits");
        assert_eq!(
            component_plain(
                "{\"extra\":[{\"text\":\"B\",\"bold\":true},\"C\"],\"color\":\"red\",\"text\":\"A\"}"
            ),
            "ABC"
        );
        assert_eq!(component_plain("[\"a\",{\"text\":\"b\"}]"), "ab");
        assert_eq!(component_plain("{\"text\":\"q\\\"\\u0041\"}"), "q\"A");
        // Not JSON: kept.
        assert_eq!(component_plain("just words"), "just words");
        assert_eq!(component_plain("{\"text\":"), "{\"text\":");
    }

    #[test]
    fn plain_text_round_trips_through_a_component() {
        for text in ["Kits", "a \"b\" \\ c", "línea\n2", ""] {
            assert_eq!(component_plain(&plain_component(text)), text);
        }
    }

    #[test]
    fn titles_prefer_the_name() {
        let mut meta = Meta::default();
        assert_eq!(title(4, &meta), "Saved Hotbars (Page 4)");
        meta.name = Some("  ".into());
        assert_eq!(title(-1, &meta), "Saved Hotbars (Page -1)");
        meta.name = Some("Kits".into());
        assert_eq!(title(4, &meta), "Kits");
    }

    #[test]
    fn commands_parse() {
        use Command::*;
        let p = |s| parse_command(s);
        assert_eq!(p("gamemode creative"), None);
        assert_eq!(p("librarianx goto 1"), None);
        assert_eq!(p("librarian goto 12"), Some(Goto(12)));
        assert_eq!(p("lb goto -3"), Some(Goto(-3)));
        assert_eq!(p("lb goto x"), Some(Help));
        assert_eq!(p("lb next"), Some(Next));
        assert_eq!(p("lb previous"), Some(Previous));
        assert_eq!(p("lb backup"), Some(Backup));
        assert_eq!(p("lb cache list"), Some(Pages));
        assert_eq!(p("lb cache clear"), Some(Reload));
        assert_eq!(p("lb"), Some(Help));
        assert_eq!(p("lb meta delete"), Some(MetaDelete));
        assert_eq!(p("lb meta name"), Some(NameShow));
        assert_eq!(p("lb meta name set"), Some(NameSet(None)));
        assert_eq!(
            p("lb meta name set My  kits "),
            Some(NameSet(Some("My  kits".into())))
        );
        assert_eq!(p("lb meta name bogus"), Some(Help));
        assert_eq!(p("lb meta description"), Some(DescriptionShow));
        assert_eq!(
            p("lb meta description set a b"),
            Some(DescriptionSet(Some("a b".into())))
        );
        assert_eq!(p("lb meta authors list"), Some(AuthorsList));
        assert_eq!(p("lb meta authors clear"), Some(AuthorsClear));
        assert_eq!(p("lb meta authors add Zed"), Some(AuthorsAdd("Zed".into())));
        assert_eq!(
            p("lb meta authors remove Zed"),
            Some(AuthorsRemove("Zed".into()))
        );
        assert_eq!(p("lb meta authors add"), Some(Help));
    }

    #[test]
    fn settings_default_cycle_and_store() {
        let mut s = Settings::read(|_| None);
        assert_eq!(
            s,
            Settings {
                remember: true,
                notify: Notify::ActionBar,
                protect: true
            }
        );
        s.click(0);
        s.click(1);
        s.click(2);
        let stored = s.stored();
        let back = Settings::read(|key| {
            stored
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.clone())
        });
        assert_eq!(back, s);
        assert_eq!(
            back,
            Settings {
                remember: false,
                notify: Notify::Toast,
                protect: false
            }
        );
        assert_eq!(s.labels().len(), 3);
    }

    #[test]
    fn buttons_sit_in_the_title_row_and_do_not_overlap() {
        let origin = [100.0, 50.0];
        let [a, b, c] = BUTTONS.map(|b| button_rect(origin, b));
        assert_eq!(a[0] + a[2], b[0]);
        assert_eq!(b[0] + b[2], c[0]);
        // Above the first row of slots (18) and left of the panel's edge (195).
        assert!(c[1] + c[3] <= origin[1] + 18.0);
        assert!(c[0] + c[2] <= origin[0] + 195.0);
        assert_eq!(button_at(origin, [a[0], a[1]]), Some(Button::Previous));
        assert_eq!(button_at(origin, [c[0] + 11.0, c[1] + 11.0]), Some(Button::Next));
        assert_eq!(button_at(origin, [c[0] + 12.0, c[1]]), None);
        // The title's click area stops before the buttons.
        let t = title_rect(origin);
        assert!(t[0] + t[2] <= a[0]);
    }

    #[test]
    fn a_different_row_needs_a_second_press_in_time() {
        let mut s = State::default();
        // Same items, or an empty saved row: straight through.
        assert!(save_allowed(&mut s, 2, false, 0));
        // Different: refused, then allowed on the second press.
        assert!(!save_allowed(&mut s, 2, true, 1000));
        assert!(save_allowed(&mut s, 2, true, 1000 + CONFIRM_MS));
        // The confirmation is used up.
        assert!(!save_allowed(&mut s, 2, true, 2000));
        // Too late: asked again.
        assert!(!save_allowed(&mut s, 2, true, 2001 + CONFIRM_MS + 2000));
        // Another row: asked for that row.
        assert!(!save_allowed(&mut s, 3, true, 9000 + CONFIRM_MS + 2000));
        assert!(save_allowed(&mut s, 3, true, 9001 + CONFIRM_MS + 2000));
    }

    #[test]
    fn rename_box_edits_and_ends() {
        let mut r = Rename::default();
        for c in "Kíts".chars() {
            r.insert(c);
        }
        r.insert('\u{7f}');
        assert_eq!(r.text, "Kíts");
        assert_eq!(r.key("key.keyboard.left"), None);
        assert_eq!(r.key("key.keyboard.backspace"), None);
        assert_eq!(r.text, "Kís");
        assert_eq!(r.before_caret(), "Kí");
        r.key("key.keyboard.home");
        r.key("key.keyboard.delete");
        assert_eq!(r.text, "ís");
        r.key("key.keyboard.end");
        r.insert('!');
        assert_eq!(
            r.key("key.keyboard.enter"),
            Some(Action::Rename(Some("ís!".into())))
        );
        let mut blank = Rename {
            text: "  ".into(),
            caret: 2,
        };
        assert_eq!(blank.key("key.keyboard.enter"), Some(Action::Rename(None)));
        let mut long = Rename::default();
        for _ in 0..NAME_CHARS + 5 {
            long.insert('x');
        }
        assert_eq!(long.text.len(), NAME_CHARS);
    }

    #[test]
    fn pages_step_without_overflow() {
        assert_eq!(stepped(0, -1), -1);
        assert_eq!(stepped(i64::MAX, 1), i64::MAX);
    }
}
