# Marquee

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps.

## `Marquee`

New: in neither library yet; its mockup page, Marquee, is the source of truth.

Words in a room of fixed width that scroll when they run past it, so nothing beside them moves when they change: now playing's song and artist in the bar. The room is a `size` token its owner names (`now_playing`, 192), whatever the words; the words are one run in parts, each in its own type style and colour (a song in `body` and `on_surface`, a middle dot and the artist in `on_surface_variant`), one line tall.

Words that fit sit still at the room's start, nothing faded. Words that run past it are cut at the room's edges, with a soft fade `space.s16` wide only where words are cut: at the end while they run past, at the start as far as they have moved, up to `space.s16`. It never ends in an ellipsis while motion is allowed.

It isn't a control: it takes no pointer and no focus, and has no lit, pressed, disabled, selected, loading or error state. Its accessible name is the words whole.

**Motion.** Words that run past the room hold at their start for `marquee_hold` (2400), so the start is read first, then scroll left on `linear` at `space.s32` per `marquee_pace` (1000), 32 a second, slow enough to read in the corner of the eye. A copy of the words follows them `space.s32` behind; the pass ends when the copy stands where the words began, which is the first frame again, so the loop never jumps, and it holds again. A pass takes as long as the words are wide; it never goes backwards. The start's fade grows with the first `space.s16` of travel and shrinks as the copy lands.

- **Held** (its owner paused): the words glide to the nearer start, the words' or the copy's, over `medium` on `standard`, and stay there until let go, when the pass starts again with its hold.
- **New words**: the old fade out where they stand over `micro` on `standard_accel`; then the new fade in over `short` on `standard` and rise `space.s8` over `medium` on `emphasized_decel`, at the room's start, and their pass starts with its hold.
- **Reduce motion**: nothing scrolls. Words that run past the room end in an ellipsis at its edge, the part that crosses the edge cut and the parts after it gone, with no fade; new words change in place.
