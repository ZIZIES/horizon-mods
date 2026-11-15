// Once a frame: the saplings of the trees felled since, once the log the
// player broke is gone. One sapling a spot, out of the player's inventory
// (the leaves' drops went there); free in creative.
fn tree_feller_frame(game: &mut crate::game::Game<'_>) {
    use crate::game::native_player_break::tree_feller as tf;
    let pending = tf::take_replants();
    if pending.is_empty() || !game.native_authoritative() {
        return;
    }
    for r in pending {
        let Some(state) = game.pack.assets.registry.default_state(&r.sapling) else {
            continue;
        };
        game.native_as_player(r.player, |game| {
            let below = game.native_block([r.pos[0], r.pos[1] - 1, r.pos[2]]);
            if game.dim.id() != r.dimension
                || !tf::is_air(&game.native_block(r.pos))
                || !(tf::is_soil(&below) || game.block_in_tag("minecraft:dirt", &below))
            {
                return;
            }
            if !game.creative && game.native_take_items(&r.sapling, 1) == 0 {
                return;
            }
            game.native_set_block(&r.dimension, r.pos, state, 3);
        });
    }
}
