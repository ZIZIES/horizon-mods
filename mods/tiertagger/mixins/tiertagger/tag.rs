// A name tag is about to be drawn: a player's gets their tier before the
// name. The game takes plain text with legacy codes here, so the name is
// handed back with its team's prefix, suffix and colour written as codes.
fn tiertagger_tag(
    game: &crate::game::Game<'_>,
    entity_id: u32,
    kind: &str,
    text: &mut Option<String>,
) {
    use horizon_client::tab_list::{tiertagger as tt, tiertagger_net as net};
    use horizon_client::text::Text;
    if kind != "minecraft:player" || text.is_some() || !tt::on() {
        return;
    }
    let Some((uuid, name)) = crate::game::remote::tiertagger_names::player_of(game, entity_id)
    else {
        return;
    };
    let Some((label, color)) = net::tier(uuid) else {
        return;
    };
    let member = Text::literal(format!("{}{name}", tt::legacy_prefix(&label, color)));
    let shown = match game.mob_team(entity_id) {
        Some(team) => team.formatted_name(member),
        None => member,
    }
    .resolve_legacy();
    let is = |flag: Option<bool>| flag == Some(true);
    *text = Some(tt::flatten(shown.chars().map(|(c, s)| {
        let decorations = [
            is(s.bold),
            is(s.italic),
            is(s.underlined),
            is(s.strikethrough),
            is(s.obfuscated),
        ];
        (c, (s.color, decorations))
    })));
}
