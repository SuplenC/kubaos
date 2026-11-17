# Meter

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Meter`

Rust `meter::meter`; QML `Meter`. Its mockup page is the source of truth: Molecules, Meter, in `design/mockups/`.

A reading in parts on a progress bar's track: 8 tall, a pill, each part a share and its colour, left to right (memory as used and cache), every part keeping the bar's rounded ends. It has no glow. A new reading eases each part over `short` on `linear`, as a progress bar follows its work. The system monitor's memory, disks and battery across a card, and the bar pill's bars, a small control (`control_sm`) wide beside a 16 symbol.

A reading steps by its threshold in `sysmon.toml`: `primary` while fine, `warning` within its margin of the threshold (10 for memory and the CPU's temperature, 5 for a disk and the battery's draw), `error` past it; its figure beside it takes the same step, and cache keeps `tertiary`. Those are the fixed palette's roles, the same on every wallpaper.

**Live.** A mic's level as it comes, the Audio panel's under the mic's slider: one `primary` part that jumps to each level with no easing, and a peak tick, a `stroke.thin` line in `primary` the bar's height at the highest level lately, clipped to the pill so at either end it takes the round end. The tick holds for `slow`, then falls the whole bar per `scene` on `linear`, never under the level. Muted, it is the track alone, with no tick. Disabled, with no mic to hear (none at all, or a headset in Music mode), its track dims as a disabled slider's does (`opacity.disabled_content` of the track's own), empty, and it takes no level.

*Not built yet: `kubaos-ui`'s meter doesn't ease a new reading, and neither library has the live variant.*
