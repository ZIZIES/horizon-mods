// processLegacySkin is about to run on a decoded skin: note a slim-drawn one
// while its alpha still tells.
fn black_bar_concealer_note(width: u32, height: u32, rgba: &[u8]) {
    crate::skins::black_bar_concealer::note_raw(width, height, rgba);
}
