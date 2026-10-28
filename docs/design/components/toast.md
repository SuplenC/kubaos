# Toast

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Toast`

Rust `toast::toast`; QML `Toast`.

One `body` line saying what just happened, in the inverse colours so it reads over anything, an optional 20 icon 12 from the text, padding 16 × 12, the surface radius (24), which on one line is a pill. It arrives over `short`, rising 32 into place as it fades in (`emphasized_decel`), stays 3.6 s, and leaves over `medium`, fading out (`standard_accel`): `toast::toast_at` draws it at an age.
