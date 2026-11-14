# PopupStack

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `PopupStack`

QML `PopupStack`, the shell's alone. Its mockup page is the source of truth: Shell surfaces, PopupStack, in `design/mockups/`. It isn't a `Toast`: a toast says in one line what just happened in its window; the popup stack shows notifications. *Not built yet: the shell's files are still named `Popups.qml`, `ToastList.qml`, `PopupHold.qml` and `PopupsBelow.qml`; they become `PopupStack` with the build's move to the library.*

New notifications, on a corner `Sheet` hung from the frame's top right corner, `size.panel` (384) wide, `space.s8` in from its edges: one `NotificationPopup` per app, two lines each, `space.s4` apart, newest first. A critical one is today's full `NotificationCard` among them, with no timer: it stays until it is acted on or dismissed. On the screen with the keyboard only; with no popup to show, the bell's peek takes the same place while the bar asks for it. No popup carries the unread dot: the bell in the bar does, while anything is unread.

- **A timer each:** every popup runs its own time down, a hairline at its foot, and leaves by itself when it is up; the time is the shell's setting, `Settings.notificationTimeout`, or what the notification asks for, never a token.
- **Hold:** while the pointer is on any popup, every popup's time stops where it was; they go on together when it leaves the stack. An opened popup's time stays held until it folds.
- **Several from one app** make one popup: the count in its symbol's place, 99+ past 99, the newest's words, and its time from the whole again with each new one; it keeps its place.
- **Opening:** a click or Enter opens a popup in place into its card, one at a time; a second click on its words, Esc or a click outside folds it. Delete dismisses it, as its × does.
- **Covered:** while an open panel covers the corner, the popups wait, still in the history, and come back when it has gone. Critical ones show under that panel instead, on a right-edge sheet starting `radius.surface` and `space.s12` below it; where there is no room under it (the panels that reach the frame's foot) they wait with the rest. The stack keeps its place as the panel closes, so it retracts where it is. Opening the notifications panel is seeing them, so they leave for good.

**Motion.** A popup arrives sliding in `space.s64` from the right over `long` on `emphasized_decel` as it fades in over `short` on `standard`, its room opening under the others over `medium` on `standard_decel`. Dismissed, it slides out to the right twice that over `medium` on `emphasized_accel` as it fades over `medium` on `standard_accel`, and then those below close the gap over `medium` on `standard_decel`. Its time up, it rises `space.s24` as it fades over `medium` on `emphasized_accel`, while those below close the gap. The sheet's size follows them frame by frame. With reduce motion each only fades, over `short`, and the gap closes at once.

**Keys.** Tab reaches each popup, newest first, and a critical card; Enter opens or folds a popup, Delete dismisses, Esc folds. It never takes the keyboard by itself: a popup arriving leaves it where it was. A right click opens a card's menu over a modal panel of its own.

*Not built yet: the shell's stack shows today's full cards, one `CardGroup` per app, and holds them all on one countdown each with the pointer anywhere on the stack; it has no two-line popup, no timer held while open, no slide up when time runs out, and its cards carry the unread dot.*
