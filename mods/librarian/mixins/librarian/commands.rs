// /librarian and /lb, for the chat box to complete.
fn librarian_commands(commands: &mut Vec<&'static str>) {
    crate::game::librarian_glue::commands(commands);
}
