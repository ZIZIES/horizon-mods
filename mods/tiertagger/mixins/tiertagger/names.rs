// TierTagger: who an entity id is, from the connection's player tables.
use crate::game::Game;

/// The UUID and profile name of the player with entity `id`: one the server
/// sent, or the player's own model.
pub(in crate::game) fn player_of(game: &Game<'_>, id: u32) -> Option<(u128, String)> {
    let r = game.remote.as_ref()?;
    if let Some(uuid) = r.level_players.get(&(id as i32)) {
        let name = r.level_player_name(id as i32, uuid)?;
        return Some((*uuid, name.to_owned()));
    }
    game.player_model
        .as_ref()
        .is_some_and(|m| m.id == id)
        .then(|| (r.profile, game.own_profile_name()))
}
