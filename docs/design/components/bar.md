# Bar

The shell's top edge, an organism of the shell built from `Pill`s: not a library component, so it has no specimen section. Its mockup pages are the source of truth: Shell surfaces, Bar, Bar contents, Workspaces, Tray and Recording and countdowns, in `design/mockups/`. This page is the current design, for Kuba to try and change.

## What sits where

The bar is `size.bar` (40) tall along the frame's top. Its pills are `control_sm` (32), `space.s8` apart and `space.s8` in from its ends.

- **The left holds what you're doing**, in this order: the workspaces, the live preview, recording, Keep awake, the Proxy, the keyboard layout, the update, then the focused window's title, `space.s12` after the last pill.
- **The clock is always centred** on the bar, whatever either side holds.
- **The right holds what opens a panel**: the tray (or its ⋯), the system monitor, then the bell and status, which end the bar, opening their panels from that corner.
- **No power button.** Power is in the control center.
- **The live preview is its `code` symbol alone**, on its `cold` tint, a lone-symbol pill at every width; the branch is in its hint.
- **⋯ (`more`)** is a new KubaOS Symbol, three dots on the set's grid: a lone-symbol pill holding the tray's apps once they fold, and whatever folds in with them.

Each side keeps `space.s40` clear of the clock; `space.s16` below 640 logical pixels, `space.s8` at 360. A symbol and its own value sit `space.s4` apart (see Pill).

## How it narrows

As the bar narrows, its parts give way in one order, the same at every width, a part only while its own side runs closer to the clock than its clear space allows (a part of the clock's own, while either side does). The title shrinks first, and nothing else gives way while any of it shows; once it has under `space.s48` × 2 of room it goes. Then, in order:

1. The keyboard layout goes (the control center has it).
2. The countdown goes (the clock's popup has it).
3. Keep awake and the Proxy go (the control center has them).
4. The date drops its month, then goes; the kanji keeps the weekday.
5. Recording keeps its dot; its time is in its hint.
6. The tray's apps fold into ⋯; the update and the preview join them.
7. A borrowed group's name goes, then each group's ghost.
8. The system monitor turns compact: one `control_sm` circle of rings edge to edge, the rim CPU in `primary`, RAM in `cold`, temperature in `on_surface_variant`, each ring `space.s4` less a device hairline thick and `space.s4` from the next, over its track at `opacity.spinner_track`; a ring near its limit turns `warning`, past it `error`. The numbers are in its hint.
9. Status sheds: the battery's share, then its symbols one by one from the left; the battery's symbol stays.
10. Below 640: the workspaces show the shown one alone (a click and the wheel still move between them); the bell joins the status pill as its own leading half; recording moves onto the clock, which takes the recording pill's look (`error_container`, `on_error_container`, its `error` dot leading), where there is room for it; the clock keeps `space.s16` clear and drops its kanji.
11. At 360: ⋯ folds into the status pill as its own half before the bell, and the clock keeps `space.s8` clear.

Never gone: the clock's time, the system monitor, status's battery symbol, the bell, ⋯ once there is anything behind it, and the recording's dot. With every pill showing, the bar fits from 3840 down to 360 with the clock centred. At 360 the left has room for recording's own dot, so the clock stays its own colour; moving recording onto it there would leave the right side 16 px short.

*Not built yet: `modules/bar/Bar.qml` keeps the old arrangement (recording, Keep awake, the Proxy, the layout and the update on the right, a power `IconButton` at its end), shows the preview's branch in its pill, and hides nothing for width: only the title elides, at the room up to the clock less 40, and the right group runs under the clock on a narrow screen. The ⋯ symbol, the compact system monitor and the status pill's shedding don't exist in QML.*
