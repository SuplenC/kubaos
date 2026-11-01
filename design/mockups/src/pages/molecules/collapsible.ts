// The Collapsible page: `docs/design/components/collapsible.md`. A titled part that opens to show
// more: the system monitor's process details. The header lights as any control does; the room
// under it eases open and shut as the chevron turns.

import { button } from "../../lib/button.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { type CollapsibleOptions, collapsible } from "../../lib/collapsible.ts";
import { shown, type Words } from "../../lib/content.ts";
import { row, stack, within } from "../../lib/layout.ts";
import { text, wrapped } from "../../lib/text.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

const TITLE: Words = { short: "Hyprland · pid 1203", long: "firefox -contentproc · pid 4127 · Isolated Web Content" };

/** A panel's body: the system monitor's width less its padding, where a collapsible lives. */
const ROOM = "calc(var(--size-panel) - 2 * var(--container-0-padding))";

/** A process's details: its command and what it is doing. */
function details(): HTMLElement {
  return stack("s4", "start", wrapped("/usr/lib/firefox/firefox -contentproc", "mono", "on_surface_variant"), wrapped("kuba · sleeping · 92 threads · 2.1 GB", "small", "on_surface_variant"));
}

/** A process's details and what can be done to it: controls inside, which take Tab while it is open. */
function actions(): HTMLElement {
  return stack("s12", "start", details(), row("s8", "centre", button({ label: "Stop", kind: "Tonal", size: "Sm" }), button({ label: "End", kind: "Text", size: "Sm" })));
}

function part(extra: Partial<CollapsibleOptions> = {}): HTMLElement {
  return within(ROOM, collapsible({ title: shown(TITLE), content: details(), ...extra }));
}

/** A collapsible drawn for a player: its room always cuts what it holds, as it does while moving. */
function moving(open: boolean): () => HTMLElement {
  return () => {
    const room = within(ROOM, collapsible({ title: "firefox · pid 4127", content: details(), open }));
    room.querySelector(".k-collapsible")?.removeAttribute("data-settled");
    return room;
  };
}

const ROWS = [
  { name: "Closed", open: false },
  { name: "Open", open: true },
] as const;

type MatrixState = "rest" | "lit" | "pressed" | "disabled";

function cell(open: boolean, state: MatrixState): HTMLElement {
  return part({ open, ...stateOptions(state as Shown) });
}

/** Two headers, closed and open, in `state`: what the lighting moves play on. */
function pair(state: Shown = "rest"): () => HTMLElement {
  return () => stack("s8", "start", part(stateOptions(state)), part({ open: true, ...stateOptions(state) }));
}

const ROOM_PART = "the room, and the chevron's turn";

const OPENS: Move = {
  name: "Opening",
  what: "Enter, Space or a press on the header: the room under it eases from nothing to what it holds over `medium` on `standard_decel`, as a list reordering settles, cutting what it holds as it goes, and the chevron turns from down to up over the same. Nothing moves above it; what is below moves down. With reduced motion the room is open at once and what it holds fades in over `short` on `standard`.",
  tracks: [{ part: ROOM_PART, on: "a process", target: ".k-collapsible", property: "--open", from: 0, to: 1, duration: "medium", curve: "standard_decel" }],
  reduced: [
    { part: ROOM_PART, on: "a process", target: ".k-collapsible", property: "--open", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "what it holds", on: "a process", target: ".k-collapsible-content", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: moving(true),
};

const CLOSES: Move = {
  name: "Closing",
  what: "Pressed again: the room eases shut over `short` on `standard_accel`, what it holds cut away from the bottom, the chevron turning back down. With reduced motion it is shut at once.",
  tracks: [{ part: ROOM_PART, on: "a process", target: ".k-collapsible", property: "--open", from: 1, to: 0, duration: "short", curve: "standard_accel" }],
  reduced: [{ part: ROOM_PART, on: "a process", target: ".k-collapsible", property: "--open", from: 1, to: 0, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: moving(true),
};

const DESCRIBED =
  "The header is a control as tall as a row (`size.row`, 44) at the control radius, filling its room, `space.s12` inside its sides: its title in `body_strong`, `on_surface`, on one line ending in an ellipsis where it doesn't fit, and a chevron (`expand_more`, `body_strong`'s 20) at its end in `on_surface_variant`, `space.s12` from the title. It has no fill. Open, what it holds sits `space.s8` under the header and `space.s12` in at either side, in line with the title, so it brings no inset of its own.";

export default page({
  level: "molecules",
  slug: "collapsible",
  title: "Collapsible",
  summary: "A titled part that opens to show more: a header with a chevron that turns, and a room that eases open under it.",
  live: () => within(ROOM, stack("s8", "start", collapsible({ title: "firefox · pid 4127", content: actions(), open: true }), collapsible({ title: "Hyprland · pid 1203", content: details() }), collapsible({ title: "systemd · pid 1", content: details(), disabled: true }))),
  description: [
    "Rust `collapsible::collapsible`, QML `Collapsible`. Press a header, or Tab to it and press Enter or Space. The first holds two buttons, which Tab reaches only while it is open. The system monitor's process details.",
    DESCRIBED,
    "The chevron points down while it is closed and turns to point up as it opens, over the same time and curve as the room. The room eases from nothing to what it holds over `medium` on `standard_decel` and back over `short` on `standard_accel`, cutting what it holds while it moves; fully open it cuts nothing, so a lit control inside keeps its whole glow. The header never moves: what is below it moves down and back. The owner keeps whether it is open.",
    "Its states follow the neon, as a Text button's: lit under the pointer or with the keyboard on it, the tube burns inside the header's edge, `primary` at `focus.tube`, with the glow outside at night; pressed, the tube runs white-hot and the glow brightens, and by day the tone tints the header (`opacity.press_tint`). The title and the chevron keep their colours. Disabled (no toggle), the title and the chevron are at `opacity.disabled_content`, and it takes no press and no stop for Tab; what it holds is the owner's to dim with it.",
  ],
  variants: [
    { name: "Closed", for: "Its title alone: what is behind it isn't needed until asked for.", sample: () => part() },
    { name: "Open", for: "What it holds under the header, in line with the title.", sample: () => part({ open: true }) },
  ],
  sizes: doesntApply("the header is always a row tall (`size.row`), and the part fills the room it is given."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "open is its own row here: a collapsible isn't picked.",
      loading: "what it holds is the owner's: a part still loading shows its own skeleton inside.",
      empty: "a collapsible with nothing to show isn't drawn: its owner leaves it out.",
      error: "it has no value to be wrong; what it holds shows its own errors.",
    },
    rows: nonEmpty(ROWS.map(({ name, open }) => ({ name, cell: (state: MatrixState) => cell(open, state) }))),
    described: {
      rest: "No fill; the title `on_surface`, the chevron `on_surface_variant`, down while closed and up while open. By day the same roles in daylight colours.",
      lit: "Under the pointer or with the keyboard on the header, here with the signs on: the tube burns inside the header's edge, `primary` at `focus.tube`, with the glow outside. By day the tube with no glow.",
      pressed: "Lit and pressed: the tube runs white-hot, to the core, and the glow brightens to `focus.pressed_glow` times the lit one; by day the tone tints the header at `opacity.press_tint`. Letting go toggles it.",
      disabled: "The title and the chevron at `opacity.disabled_content`; never lit, no press, no stop for Tab. What it holds is dimmed by its owner.",
    },
  },
  focus: {
    tubes: [{ name: "The header", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "At night at rest, the signs burn: the tube inside the header's edge at the control radius, the glow outside it. Pressed, the tube runs white-hot.",
      signsOff: "Still lit with the neon unpowered: by day always, and at night while a power change has the lights out. The tube is solid glass, `primary_glass`, with no glow.",
    },
    signsOff: (index: number) => part({ open: ROWS[index]!.open, pinned: "signs-off" }),
  },
  content: {
    text: [{ name: "The title", words: TITLE, sample: (words: string) => within(ROOM, collapsible({ title: words, content: details() })) }],
    missingPicture: doesntApply("a collapsible draws no picture: its chevron is always there."),
    counts: [
      { name: "One line", for: "A single line under the header.", sample: () => within(ROOM, collapsible({ title: "Hyprland · pid 1203", content: wrapped("kuba · running · 18 threads · 310 MB", "small", "on_surface_variant"), open: true })) },
      { name: "Several lines", for: "A process's command and state.", sample: () => part({ open: true }) },
      { name: "Controls", for: "Buttons inside: Tab reaches them only while it is open, and their glow reaches past the room once it has settled.", sample: () => within(ROOM, collapsible({ title: "firefox · pid 4127", content: actions(), open: true })) },
    ],
  },
  motion: {
    arrives: doesntApply("a collapsible comes with the surface it is on."),
    leaves: doesntApply("it goes with the surface it is on."),
    between: [
      OPENS,
      CLOSES,
      { name: "Focus arrives", what: "Under the pointer or with the keyboard on the header, it lights up over `short` on `standard`: the tube and the glow fade in.", tracks: lights(".k-collapsible-header", "two headers", 0, 1), reduced: "same", back: "cuts", sample: pair() },
      { name: "Focus leaves", what: "It goes back to rest over `short` on `standard`.", tracks: lights(".k-collapsible-header", "two headers", 1, 0), reduced: "same", back: "cuts", sample: pair("lit") },
      { name: "Pressed", what: "A lit header pressed: the tube runs white-hot and the glow brightens over `micro`, and back as it comes up; letting go toggles it.", tracks: press(".k-collapsible-header", "two headers"), reduced: "same", back: "plays", sample: pair("lit") },
      ...powerMoves(pair("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Enter", "Space"], does: "On the header, opens it or closes it, once per press, never on key repeat." },
      { keys: ["Tab", "Shift+Tab"], does: "To the header, then into what it holds while it is open; closed, what it holds is passed over." },
    ],
    order: "The header is one stop in the surface's reading order; open, the controls it holds follow it. A disabled one is skipped.",
    lands: "On the header, open or shut: toggling never moves the keyboard into what it holds.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Open", for: "The header's height and padding, the chevron, and what it holds under it.", sample: () => part({ open: true }) },
      { name: "Closed", for: "The header alone.", sample: () => part() },
    ]),
    measures: [
      { what: "The header's height, a row's.", token: "size-row" },
      { what: "Inside the header's sides, between the title and the chevron, and what it holds in from the sides.", token: "space-s12" },
      { what: "What it holds under the header.", token: "space-s8" },
      { what: "The title, `body_strong`'s line.", token: "type-body_strong-line" },
      { what: "The chevron, `body_strong`'s symbol.", token: "type-body_strong-icon" },
      { what: "The header's corners.", token: "radius-control" },
    ],
    reflow: {
      described: "A collapsible fills the room it is given at every window size; its title ends in an ellipsis where it doesn't fit, and what it holds wraps in the room.",
      sample: () => within("100%", stack("s8", "start", collapsible({ title: shown(TITLE), content: details(), open: true }), text("A collapsible as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
