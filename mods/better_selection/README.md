# Better Selection for Horizon

After MDLC01's Fabric mod (Unlicense). In every single-line text field (chat,
commands, search boxes, server addresses, anvils):

- Ctrl+Left/Right, Ctrl+Backspace/Delete stop at word boundaries the way an
  editor does: letters and digits are one run, punctuation another, so
  `minecraft:diamond_sword` is three stops, not one.
- Plain Left/Right with a selection puts the cursor at that edge of the
  selection instead of moving a character.
- A double click selects just the word (or the run of punctuation or spaces)
  under the pointer.

Book pages keep the game's own behaviour. No settings.

Status: attempted. Compiles into the game; not played yet.
