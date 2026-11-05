// The List page, with ListRow and ListRows: `docs/design/components/list.md`. Rows in a well,
// moved through with the keys; one build with every raised list, at a window list's scale. The row
// the keyboard is on is lit on the secondary container, and the pointer moving onto a row puts the
// keyboard on it.

import type { Words } from "../../lib/content.ts";
import { stack, within } from "../../lib/layout.ts";
import { type ListOptions, type ListRowSpec, list, type RowPinned } from "../../lib/list.ts";
import { text } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty } from "../../site/page.ts";
import { page } from "../../site/legacy.ts";

/** A panel's body, where a list's well lives: the control center's width less its padding. */
export const WELL_ROOM = "calc(var(--size-panel) - 2 * var(--container-0-padding))";

const TITLE: Words = { short: "Polish", long: "English (United States, international with dead keys)" };
const SUBTITLE: Words = { short: "WPA2", long: "Secured · WPA2 Enterprise · connected automatically" };

/** The control center's keyboard layouts: the default first, on the accent's star. */
const LAYOUTS: readonly ListRowSpec[] = [
  { title: "English (US)", subtitle: "Default", icon: "star", tile: true, accent: true, checked: true },
  { title: "Polish", subtitle: "Programmer", icon: "keyboard", tile: true, checked: false },
  { title: "Japanese", subtitle: "Kana", icon: "keyboard", tile: true, checked: false },
  { title: "German", subtitle: "No dead keys", icon: "keyboard", tile: true, checked: false, disabled: true },
];

/** The control center's Wi-Fi networks. */
const NETWORKS: readonly ListRowSpec[] = [
  { title: "Hoshi 5G", subtitle: "Connected · WPA2", subtitleIcon: "lock", icon: "wifi", checked: true },
  { title: "Kissa", subtitle: "Secured · WPA3", subtitleIcon: "lock", icon: "network_wifi_3_bar", checked: false },
  { title: "Station Free Wi-Fi", subtitle: "Open", icon: "network_wifi_1_bar", checked: false },
];

/** A settings app's pages: each row opens a page of its own, so it ends in a chevron and never ticks. */
const PAGES: readonly ListRowSpec[] = [
  { title: "Wi-Fi", subtitle: "Hoshi 5G", icon: "wifi", opens: true },
  { title: "Bluetooth", subtitle: "2 connected", icon: "bluetooth", opens: true },
  { title: "Displays", subtitle: "2 screens", icon: "desktop_windows", opens: true },
];

/** The Command menu's Results: tall, tiled, the query's matches lit, keys on the current one. */
const RESULTS: readonly ListRowSpec[] = [
  { title: "Firefox", matched: [{ start: 0, end: 3 }], subtitle: "Web browser", icon: "public", tile: true, tall: true, clear: true, tight: true, keys: "Enter" },
  { title: "Files", matched: [{ start: 0, end: 1 }, { start: 2, end: 3 }], subtitle: "Browse the disk", icon: "folder", tile: true, tall: true, clear: true, tight: true, keys: "Enter" },
  { title: "Focus stutter: off", matched: [{ start: 0, end: 1 }], icon: "bolt", tile: true, tall: true, clear: true, tight: true, keys: "Enter" },
];

/** The power menu: a danger row last. */
const POWER: readonly ListRowSpec[] = [
  { title: "Lock", icon: "lock", keys: "Super+L" },
  { title: "Sleep", icon: "bedtime" },
  { title: "Restart", icon: "restart_alt" },
  { title: "Shut down", icon: "power_settings_new", danger: true },
];

/** The rows of the state matrix, each the option it shows. */
const ROWS: readonly { readonly name: string; readonly spec: ListRowSpec }[] = [
  { name: "One line, a symbol and a tick", spec: { title: "Polish", icon: "keyboard", checked: false } },
  { name: "Two lines, a chevron", spec: PAGES[1]! },
  { name: "A tile, tall, with keys", spec: RESULTS[0]! },
  { name: "Chips and a detail", spec: { title: "Samsung 990 Pro", subtitle: "2 TB · NVMe", icon: "hard_drive", chips: [{ kind: "Accent", label: "Default" }], detail: "nvme0n1", checked: false } },
  { name: "A switch", spec: { title: "Bluetooth", subtitle: "2 connected", icon: "bluetooth", toggle: true } },
  { name: "Danger", spec: { title: "Shut down", icon: "power_settings_new", danger: true } },
];

type MatrixState = "rest" | "lit" | "pressed" | "selected" | "disabled";

/** `specs` in a well as wide as a panel's body. */
function well(specs: readonly ListRowSpec[], options: ListOptions = {}): HTMLElement {
  return within(WELL_ROOM, list(specs, options));
}

/** One row alone in its well, in `state`; the well lit as the list with the keyboard when it is. */
function cell(spec: ListRowSpec, state: MatrixState | "signs-off"): HTMLElement {
  if (state === "disabled") return well([{ ...spec, disabled: true }]);
  if (state === "selected") return well([spec.opens || spec.checked === undefined ? spec : { ...spec, checked: true }]);
  if (state === "rest") return well([spec]);
  const pinned: RowPinned = state === "pressed" ? "pressed" : "lit";
  const box = well([spec], { current: 0, pinned, focused: true });
  if (state === "signs-off") box.querySelector<HTMLElement>(".k-list")!.dataset.pinned = "signs-off";
  return box;
}

/**
 * `specs` in a well where taking a row picks it, as the keyboard layouts are picked: the tick moves
 * and the list is drawn again with the keyboard still in it.
 */
function picking(specs: readonly ListRowSpec[], options: ListOptions = {}): HTMLElement {
  const room = within(WELL_ROOM);
  const draw = (rows: readonly ListRowSpec[], current?: number) => {
    const shownList = list(rows, {
      ...options,
      ...(current !== undefined ? { current } : {}),
      onTake: (index) => {
        if (rows[index]?.checked === undefined) return;
        const had = document.activeElement === shownList;
        draw(
          rows.map((row, at) => (row.checked === undefined ? row : { ...row, checked: at === index })),
          index,
        );
        if (had) room.querySelector<HTMLElement>(".k-list")?.focus({ focusVisible: true, preventScroll: true } as FocusOptions);
      },
    });
    room.replaceChildren(shownList);
  };
  draw(specs);
  return room;
}

const DESCRIBED =
  "The well is a content box at the content radius (`radius.content`, 16), `surface_container_low` at `opacity.well` with its sunken edge, a hairline of `outline_variant` at `opacity.outline_soft` inside it, `space.s8` inside, its rows `space.s4` apart, so each row's corners are concentric inside it: 16 − 8, the row radius (`radius.row`, 8). A row follows the box it sits in, as a menu's items do: a menu's popover is 12 round with 4 inside and items 32 tall at 8 and 8 inside, so words start 12 from its edge; a well is 16 round with 8 inside and rows 44 or 56 tall at 8 and 16 inside, so words start 24 from its edge.";

const ROW_DESCRIBED =
  "A row is `size.row` (44) tall with one line and `size.row_two_line` (56) with two, `space.s16` inside its sides: a leading symbol, `body_strong`'s 20 beside one line and `icon.two_line`'s 24 beside two, so it spans both, or the symbol on an icon tile, sm beside one line and md beside two; then its title in `body_strong` over its subtitle in `small`, `space.s4` apart, each one line ending in an ellipsis, the subtitle optionally after a 16 symbol `space.s4` from its words (the lock of a secured network). At its end, `space.s12` apart: chips for facts that matter at a glance, a detail in `mono`, the keys that take it as keycaps `space.s4` apart, a switch, and last a chevron or a tick, never both: a chevron when the row opens a page of its own, a tick (its room kept when unticked, so the rows' ends line up) when it is picked, and neither when taking it simply does something. Every part is centred on the row's height, so a chip moves nothing else.";

const OPTIONS =
  "The Command menu's Results take three options: `tall`, always two lines tall with an md tile, a title alone included, so every row's words start in one column; `clear`, `space.s16` after the tile instead of `space.s12`, so the words stand clear of an app's own icon; and `tight`, the subtitle right under the title. Their titles light the query's matches in `primary`. Their selection is the list's: one shape behind the rows that slides between them over `short` on `standard`, the rows only taking its colours.";

/** The Command menu's selection sliding from one row to the next. */
const SLIDES: Move = {
  name: "The selection slides",
  what: "The Command menu's: ↓ slides the list's one selection from the row it was on to the next over `short` on `standard`, while the rows stay put; the row left fades its content back to rest and the row reached to the selection's colours over the same. Arriving on a list, it is on its row at once. With reduced motion it is on the new row at once.",
  tracks: [
    { part: "the selection", on: "two Results", target: ".k-list-highlight", property: "--along", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the row left", on: "two Results", target: ".k-list-row:nth-child(2)", property: "--lit", from: 1, to: 0, duration: "short", curve: "standard" },
    { part: "the row reached", on: "two Results", target: ".k-list-row:nth-child(3)", property: "--lit", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  reduced: [
    { part: "the selection", on: "two Results", target: ".k-list-highlight", property: "--along", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "the row left", on: "two Results", target: ".k-list-row:nth-child(2)", property: "--lit", from: 1, to: 0, duration: "instant", curve: "linear" },
    { part: "the row reached", on: "two Results", target: ".k-list-row:nth-child(3)", property: "--lit", from: 0, to: 1, duration: "instant", curve: "linear" },
  ],
  back: "plays",
  sample: () => {
    const box = well(RESULTS.slice(0, 2), { slides: true, current: 0 });
    box.querySelector<HTMLElement>(".k-list-highlight")?.setAttribute("data-stepped", "");
    return box;
  },
};


export default page({
  level: "molecules",
  slug: "list",
  title: "List, ListRow and ListRows",
  summary: "Rows in a well, moved through with the keys: the row the keyboard is on is lit, and Enter takes it.",
  live: () => stack("s16", "start", picking(LAYOUTS), well(NETWORKS)),
  description: [
    "Rust `list::list` and `list::list_row`, QML `List`, `ListRow` and `ListRows`. Tab into the keyboard layouts and use ↑ and ↓, or move the pointer over them; Enter or a press picks one. The control center's keyboard layouts and Wi-Fi networks.",
    DESCRIBED,
    ROW_DESCRIBED,
    "The row the keyboard is on is lit: it fades onto `secondary_container` over `short` on `standard`, its title and symbol to `on_secondary_container`, its subtitle and facts to that at `opacity.subdued_text`, and its keycaps' keys and edges to `primary`. Pressed, the container lifts to `secondary_container_pressed` over `micro`. The pointer moving onto a row puts the keyboard on it, so one row is lit at a time, by the pointer and the keys alike; with neither on the list, no row is lit. A row whose action can't be taken back (Shut down) is `error` at rest and lights on `error` itself, its content `on_error`. Rows aren't neon: no tube and no glow.",
    "A picked row (the layout in use, the network you're on) shows its tick, and its title and tick are `primary_lit`, a picked tab's and a select's picked option's colour, with the label glow round the title at night while the signs are on; lit, it takes the lit row's colours. A row that can't be taken now is dimmed, `on_surface` at `opacity.disabled_content`, its tile, chips and keycaps with it, and the keys pass over it. A row still being found (loading) keeps a tiled two-line row's place, its tile and its two lines as skeletons, the title a share of the words' room and the subtitle that share of it again, so rows one under another don't read as one block; it takes no press.",
    "The well's focus tube lights while the list has the keyboard: `primary` inside its edge, at the content radius, with the glow outside at night. The pointer lights a row, never the well. The list is one stop for Tab; the row it is on shows where the keyboard is.",
    OPTIONS,
  ],
  variants: [
    { name: "Ticks, on tiles", for: "One of a few of a kind, picked: the keyboard layouts. Every row keeps the tick's room; the default's star is in the accent.", sample: () => well(LAYOUTS) },
    { name: "Two lines, ticks", for: "Picked from a few: the networks, the one you're on ticked, a fact about each before its subtitle.", sample: () => well(NETWORKS) },
    { name: "Two lines, chevrons", for: "Rows that open a page of their own: a settings app's pages. A row that opens never ticks.", sample: () => well(PAGES) },
    { name: "Actions, one line", for: "A list of what can be done, the last one danger: the power menu.", sample: () => well(POWER) },
    { name: "Switches", for: "Settings the row turns: the row is the control, its switch shows the setting.", sample: () => well([{ title: "Bluetooth", subtitle: "2 connected", icon: "bluetooth", toggle: true }, { title: "Do not disturb", icon: "notifications_off", toggle: false }]) },
    { name: "Chips and a detail", for: "Facts at a glance, then a machine's name for it: the disks.", sample: () => well([{ title: "Samsung 990 Pro", subtitle: "2 TB · NVMe", icon: "hard_drive", chips: [{ kind: "Accent", label: "Default" }], detail: "nvme0n1" }, { title: "WD Blue", subtitle: "500 GB · SATA", icon: "hard_drive", chips: [{ kind: "Neutral", label: "Too small" }], detail: "sda" }]) },
    { name: "The Command menu's", for: "Tall, clear and tight, the matches lit, keys on each; the selection is the list's and slides between them. ↑ and ↓ show it.", sample: () => well(RESULTS, { slides: true }) },
    { name: "Loading", for: "Rows still being found, each a share of the words' room: a Wi-Fi scan with nothing found yet.", sample: () => well([{ title: "", loading: true, share: 0.6 }, { title: "", loading: true, share: 0.45 }, { title: "", loading: true, share: 0.75 }]) },
    { name: "With the keyboard", for: "The list with the keyboard: its well's tube lit, and the row it is on.", sample: () => well(NETWORKS, { current: 1, focused: true }) },
  ],
  sizes: [
    { name: "One line", for: "`size.row`, 44: a title alone, a 20 symbol or an sm tile.", sample: () => well([ROWS[0]!.spec]) },
    { name: "Two lines", for: "`size.row_two_line`, 56: a title over a subtitle, a 24 symbol spanning both or an md tile.", sample: () => well([NETWORKS[1]!]) },
    { name: "Tall", for: "Two lines tall whatever it holds: the Command menu's, so every row's words start in one column.", sample: () => well([RESULTS[2]!]) },
  ],
  states: {
    shown: ["rest", "lit", "pressed", "selected", "disabled"],
    absent: {
      loading: "a row still being found is the Loading variant above: skeletons in a row's place, taking no press, so it has no states of its own.",
      empty: "a well with no rows is an EmptyList (its own page).",
      error: "a row can't be wrong; one that can't be taken now is disabled, and a list that failed to load is an ErrorBlock in its place.",
    },
    rows: nonEmpty(ROWS.map(({ name, spec }) => ({ name, cell: (state: MatrixState) => cell(spec, state) }))),
    described: {
      rest: "No fill; the title and symbol `on_surface`, the subtitle and facts `on_surface_variant`, a danger row's all `error`. The well at rest, unlit. By day the same roles in daylight colours.",
      lit: "The keyboard on it, which the pointer moves: `secondary_container` behind it, its title and symbol `on_secondary_container`, its subtitle and facts that at `opacity.subdued_text`, its keycaps' keys and edges `primary`; a danger row on `error`, its content `on_error`. Here with the keyboard in the list too, the well's tube lit. The row has no line and no glow, by night or by day.",
      pressed: "Lit and pressed: the container lifts to `secondary_container_pressed` (a danger row's to `error_pressed`), 0.035 lighter at night and 0.04 darker by day. Letting go takes it.",
      selected: "Picked, on a row that can be: its tick shows, and its title and tick are `primary_lit`, with the label glow round the title at night while the signs are on. A row with no tick (a chevron, a switch, an action) has no picked look: it shows as at rest.",
      disabled: "Dimmed: `on_surface` at `opacity.disabled_content`, its tile, chips, switch and keycaps with it; never lit, and passed over by the keys.",
    },
  },
  focus: {
    tubes: [{ name: "The well", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "At night at rest, the signs burn: the tube inside the well's edge at the content radius, `primary` at `focus.tube`, the glow outside it, while the list has the keyboard. The row it is on is lit on the secondary container, which is no light: it is the same with the signs on or off. A picked row's title glows faintly.",
      signsOff: "Still lit with the neon unpowered: by day always, and at night while a power change has the lights out. The well's tube is solid glass, `primary_glass`, with no glow; the lit row is as it was, and a picked title has no glow.",
    },
    signsOff: (index: number) => cell(ROWS[index]!.spec, "signs-off"),
  },
  content: {
    text: [
      { name: "A title", words: TITLE, sample: (words: string) => well([{ title: words, icon: "keyboard", checked: false }, ROWS[0]!.spec]) },
      { name: "A subtitle", words: SUBTITLE, sample: (words: string) => well([{ title: "Hoshi 5G", subtitle: words, subtitleIcon: "lock", icon: "wifi" }]) },
      { name: "A title beside chips and a detail", words: TITLE, sample: (words: string) => well([{ title: words, subtitle: "2 TB · NVMe", icon: "hard_drive", chips: [{ kind: "Accent", label: "Default" }], detail: "nvme0n1" }]) },
    ],
    missingPicture: [
      { name: "An app with no icon", for: "Its tile shows the placeholder symbol it names, at its place, so nothing moves when the icon comes.", sample: () => well([{ title: "Obscure tool", subtitle: "No icon of its own", icon: "deployed_code", tile: true, tall: true, clear: true, tight: true }]) },
      { name: "No symbol", for: "A row may have none: its words start `space.s16` in. A list's rows all have one or none.", sample: () => well([{ title: "English (US)", checked: true }, { title: "Polish", checked: false }]) },
      { name: "A symbol that isn't there", for: "QML draws a name with no drawing as its empty box, keeping its room; Rust names symbols by their enum and can't.", sample: () => well([{ title: "Unknown device", icon: "missing" }]) },
    ],
    counts: [
      { name: "One", for: "A single row: still a list in a well.", sample: () => well([NETWORKS[0]!]) },
      { name: "A few", for: "As tall as its rows, `space.s4` apart.", sample: () => well(LAYOUTS) },
      { name: "More than its room", for: "A list taller than its room scrolls inside its well: the ScrollingList page.", sample: () => stack("s8", "start", well(NETWORKS), text("Past its room, a ScrollingList.", "small", "on_surface_variant")) },
    ],
  },
  motion: {
    arrives: doesntApply("a list comes with the surface it is on; a found row takes its loading row's place at once, and the rows a scan adds while it shows arrive as LiveList's do (its own page)."),
    leaves: doesntApply("a list goes with its surface; the rows a scan drops leave as LiveList's do."),
    between: [
      {
        name: "The keyboard moves to a row",
        what: "↓ or the pointer moves the keyboard to the next row: it lights up over `short` on `standard`, onto `secondary_container`, as the one it left goes back to rest over `short`. Never the flicker.",
        tracks: [
          { part: "the row left", on: "two rows", target: ".k-list-row:first-child", property: "--lit", from: 1, to: 0, duration: "short", curve: "standard" },
          { part: "the row reached", on: "two rows", target: ".k-list-row:last-child", property: "--lit", from: 0, to: 1, duration: "short", curve: "standard" },
        ],
        reduced: "same",
        back: "plays",
        sample: () => well([NETWORKS[1]!, NETWORKS[2]!]),
      },
      {
        name: "Pressed",
        what: "A lit row pressed: its container lifts over `micro`, and back as it comes up; letting go takes it.",
        tracks: [{ part: "pressed", on: "the lit row", target: ".k-list-row", property: "--press", from: 0, to: 1, duration: "micro", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => well([NETWORKS[1]!], { current: 0, pinned: "lit" }),
      },
      {
        name: "The list takes the keyboard",
        what: "Tab onto the list: its well's tube and glow fade in over `short` on `standard`, as any control lights up, and its row lights with it; Tab away plays it back.",
        tracks: [
          { part: "the well's tube and glow", on: "the networks", target: ".k-list > .k-ring", property: "--lit", from: 0, to: 1, duration: "short", curve: "standard" },
          { part: "the row it is on", on: "the networks", target: ".k-list-row:first-child", property: "--lit", from: 0, to: 1, duration: "short", curve: "standard" },
        ],
        reduced: "same",
        back: "plays",
        sample: () => well(NETWORKS),
      },
      SLIDES,
      ...powerMoves(() => well(LAYOUTS, { current: 1, focused: true })),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["↓", "j", "Ctrl+J", "Ctrl+N"], does: "The next row that can be taken, round from the last to the first." },
      { keys: ["↑", "k", "Ctrl+K", "Ctrl+P"], does: "The row before that can be taken, round from the first to the last." },
      { keys: ["Home", "End"], does: "The first or the last row that can be taken; a text field that forwards its keys to the list keeps them for its caret." },
      { keys: ["Page Down", "Page Up"], does: "As many rows on as show whole, held at the ends: a page never wraps round." },
      { keys: ["Enter"], does: "Takes the row, once per press, never on key repeat: picks it, opens it or turns its switch." },
      { keys: ["Alt+↑", "Alt+↓"], does: "The list's owner's: a list that reorders moves its row with them." },
    ],
    order: "A list is one stop for Tab: the keys move inside it. Disabled and loading rows are passed over; a list with a field over it (the Command menu) leaves the plain letters, Home and End to the field, which forwards the rest.",
    lands: "Tabbing in, on its picked row, else on the row it was last on, else its first that can be taken.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A well of two-line rows", for: "The well's padding, the rows' gap, each row's height, padding, symbol, words and tick.", sample: () => well(NETWORKS) },
      { name: "One-line rows", for: "A one-line row's height, its 20 symbol and its keycaps.", sample: () => well(POWER) },
      { name: "The Command menu's", for: "A tall row's md tile, `space.s16` after it, its tight words and its keycaps.", sample: () => well(RESULTS) },
      { name: "Chips and a detail", for: "A chip, a detail and the tick's room at the row's end.", sample: () => well([ROWS[3]!.spec]) },
    ]),
    measures: [
      { what: "A one-line row's height.", token: "size-row" },
      { what: "A two-line row's height, and a tall one's.", token: "size-row_two_line" },
      { what: "The well's padding.", token: "space-s8" },
      { what: "Inside a row's sides; after a tile on a `clear` row.", token: "space-s16" },
      { what: "Between a row's parts.", token: "space-s12" },
      { what: "Between rows; between a title and its subtitle, a subtitle's symbol and its words, and keycaps.", token: "space-s4" },
      { what: "A symbol beside one line, a chevron or a tick.", token: "type-body_strong-icon" },
      { what: "A symbol beside two lines.", token: "icon-two_line" },
      { what: "A subtitle's symbol.", token: "type-small-icon" },
      { what: "A tile beside one line; beside two, `size.control_md`.", token: "size-control_sm" },
      { what: "The title's line.", token: "type-body_strong-line" },
      { what: "The subtitle's line.", token: "type-small-line" },
      { what: "A chip's height.", token: "size-chip" },
      { what: "A keycap.", token: "size-keycap" },
      { what: "A switch.", token: "size-switch_width" },
      { what: "The well's corners.", token: "radius-content" },
      { what: "A row's corners: the well's less its padding.", token: "radius-row" },
      { what: "The well's hairline: one device pixel.", token: "stroke-hairline_device_px" },
    ],
    reflow: {
      described: "A list fills the room it is given at every window size, its rows across all of it; a title, a subtitle and a detail end in an ellipsis where they don't fit, and the parts at a row's end keep their place. Here as wide as the window.",
      sample: () => within("100%", stack("s8", "start", list(NETWORKS), text("A list as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
