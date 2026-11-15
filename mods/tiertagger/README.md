# TierTagger for Horizon

After uku's Fabric mod (MPL-2.0); the API calls and the tier rules follow its
published source. A player's PvP tier from MCTiers or SubTiers before their
name: `HT3 | name` on name tags and in the player list.

Settings (Options > Mods > TierTagger): Gamemode (None, then each of the tier
list's modes), Show Retired, Show Highest Tier (Never / When Not Found /
Always), Show Tiers in Player List, Tierlist (MCTiers / SubTiers), Clear Tiers
Cache. Defaults match the Fabric mod's.

Network: the first time a player's name is drawn, their UUID is sent to the
chosen tier list (`mctiers.net` or `subtiers.net`), as the Fabric mod does.
Nothing else is sent. Offline-mode UUIDs are never asked about.

Not here: gamemode icons (they are glyphs of a font the Fabric mod ships), the
gamemode cycle key (use the settings button), custom tier colours, the player
search screen and the update check. On name tags the whole line (tier, team
prefix and colour) is drawn in the sixteen legacy colours, since the name tag
hook takes plain text; the player list shows the tier's exact colour.

Status: attempted. Compiles into the game; not played yet.
