# NotificationCard

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `NotificationCard`

QML `NotificationCard`. Its mockup page is the source of truth: Molecules, NotificationCard, in `design/mockups/`. The shell's alone: only the shell shows notifications. *Not built yet: the shell draws it as `CardFace` and `NotificationCard` in `config/quickshell/modules/notifications`, not a library part; there its hover is a state layer and the tube comes only from the keyboard, and its dismiss has no hint.*

One notification, the same card in the `PopupStack`, the bell's peek and the notifications panel. It lies flat on what holds it on `surface_container`, `error_container` for a critical one; its corners are the panel's surface radius `space.s8` in (16).

- **The header band**, a small control tall (32), `space.s8` from the top and from the far corner: the app's icon or the notification's image on a `Sm` `IconTile` `space.s12` in, an accent `Badge` dot on its top right corner while unread, the app's name in `label` on one line (Notification when it has none), a `Chip` with the count on a stack's newest card, and the day and time it came in `small` and `on_surface_variant`, `space.s8` apart. At the band's end the dismiss, a Text sm `IconButton` with its hint (Dismiss, or Dismiss all N on a stack), shown while the card is lit, its room kept at rest so the band never moves; before it, while the text is cut off, a chevron in a box as big as the dismiss.
- **The words**, `space.s8` under the band and `space.s12` in at the sides: the summary in `body_strong` and the body in `body` and `on_surface_variant` `space.s4` under it, two lines each while folded, each cut after the last whole word that fits with the ellipsis (`Text`).
- **The buttons**, `space.s12` under the words, `space.s8` apart, all sm: the main action Filled (Danger on a critical card), the others Text, then Mark read while it is unread, unless the sender offers its own. A row of Text buttons starts its first label on the words' edge, its box out in the padding. `space.s12` below them is the card's foot.
- **A popup's time left**, a `stroke.thin` line in `primary` along its foot inside the padding, from full to nothing while it shows; its length is the shell's setting, `Settings.notificationTimeout`, not a token. A critical card has none: it stays until it is acted on.

**States.** A card a click does something on (its text cut off, or a stack's newest) is a control: lit, by the pointer or the keyboard, the accent's tube inside its edge with the glow outside at night, and its dismiss shows; pressed, the tube white-hot, by day `opacity.press_tint` of the accent over its fill. Its own buttons light their own lines, and the card stays at rest under them. A card with nothing to unfold takes no click: the pointer on it only shows its dismiss. Open, it is unfolded to its whole text, the chevron up.

**Motion.** Unfolding, the text's room opens to its whole height over `medium` on `standard_decel`, the whole text laid out and cut to the room as it goes; folding closes it over `short` on `standard_accel`, and the ellipsis comes back once it has closed. What holds the card moves with it. With reduce motion the room takes its height at once and the words fade in over `short`.

**Keys.** It is never a Tab stop: what holds it has the keyboard. Enter or Space unfolds or folds it, or opens a stack; Shift+Enter runs its main action; Esc folds it, and folded, goes on to what holds it. A right click opens its menu: its actions, then Open, Dismiss and Clear all.
