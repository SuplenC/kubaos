# DepthPicture

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `DepthPicture`

QML `DepthPicture`. Its mockup page is the source of truth: Molecules, DepthPicture, in `design/mockups/`.

A wallpaper with its depth (ADR 0004, 0005): three layers the size of its box, each laid as `cover` lays the picture. The picture; its plate, the picture with the city painted out a little past its outline, transparent elsewhere; and its city, the picture itself drawn through the cut-out's alpha, so with nothing pushed the three are the picture exactly. A picture without a cut-out and a plate is drawn flat.

- **The push** (`--push`, 0 to 1): the picture and the plate grow to `scale.push` and the city to `scale.near`, all round the middle of the bottom edge, where the city stands, so its roofs only grow over the sky and what it uncovers is the plate's band. The plate fades in with the push.
- **Depth on or off** (`--depth`, 1 to 0): off, the city's scale settles to the picture's and the plate goes, over `long` on `emphasized_decel`, so turning it off moves rather than snaps; once settled, the plate and the city aren't drawn.
- **Sliding past** (`--city-lead`): a picture browsed onto starts with its city `space.s24` ahead of its sky, which eases to nothing over `long` on `spring`, so the city travels faster than the sky.
- **Reduce motion** holds the picture at 1:1 and swaps depth at once; the layers still stand in front of one another.

The Lock draws these layers with its clock between the plate and the city; the wallpaper picker's wall draws them on every picture.

*Not built yet: no library has it. The Lock's QML (also not built) draws the same three layers.*
