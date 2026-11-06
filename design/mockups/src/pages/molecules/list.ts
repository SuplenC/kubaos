// The List page, with ListRow and ListRows: `docs/design/components/list.md`. Rows in a well,
// moved through with the keys; one build with every raised list, at a window list's scale. The row
// the keyboard is on is lit on the secondary container, and the pointer moving onto a row puts the
// keyboard on it.

import type { Words } from "../../lib/content.ts";
import { filling, flow, within } from "../../lib/layout.ts";
import { type ListOptions, type ListRowSpec, list, type RowPinned } from "../../lib/list.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

/** The stage's pane, less the room a lit control's glow throws past its edge, which it would otherwise scale down for. */
const ACROSS = "calc(100% - var(--space-s24))";

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
  { name: "One line, symbol, tick", spec: { title: "Polish", icon: "keyboard", checked: false } },
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

/** The Command menu's selection sliding from one row to the next. */
const SLIDES: Move = {
  name: "The selection slides",
  what: "↓ slides the one selection to the next row over `short` on `standard`; the rows stay put.",
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
  lede: "Rows in a well, moved through with the keys: the row the keyboard is on is lit, and Enter takes it.",
  code: { rust: "list::list, list::list_row", qml: "List, ListRow, ListRows" },
  stage: {
    live: () => within(ACROSS, filling(flow("s32", picking(LAYOUTS), well(NETWORKS)))),
    hint: "Tab in, ↑ ↓ and Enter, or click a row",
    daylight: true,
  },
  variants: [
    { name: "Ticks, on tiles", for: "one of a few, picked", sample: () => well(LAYOUTS) },
    { name: "Two lines, ticks", for: "the networks, with a fact before the subtitle", sample: () => well(NETWORKS) },
    { name: "Two lines, chevrons", for: "rows that open a page; these never tick", sample: () => well(PAGES) },
    { name: "Actions, one line", for: "what can be done, danger last", sample: () => well(POWER) },
    { name: "Switches", for: "the row is the control", sample: () => well([{ title: "Bluetooth", subtitle: "2 connected", icon: "bluetooth", toggle: true }, { title: "Do not disturb", icon: "notifications_off", toggle: false }]) },
    { name: "Chips and a detail", for: "facts at a glance, then a machine's name", sample: () => well([{ title: "Samsung 990 Pro", subtitle: "2 TB · NVMe", icon: "hard_drive", chips: [{ kind: "Accent", label: "Default" }], detail: "nvme0n1" }, { title: "WD Blue", subtitle: "500 GB · SATA", icon: "hard_drive", chips: [{ kind: "Neutral", label: "Too small" }], detail: "sda" }]) },
    { name: "The Command menu's", for: "tall, clear, tight; one selection slides between rows", sample: () => well(RESULTS, { slides: true }) },
    { name: "Loading", for: "a scan with nothing found yet", sample: () => well([{ title: "", loading: true, share: 0.6 }, { title: "", loading: true, share: 0.45 }, { title: "", loading: true, share: 0.75 }]) },
    { name: "With the keyboard", for: "the well's tube lit, and its row", sample: () => well(NETWORKS, { current: 1, focused: true }) },
  ],
  sizes: [
    { name: "One line", for: "`size.row`: a 20 symbol or an sm tile", sample: () => well([ROWS[0]!.spec]) },
    { name: "Two lines", for: "`size.row_two_line`: a 24 symbol or an md tile", sample: () => well([NETWORKS[1]!]) },
    { name: "Tall", for: "two lines tall whatever it holds", sample: () => well([RESULTS[2]!]) },
  ],
  states: {
    shown: ["rest", "lit", "pressed", "selected", "disabled"],
    absent: {
      loading: "the Loading variant: skeletons, no press",
      empty: "a well with no rows is an `EmptyList`",
      error: "a failed list is an `ErrorBlock`",
    },
    rows: nonEmpty(ROWS.map(({ name, spec }) => ({ name, cell: (state: MatrixState) => cell(spec, state) }))),
    looks: {
      rest: "no fill; a danger row's content in `error`",
      lit: "`secondary_container`, content `on_secondary_container`; no line or glow",
      pressed: "lifts to `secondary_container_pressed`, a danger row to `error_pressed`",
      selected: "its tick; title and tick `primary_lit`",
      disabled: "`on_surface` at `opacity.disabled_content`, passed over by keys",
    },
  },
  focus: {
    tubes: [{ name: "The well", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "the well's tube at `radius.content`, glow outside; the lit row unchanged",
      signsOff: "the well's tube as `primary_glass`, no glow",
    },
    signsOff: (index: number) => cell(ROWS[index]!.spec, "signs-off"),
  },
  content: {
    text: [
      { name: "A title", words: TITLE, sample: (words: string) => well([{ title: words, icon: "keyboard", checked: false }, ROWS[0]!.spec]) },
      { name: "A subtitle", words: SUBTITLE, sample: (words: string) => well([{ title: "Hoshi 5G", subtitle: words, subtitleIcon: "lock", icon: "wifi" }]) },
      { name: "A title beside chips, a detail", words: TITLE, sample: (words: string) => well([{ title: words, subtitle: "2 TB · NVMe", icon: "hard_drive", chips: [{ kind: "Accent", label: "Default" }], detail: "nvme0n1" }]) },
    ],
    missingPicture: [
      { name: "An app with no icon", for: "its tile shows the placeholder it names", sample: () => well([{ title: "Obscure tool", subtitle: "No icon of its own", icon: "deployed_code", tile: true, tall: true, clear: true, tight: true }]) },
      { name: "No symbol", for: "words start `space.s16` in", sample: () => well([{ title: "English (US)", checked: true }, { title: "Polish", checked: false }]) },
      { name: "A symbol that isn't there", for: "QML's empty box keeps its room", sample: () => well([{ title: "Unknown device", icon: "missing" }]) },
    ],
    counts: [
      { name: "One", for: "still a list in a well", sample: () => well([NETWORKS[0]!]) },
      { name: "A few", for: "as tall as its rows", sample: () => well(LAYOUTS) },
      { name: "More than its room", for: "a `ScrollingList`, on its own page", sample: () => well(NETWORKS) },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its surface; a scan's rows arrive as `LiveList`'s do."),
    leaves: doesntApply("it goes with its surface; a scan's rows leave as `LiveList`'s do."),
    between: [
      {
        name: "The keyboard moves to a row",
        what: "The row reached lights over `short` on `standard` as the one left goes back to rest.",
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
        what: "Its container lifts over `micro`, and back as it comes up.",
        tracks: [{ part: "pressed", on: "the lit row", target: ".k-list-row", property: "--press", from: 0, to: 1, duration: "micro", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => well([NETWORKS[1]!], { current: 0, pinned: "lit" }),
      },
      {
        name: "The list takes the keyboard",
        what: "The well's tube and glow fade in over `short` on `standard`, its row lighting with them.",
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
      { keys: ["↓", "j", "Ctrl+J", "Ctrl+N"], does: "the next row, round from the last" },
      { keys: ["↑", "k", "Ctrl+K", "Ctrl+P"], does: "the row before, round from the first" },
      { keys: ["Home", "End"], does: "the first or last row" },
      { keys: ["Page Down", "Page Up"], does: "a page of rows, held at the ends" },
      { keys: ["Enter"], does: "takes the row: picks, opens or turns it" },
      { keys: ["Alt+↑", "Alt+↓"], does: "the owner's: moves a row in a reorderable list" },
    ],
    order: "One stop for Tab; disabled and loading rows are passed over.",
    lands: "On its picked row, else the row it was last on, else its first.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A well of two-line rows", for: "padding, gaps, a row's height, symbol, words, tick", sample: () => well(NETWORKS) },
      { name: "One-line rows", for: "height, the 20 symbol, keycaps", sample: () => well(POWER) },
      { name: "The Command menu's", for: "a tall row's md tile, tight words, keycaps", sample: () => well(RESULTS) },
      { name: "Chips and a detail", for: "a chip, a detail, the tick's room", sample: () => well([ROWS[3]!.spec]) },
    ]),
    measures: [
      { part: "One-line row", token: "size-row" },
      { part: "Two-line or tall row", token: "size-row_two_line" },
      { part: "Well padding", token: "space-s8" },
      { part: "Row inset, after a `clear` tile", token: "space-s16" },
      { part: "Between a row's parts", token: "space-s12" },
      { part: "Between rows, title → subtitle, keycaps", token: "space-s4" },
      { part: "Symbol by one line, chevron, tick", token: "type-body_strong-icon" },
      { part: "Symbol by two lines", token: "icon-two_line" },
      { part: "Subtitle's symbol", token: "type-small-icon" },
      { part: "Tile by one line", token: "size-control_sm" },
      { part: "Title line", token: "type-body_strong-line" },
      { part: "Subtitle line", token: "type-small-line" },
      { part: "Chip", token: "size-chip" },
      { part: "Keycap", token: "size-keycap" },
      { part: "Switch", token: "size-switch_width" },
      { part: "Well corner", token: "radius-content" },
      { part: "Row corner", token: "radius-row" },
      { part: "Well hairline", token: "stroke-hairline_device_px" },
    ],
    reflow: {
      note: "It fills its room: words end in an ellipsis, and the parts at a row's end keep their place.",
      sample: () => within("100%", list(NETWORKS)),
    },
  },
});
