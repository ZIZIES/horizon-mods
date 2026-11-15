// A player's break, before the block goes (ServerPlayerGameMode.destroyBlock):
// sneaking on a log of a tree fells the rest of it. The log they broke is
// left to the game's own break, which carries on after this.
fn tree_feller_destroy(game: &mut crate::game::Game<'_>, pos: [i32; 3]) {
    use crate::game::native_player_break::tree_feller as tf;
    if !game.native_on(tf::ID) || !game.native_authoritative() || !game.native_sneaking() {
        return;
    }
    // Only with an axe in hand.
    let axe = game.held().is_some_and(|s| {
        tf::is_axe(&s.item) || game.res.items().item_in_tag("minecraft:axes", &s.item)
    });
    if !axe {
        return;
    }
    let log = game.native_block(pos);
    if !game.block_in_tag("minecraft:logs", &log) {
        return;
    }
    // The breaks below come back through this hook: they fell nothing more.
    if !tf::begin() {
        return;
    }
    let plan = tf::plan(pos, |p| {
        let name = game.native_block(p);
        if name == log {
            tf::Cell::Log
        } else if game.block_in_tag("minecraft:leaves", &name) {
            tf::Cell::Leaf {
                distance: game
                    .native_block_property(p, "distance")
                    .and_then(|d| d.parse().ok())
                    .unwrap_or(0),
                persistent: game.native_block_property(p, "persistent").as_deref() == Some("true"),
            }
        } else {
            tf::Cell::Other
        }
    });
    if let Some(plan) = plan {
        let player = game.native_acting_id();
        let dimension = game.dim.id();
        // Where the tree stood on soil, and what grows back there, read
        // before anything goes.
        let bases: Vec<[i32; 3]> = std::iter::once(pos)
            .chain(plan.logs.iter().copied())
            .filter(|p| {
                let below = game.native_block([p[0], p[1] - 1, p[2]]);
                tf::is_soil(&below) || game.block_in_tag("minecraft:dirt", &below)
            })
            .collect();
        let leaves: Vec<String> = plan.leaves.iter().map(|p| game.native_block(*p)).collect();
        let sapling = tf::sapling_for(&leaves)
            .filter(|s| game.pack.assets.registry.default_state(s).is_some());
        // The logs as the player breaks them, with what they hold: it wears,
        // and a tool that breaks on the way ends the felling there.
        let held = |game: &crate::game::Game<'_>| {
            game.inventory[game.selected].as_ref().map(|s| s.item.clone())
        };
        let tool = held(game);
        let mut felled = true;
        for p in &plan.logs {
            // (Bare hands stay bare hands: a drop may land in the slot.)
            if tool.is_some() && held(game) != tool {
                felled = false;
                break;
            }
            game.native_player_break_block(player, *p, true);
        }
        if felled {
            // The leaves by hand, so the tool doesn't wear on them: it is
            // set aside, and their drops are kept until it is back.
            let hand = game.inventory[game.selected].take();
            let outer = std::mem::replace(&mut game.break_collect, Some(Vec::new()));
            for p in &plan.leaves {
                if !game.world.loaded(p[0], p[2]) {
                    continue;
                }
                if let Err(e) = game.player_break_block(*p) {
                    log::warn!("tree feller: the leaves at {p:?} stayed: {e}");
                }
            }
            let mut drops =
                std::mem::replace(&mut game.break_collect, outer).unwrap_or_default();
            game.inventory[game.selected] = hand;
            // A sapling a spot, out of what the leaves dropped, else out of
            // the inventory; free in creative.
            let mut planned = 0;
            if let Some(sapling) = &sapling {
                for at in &bases {
                    let mut paid = game.creative;
                    if !paid
                        && let Some(stack) =
                            drops.iter_mut().find(|d| &*d.item == sapling.as_str() && d.count > 0)
                    {
                        stack.count -= 1;
                        paid = true;
                    }
                    if !paid {
                        paid = game.native_take_items(sapling, 1) == 1;
                    }
                    if paid {
                        planned += 1;
                        tf::replant_later(tf::Replant {
                            player,
                            dimension: dimension.to_string(),
                            pos: *at,
                            sapling: sapling.clone(),
                        });
                    }
                }
            }
            log::info!(
                "tree feller: felled {} logs and {} leaves at {pos:?}; sapling {sapling:?}, {} spots on soil, {planned} to plant",
                plan.logs.len() + 1,
                plan.leaves.len(),
                bases.len(),
            );
            for stack in drops.into_iter().filter(|d| d.count > 0) {
                game.give_or_drop(stack);
            }
        } else {
            log::info!("tree feller: the tool broke, felling stopped at {pos:?}");
        }
    }
    tf::end();
}
