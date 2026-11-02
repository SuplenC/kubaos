# Spinner

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Spinner`

Rust `progress::spinner`; QML `Spinner`. Its mockup pages are the source of truth: Atoms, Spinner, and Molecules, Spinner in context, in `design/mockups/`.

Work that doesn't know: an icon size (16, 20 or 24 beside text; 32 and 48 for a page's one spinner), a lit stretch of neon running round an unlit glass ring. The ring is the tone at `opacity.spinner_track`; the arc, `spinner.arc` of it with round ends, is the tone's core at night with the content glow round it, and the tone by day with no glow. It turns once every `spinner.turn` at one length, never breathing, and keeps turning with reduce motion, since a spinner standing still reads as a hang. The caller passes the time.

**In context.** Its size is the icon size of the text beside it, and its gap a symbol's: 16 `space.s8` from `small` or `label` words, 20 `space.s12` from `body`, 24 from a `subheading`, 48 for a page's one, `space.s20` beside its `heading` in a card or `space.s16` above its words. A button whose work can't be measured gives its place to a 20 spinner and a `body` line saying what is going on (Kōbō's Reboot); a spinner never sits inside a button. A row of work leads with it while the work runs, and the mark of how it went (`check_circle` in `primary`, `error` in `error`) takes its place at its size when it ends. A form's work shows in words on its `small` status line under the field. Work that destroys something spins in `error`.

*Not built yet: both libraries turn the arc at 0.8 turns a second and breathe it between a quarter and 0.85 of a half turn, in the tone at night and by day.*
