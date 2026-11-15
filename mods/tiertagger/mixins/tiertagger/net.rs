// TierTagger's requests to the tier list, each on a thread of its own: the
// game modes once per tier list, and a player's rankings the first time
// their name is drawn. Nothing but the player's UUID is sent.
use super::tiertagger as tt;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Requests under way, and the most at once (a full player list asks for
/// everyone in it).
static BUSY: AtomicUsize = AtomicUsize::new(0);
const MOST_BUSY: usize = 6;
static MODES_ASKED: AtomicBool = AtomicBool::new(false);
/// Players waiting for a free thread.
static QUEUE: std::sync::Mutex<Vec<(u128, &'static str)>> = std::sync::Mutex::new(Vec::new());

fn get(url: &str) -> Option<serde_json::Value> {
    let body = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("TierTagger-Horizon")
        .build()
        .get(url)
        .call()
        .ok()?
        .into_string()
        .ok()?;
    serde_json::from_str(&body).ok()
}

/// `{"vanilla": {"tier": 3, "pos": 0, "peak_tier": 2, "peak_pos": 1,
/// "attained": 0, "retired": false}, ...}`.
fn rankings(value: &serde_json::Value) -> Vec<(String, tt::Ranking)> {
    let int = |v: &serde_json::Value, key: &str| v.get(key)?.as_i64().map(|n| n as i32);
    value
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(mode, v)| {
            Some((
                mode.clone(),
                tt::Ranking {
                    tier: int(v, "tier")?,
                    pos: int(v, "pos")?,
                    peak: int(v, "peak_tier").zip(int(v, "peak_pos")),
                    retired: v.get("retired").and_then(serde_json::Value::as_bool).unwrap_or(false),
                },
            ))
        })
        .collect()
}

fn work(mut next: Option<(u128, &'static str)>) {
    while let Some((uuid, api)) = next {
        let url = format!("{api}/v2/profile/{}/rankings", tt::hyphenated(uuid));
        // No answer (unknown player, no network): no tier, until the cache
        // is cleared.
        tt::store(uuid, get(&url).map(|v| rankings(&v)).unwrap_or_default());
        next = QUEUE.lock().unwrap_or_else(|e| e.into_inner()).pop();
    }
    BUSY.fetch_sub(1, Ordering::SeqCst);
}

/// The tier before `uuid`'s name, asking the tier list the first time.
pub fn tier(uuid: u128) -> Option<(String, u32)> {
    match tt::tier(uuid) {
        tt::Known::Tier(label, color) => Some((label, color)),
        tt::Known::Nothing => None,
        tt::Known::Ask => {
            let api = tt::settings()?.api();
            if BUSY.fetch_add(1, Ordering::SeqCst) < MOST_BUSY {
                std::thread::spawn(move || work(Some((uuid, api))));
            } else {
                BUSY.fetch_sub(1, Ordering::SeqCst);
                QUEUE.lock().unwrap_or_else(|e| e.into_inner()).push((uuid, api));
            }
            None
        }
    }
}

/// The tier list's game modes, asked for once (again after `forget_modes`).
pub fn ask_modes() {
    let Some(settings) = tt::settings() else {
        return;
    };
    if MODES_ASKED.swap(true, Ordering::SeqCst) {
        return;
    }
    let api = settings.api();
    std::thread::spawn(move || {
        let modes = get(&format!("{api}/v2/mode/list"))
            .and_then(|v| {
                Some(
                    v.as_object()?
                        .iter()
                        .map(|(id, mode)| {
                            let title = mode.get("title").and_then(|t| t.as_str()).unwrap_or(id);
                            (id.clone(), title.to_owned())
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .unwrap_or_default();
        tt::set_modes(modes);
    });
}

/// Another tier list was chosen, or the cache cleared: ask again.
pub fn forget_modes() {
    tt::set_modes(Vec::new());
    MODES_ASKED.store(false, Ordering::SeqCst);
}
