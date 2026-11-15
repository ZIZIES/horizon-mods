// /librarian and /lb.
fn librarian_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    crate::game::librarian_glue::command(game, command, ran);
}
