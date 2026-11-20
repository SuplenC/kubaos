# Bar

The shell's top edge, an organism of the shell built from `Pill`s: not a library component, so it has no specimen section. Its mockup pages are the source of truth: Shell surfaces, Bar, Bar contents, Workspaces, Tray, Recording and Keep awake, and Clock popup, in `design/mockups/`. This page is the current design, for Kuba to try and change.

## What sits where

The bar is `size.bar` (40) tall along the frame's top. Its pills are `control_sm` (32), `space.s8` apart and `space.s8` in from its ends.

- **The left holds what you're doing**, in this order: the workspaces, the live preview, recording, Keep awake, the Proxy, the keyboard layout, the update, then the focused window's title, `space.s12` after the last pill, and now playing at the left's end, beside the clock `space.s8` away, as the bar's pills stand from each other, the title between the pills and it.
- **Now playing** shows while a player plays or is paused: the player's icon, which opens the players Menu under it, the song and its artist in a `Marquee` `size.now_playing` (192) wide, then previous, play or pause on the accent's fill, next, each a circle a chip tall, `space.s4` in and apart; the song's room stands `space.s8` from the circles either side. It has two widths, wide and folded (the icon and the controls), and nothing else changes it: a song that runs past its room scrolls in it, and a new song fades in where the old one was. Between the two widths the song fades as its room closes or opens, over `medium` on `standard_decel`, as a pill whose content changes does. Paused, the song holds at its start: moving words would say it still plays.
- **The clock's time is the bar's exact centre**, whatever either side holds. The clock pill reads the weekday's seal (its kanji on a disc of `cold`), the date plain (`body`, `on_surface_variant`), the time in a flat capsule of `primary` (`on_primary`), and home's weather plain like the date (its symbol and the temperature; none with no home city); the seal and the capsule stand `space.s4` in from the pill's curves, a plain part at an end `space.s12`. The pill is laid out round the capsule. The time is `body`, Geist's own proportional figures, centred in a slot as wide as the widest time a 24-hour clock shows, as the lock's bar clock is: the pill keeps its width as the minutes turn. Clicking the time opens the clock popup; clicking the temperature opens it on home's weather.
- **The right holds what opens a panel**: the tray (or its ⋯), the system monitor, the mute pill, then the bell and status, which end the bar, opening their panels from that corner.
- **The bell carries the unread dot**: the accent's `Badge` dot on its symbol's top right corner while any notification is unread, under Do not disturb too, where the bell is crossed out. The popups carry no dot of their own, since a popup is new by being one; the notifications panel's and the peek's cards keep theirs, where read and unread sit side by side. *Not built yet: the shell's bell shows no dot, and its popups' cards carry one.*
- **No power button.** Power is in the control center.
- **The live preview is its `code` symbol alone**, on its `cold` tint, a lone-symbol pill at every width; the branch is in its hint.
- **The tray's menu** is our `Menu` on a popover `space.s12` under the app's icon, padded 4 round the menu as every menu is; an entry that opens a submenu ends in a chevron (see Menu).
- **⋯ (`more`)** is a new KubaOS Symbol, three dots on the set's grid: a lone-symbol pill holding the tray's apps once they fold, and whatever folds in with them.

Each side keeps `space.s40` clear of the clock at every width; now playing, a pill, stands `space.s8` from it, and keeps the title its clear space away. A symbol and its own value sit `space.s4` apart (see Pill).

## How it narrows

As the bar narrows, its parts give way in one order, the same at every width, a part only while its own side runs closer to the clock than its clear space allows (a part of the clock's own, while either side does). Now playing folds first, before the title is cut: when the left runs short, or the title would have less room than its own width or the song's room, whichever is less, or under `space.s48` × 2. Its song comes back whenever that no longer costs the title anything: once the title has gone, or while it keeps that room with the pill wide, and the wide pill still fits its side. The title shrinks next, and nothing else gives way while any of it shows; once it has under `space.s48` × 2 of room it goes. Then, in order:

1. The keyboard layout goes (the control center has it).
2. The clock's weather goes (the clock's popup has it).
3. Keep awake and the Proxy go (the control center has them).
4. The date drops its month, then goes; the seal keeps the weekday.
5. Recording joins the clock; its time is in its hint.
6. The tray's apps fold into ⋯; the update and the preview join them.
7. A borrowed group's name goes, then each group's ghost.
8. Now playing goes where even folded it doesn't fit, as at 540 with every pill on: its player's own window and the media keys have it.
9. The system monitor turns compact: one `control_sm` circle of rings edge to edge, CPU the rim, RAM inside it, temperature innermost, each ring `space.s4` less a device hairline thick and `space.s4` from the next, over its track at `opacity.spinner_track`. Every ring has one palette, its place saying which reading it is: `primary` at ease, `warning` near its limit, `error` past it. Its hint names each with its number.
10. Status sheds: the battery's share, then its symbols one by one from the left; the battery's symbol stays.
11. The mute pill keeps the mic's circle alone; the output is left to its keys and the OSD.
12. The clock's seal goes, last of all: the time never goes.
Never gone: the clock's time, the system monitor, status's battery symbol, the bell, ⋯ once there is anything behind it, and the recording's dot. The steps are 3840, 2560, 1920, 1440, 1280, 1024, 640 and 540, a portrait screen at scale 2: with every pill showing, the bar fits all of them with the clock centred. A phone's width is a topic of its own.

When recording merges into the clock, the clock takes the recording's colours: the pill `error_container` with `on_error_container`, its `error` dot leading, the capsule `error` with `on_error`.

*Not built yet: `modules/bar/Bar.qml` keeps the old arrangement (recording, Keep awake, the Proxy, the layout and the update on the right, a power `IconButton` at its end), shows the preview's branch in its pill, and hides nothing for width: only the title elides, at the room up to the clock less 40, and the right group runs under the clock on a narrow screen. The ⋯ symbol, the compact system monitor and the status pill's shedding don't exist in QML. `Clock.qml` draws the time in `mono_large`, and under a minute its countdown shows only its name. `Tray.qml` pads its menu's popover 12, and its submenu entries have no chevron. The power menu's dialog dim (`opacity.scrim`) is the library's screen's; the power menu itself is drawn elsewhere.*
