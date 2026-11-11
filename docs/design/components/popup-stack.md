# PopupStack

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `PopupStack`

QML `PopupStack`, the shell's alone. Its mockup page is the source of truth: Shell surfaces, PopupStack, in `design/mockups/`. It isn't a `Toast`: a toast says in one line what just happened in its window; the popup stack shows notifications. *Not built yet: the shell's files are still named `Popups.qml`, `ToastList.qml`, `PopupHold.qml` and `PopupsBelow.qml`; they become `PopupStack` with the build's move to the library.*

New notifications, on a corner `Sheet` hung from the frame's top right corner, `size.panel` (384) wide, `space.s8` inside, one app's in a `NotificationGroup`, `space.s8` apart, newest first, at most five groups. Each shows its time left and leaves when it is up; the time is the shell's setting, `Settings.notificationTimeout`, or what the notification asks for, never a token. A critical one stays until it is acted on. On the screen with the keyboard only; with no popup to show, the bell's peek takes the same place while the bar asks for it.

- **Hold:** while the pointer is anywhere on the stack, the gaps between its cards included, every card it showed when the pointer came stays, its time stopped where it was, and a new one waits, so nothing moves under the pointer. When the pointer leaves each goes on with the time it had left and the waiting ones come in. A card dismissed or acted on while held still goes.
- **Covered:** while an open panel covers the corner, the popups wait, still in the history, and come back when it has gone. Critical ones show under that panel instead, on a right-edge sheet starting `radius.surface` and `space.s12` below it; where there is no room under it (the panels that reach the frame's foot) they wait with the rest. The stack keeps its place as the panel closes, so it retracts where it is. Opening the notifications panel is seeing them, so they leave for good.

**Motion.** A popup arrives sliding in `space.s64` from the right over `long` on `emphasized_decel` as it fades in over `short` on `standard`; it leaves sliding out twice that over `medium` on `emphasized_accel` as it fades over `medium` on `standard_accel`; the others close up over `medium` on `standard_decel`. The sheet's size follows a group moving between its states frame by frame. With reduce motion a popup only fades, over `short`.

It never takes the keyboard: the pointer and its cards' buttons do, and a right click opens a card's menu over a modal panel of its own.
