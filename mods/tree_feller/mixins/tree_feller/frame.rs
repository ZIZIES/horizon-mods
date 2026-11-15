// Once a frame: the saplings of the trees felled since, once the log the
// player broke is gone. Each was paid for at the felling; one that can't
// stand after all goes back to the player.
fn tree_feller_frame(game: &mut crate::game::Game<'_>) {
    use crate::game::native_player_break::tree_feller as tf;
    let pending = tf::take_replants();
    if pending.is_empty() {
        return;
    }
    for r in pending {
        let state = game.pack.assets.registry.default_state(&r.sapling);
        let here = game.native_block(r.pos);
        let below = game.native_block([r.pos[0], r.pos[1] - 1, r.pos[2]]);
        let stands = game.dim.id() == r.dimension
            && tf::is_air(&here)
            && (tf::is_soil(&below) || game.block_in_tag("minecraft:dirt", &below));
        let planted = match state {
            Some(state) if stands => game.native_set_block(&r.dimension, r.pos, state, 3),
            _ => false,
        };
        log::info!(
            "tree feller: {} at {:?} (there: {here}, below: {below}): {}",
            r.sapling,
            r.pos,
            if planted { "planted" } else { "not planted" },
        );
        if !planted && !game.creative {
            let back = horizon_client::items::ItemStack::new(&r.sapling, 1);
            game.native_as_player(r.player, |game| game.give_or_drop(back));
        }
    }
}
