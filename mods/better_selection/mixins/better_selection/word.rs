// EditBox.getWordPosition: the mod's word boundaries.
fn better_selection_word(value: &str, dir: isize, from: usize) -> std::ops::ControlFlow<usize> {
    use crate::ui::better_selection as bs;
    if !bs::on() {
        return std::ops::ControlFlow::Continue(());
    }
    std::ops::ControlFlow::Break(bs::word_position(value, dir, from))
}
