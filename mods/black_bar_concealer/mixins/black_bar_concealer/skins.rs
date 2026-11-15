// updateRemoteSkins: let the game resolve every player's skin and model,
// then give the slim-drawn skins it put on the wide model the slim one, or
// widen their arms.
fn black_bar_concealer_skins(game: &mut crate::game::Game<'_>) -> std::ops::ControlFlow<()> {
    use horizon_client::skins::black_bar_concealer as bbc;
    use std::ops::ControlFlow;
    // The game's own pass, asked for below.
    if bbc::inside() || !game.native_on(bbc::ID) {
        return ControlFlow::Continue(());
    }
    bbc::set_inside(true);
    game.update_remote_skins();
    bbc::set_inside(false);
    let settings = {
        let game = &*game;
        bbc::settings(|key| game.native_setting(bbc::ID, key))
    };
    // The wide players wearing a downloaded skin: the level's (by index) and
    // the player's own model (None).
    let wide = |m: &horizon_client::mobs::Mob| {
        (!m.slim)
            .then(|| m.skin.clone())
            .flatten()
            .filter(|s| s.starts_with("skins/"))
    };
    let mut wearers: Vec<(Option<usize>, String)> = game
        .mobs
        .iter()
        .enumerate()
        .filter(|(_, m)| &*m.kind.id == "minecraft:player")
        .filter_map(|(i, m)| Some((Some(i), wide(m)?)))
        .collect();
    if let Some(skin) = game.player_model.as_ref().and_then(wide) {
        wearers.push((None, skin));
    }
    for (at, skin) in wearers {
        let slim_drawn = {
            let downloads = game.skin_downloads.borrow();
            bbc::known(&skin, || downloads.image(&skin))
        };
        match bbc::fix(settings, slim_drawn, bbc::widened(&skin)) {
            bbc::Fix::Leave => {}
            bbc::Fix::Slim => {
                let kind = match at {
                    Some(i) => Some(game.mobs[i].kind.clone()),
                    None => game.player_model.as_ref().map(|m| m.kind.clone()),
                };
                let skin_3d = kind.and_then(|kind| game.skin_3d_model(&kind, &skin, true));
                let mob = match at {
                    Some(i) => Some(&mut game.mobs[i]),
                    None => game.player_model.as_mut(),
                };
                if let Some(mob) = mob {
                    mob.slim = true;
                    mob.skin_3d = skin_3d;
                }
            }
            bbc::Fix::Widen => {
                let image = game.skin_downloads.borrow().image(&skin).map(<[u8]>::to_vec);
                if let Some(mut image) = image {
                    bbc::widen(&mut image);
                    bbc::set_widened(&skin);
                    game.store_skin(skin, image);
                }
            }
        }
    }
    ControlFlow::Break(())
}
