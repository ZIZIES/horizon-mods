// Tree Feller for Horizon: sneak and break one log of a tree, and the rest of
// its logs and its leaves break too, then a sapling goes back where the trunk
// stood. This file is the part that needs nothing of the game: which blocks
// are the tree, which sapling it grows from, and the saplings waiting to be
// planted. The bundle adds it as a module beside the game's player break.
use std::collections::{HashSet, VecDeque};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

pub const ID: &str = "tree_feller";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "Sneak and break a log with an axe to fell the whole tree: its logs and leaves break, the drops go to your inventory and a sapling is replanted.";

/// More logs than this is no tree (a log wall, a cabin): nothing is felled.
pub const MAX_LOGS: usize = 512;
pub const MAX_LEAVES: usize = 4096;

pub type Pos = [i32; 3];

/// What the game says is at a position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    /// The same log as the one broken.
    Log,
    /// Leaves, with their `distance` to the nearest log and whether a player
    /// placed them.
    Leaf { distance: u8, persistent: bool },
    Other,
}

/// The rest of a tree: its other logs, the highest first, and its leaves.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Plan {
    pub logs: Vec<Pos>,
    pub leaves: Vec<Pos>,
}

/// The tree the log at `start` belongs to. Logs join through their 26
/// neighbours (acacia and big oak branches run diagonally). Leaves belong
/// when a path of leaves leads to them from a log with `distance` going up
/// by one each step, as the game counts it, so a neighbouring tree keeps the
/// leaves nearer its own trunk. None for what isn't a grown tree: logs with
/// no such leaves, or more than `MAX_LOGS` of them.
pub fn plan(start: Pos, mut cell: impl FnMut(Pos) -> Cell) -> Option<Plan> {
    let mut logs = vec![start];
    let mut seen: HashSet<Pos> = HashSet::from([start]);
    let mut next = 0;
    while next < logs.len() {
        let at = logs[next];
        next += 1;
        for dy in -1..=1 {
            for dx in -1..=1 {
                for dz in -1..=1 {
                    let p = [at[0] + dx, at[1] + dy, at[2] + dz];
                    if p != at && !seen.contains(&p) && cell(p) == Cell::Log {
                        if logs.len() >= MAX_LOGS {
                            return None;
                        }
                        seen.insert(p);
                        logs.push(p);
                    }
                }
            }
        }
    }
    let mut leaves = Vec::new();
    let mut queue: VecDeque<(Pos, u8)> = logs.iter().map(|p| (*p, 0)).collect();
    while let Some((at, distance)) = queue.pop_front() {
        for (dx, dy, dz) in [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)] {
            let p = [at[0] + dx, at[1] + dy, at[2] + dz];
            if seen.contains(&p) || leaves.len() >= MAX_LEAVES {
                continue;
            }
            if let Cell::Leaf { distance: d, persistent: false } = cell(p)
                && d == distance + 1
            {
                seen.insert(p);
                leaves.push(p);
                queue.push_back((p, d));
            }
        }
    }
    if leaves.is_empty() {
        return None;
    }
    logs.remove(0);
    logs.sort_by(|a, b| b[1].cmp(&a[1]));
    Some(Plan { logs, leaves })
}

/// The sapling a tree with these leaves grows from: the commonest leaves
/// decide (an azalea tree has two kinds).
pub fn sapling_for(leaves: &[String]) -> Option<String> {
    let mut counts: Vec<(&str, usize)> = Vec::new();
    for leaf in leaves {
        match counts.iter_mut().find(|(name, _)| *name == leaf.as_str()) {
            Some((_, n)) => *n += 1,
            None => counts.push((leaf, 1)),
        }
    }
    let (leaf, _) = counts.into_iter().max_by_key(|(_, n)| *n)?;
    let kind = leaf.strip_suffix("_leaves")?;
    Some(match kind {
        "minecraft:azalea" | "minecraft:flowering_azalea" => kind.to_string(),
        "minecraft:mangrove" => "minecraft:mangrove_propagule".to_string(),
        _ => format!("{kind}_sapling"),
    })
}

/// Ground a sapling stands on, should the game's `dirt` tag not say.
pub fn is_soil(block: &str) -> bool {
    matches!(
        block.strip_prefix("minecraft:").unwrap_or(block),
        "grass_block"
            | "dirt"
            | "coarse_dirt"
            | "podzol"
            | "rooted_dirt"
            | "mycelium"
            | "moss_block"
            | "pale_moss_block"
            | "mud"
            | "muddy_mangrove_roots"
            | "farmland"
    )
}

/// An axe by its name, should the game's `axes` tag not say.
pub fn is_axe(item: &str) -> bool {
    item.ends_with("_axe")
}

pub fn is_air(block: &str) -> bool {
    matches!(block, "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air")
}

/// True while a tree is being felled: the breaks it makes are heard by the
/// same hook and must not fell again.
static FELLING: AtomicBool = AtomicBool::new(false);

/// Starts a felling; false when one is already running.
pub fn begin() -> bool {
    !FELLING.swap(true, Ordering::Relaxed)
}

pub fn end() {
    FELLING.store(false, Ordering::Relaxed);
}

/// A sapling to plant once the broken log is gone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Replant {
    pub player: u128,
    pub dimension: String,
    pub pos: Pos,
    pub sapling: String,
}

static REPLANTS: Mutex<Vec<Replant>> = Mutex::new(Vec::new());

pub fn replant_later(replant: Replant) {
    REPLANTS.lock().unwrap_or_else(|e| e.into_inner()).push(replant);
}

pub fn take_replants() -> Vec<Replant> {
    std::mem::take(&mut *REPLANTS.lock().unwrap_or_else(|e| e.into_inner()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn world(cells: &[(Pos, Cell)]) -> impl FnMut(Pos) -> Cell {
        let map: HashMap<Pos, Cell> = cells.iter().copied().collect();
        move |p| map.get(&p).copied().unwrap_or(Cell::Other)
    }
    fn leaf(distance: u8) -> Cell {
        Cell::Leaf { distance, persistent: false }
    }

    #[test]
    fn a_small_tree_is_felled_whole_from_any_log() {
        let cells = [
            ([0, 0, 0], Cell::Log),
            ([0, 1, 0], Cell::Log),
            ([0, 2, 0], Cell::Log),
            ([0, 3, 0], leaf(1)),
            ([1, 2, 0], leaf(1)),
            ([2, 2, 0], leaf(2)),
        ];
        let plan = plan([0, 1, 0], world(&cells)).unwrap();
        assert_eq!(plan.logs, vec![[0, 2, 0], [0, 0, 0]]);
        assert_eq!(plan.leaves.len(), 3);
    }

    #[test]
    fn diagonal_branches_count() {
        let cells = [([0, 0, 0], Cell::Log), ([1, 1, 1], Cell::Log), ([1, 2, 1], leaf(1))];
        let plan = plan([0, 0, 0], world(&cells)).unwrap();
        assert_eq!(plan.logs, vec![[1, 1, 1]]);
    }

    #[test]
    fn logs_without_grown_leaves_are_a_build() {
        let placed = Cell::Leaf { distance: 1, persistent: true };
        let cells = [([0, 0, 0], Cell::Log), ([0, 1, 0], Cell::Log), ([0, 2, 0], placed)];
        assert_eq!(plan([0, 0, 0], world(&cells)), None);
    }

    #[test]
    fn a_neighbours_leaves_stay() {
        // Leaves at distance 1 from another trunk, two away from this one.
        let cells = [([0, 0, 0], Cell::Log), ([1, 0, 0], leaf(1)), ([2, 0, 0], leaf(1))];
        let plan = plan([0, 0, 0], world(&cells)).unwrap();
        assert_eq!(plan.leaves, vec![[1, 0, 0]]);
    }

    #[test]
    fn a_wall_of_logs_is_left() {
        assert_eq!(plan([0, 0, 0], |p| if p[1] == 0 { Cell::Log } else { leaf(1) }), None);
    }

    #[test]
    fn axes_only() {
        assert!(is_axe("minecraft:diamond_axe"));
        assert!(!is_axe("minecraft:diamond_pickaxe"));
        assert!(!is_axe("minecraft:oak_log"));
    }

    #[test]
    fn saplings_by_leaves() {
        let of = |names: &[&str]| sapling_for(&names.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        assert_eq!(of(&["minecraft:oak_leaves"]).as_deref(), Some("minecraft:oak_sapling"));
        assert_eq!(of(&["minecraft:mangrove_leaves"]).as_deref(), Some("minecraft:mangrove_propagule"));
        assert_eq!(
            of(&["minecraft:azalea_leaves", "minecraft:flowering_azalea_leaves", "minecraft:azalea_leaves"]).as_deref(),
            Some("minecraft:azalea")
        );
        assert_eq!(of(&[]), None);
    }

    #[test]
    fn one_felling_at_a_time_and_replants_wait() {
        assert!(begin());
        assert!(!begin());
        end();
        assert!(begin());
        end();
        let r = Replant { player: 1, dimension: "minecraft:overworld".into(), pos: [0, 64, 0], sapling: "minecraft:oak_sapling".into() };
        replant_later(r.clone());
        assert_eq!(take_replants(), vec![r]);
        assert!(take_replants().is_empty());
    }
}
