# Scroll

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Scroll`

Rust `layout::scroll`; QML `Scroll`.

A page's content taller than its room. The thumb is 4 wide (`scrollbar`), on the right edge of its box, a pill in the outline variant (the outline while the pointer is on it), floating over the content so it takes no width: content that runs to the edge leaves 16 on its right. A scroll clips what it scrolls, a focus glow included, only at an edge you can see. A page's or a panel's scroll has no edge of its own, so it reaches past its box into the room around it, as far as a glow reaches (`focus.far`, 20), and above and below up to the part of the page there (`room_above`, `room_below`, `roomAbove`, `roomBelow`: Kōbō's header and footer, the wallpaper picker's tabs), with its content that far in: the content lines up with what is around it, a resting control at its edge keeps its whole glow, content scrolled past its box shows in that room and stops at the part beside it, and the panel's or window's own edge cuts whatever passes it. In iced, whose content leaves 16 on its right for the thumb, it reaches the 4 of the glow past that room on its right, so the thumb keeps its place and never takes the pointer from the content; in QML it reaches the glow there too, and the thumb floats over the content's last 4. A scroll with an edge of its own (a list's well, a log, a menu) clips at that edge.

A scroll never rests past the end of its content: content that shrinks under it, or a glide that set out for where its end was, ends at its new end.
