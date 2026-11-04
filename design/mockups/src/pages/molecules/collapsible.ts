// The Collapsible page: `docs/design/components/collapsible.md`. A titled part that opens to show
// more: the system monitor's process details. The header lights as any control does; the room
// under it eases open and shut as the chevron turns, and the rail down what it holds closes it from
// wherever the pointer is, scrolling the header back into view.

import { button } from "../../lib/button.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { type CollapsibleOptions, collapsible } from "../../lib/collapsible.ts";
import { shown, type Words } from "../../lib/content.ts";
import { row, stack, within } from "../../lib/layout.ts";
import { text, wrapped } from "../../lib/text.ts";
import { el } from "../../site/dom.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

const TITLE: Words = { short: "Hyprland · pid 1203", long: "firefox -contentproc · pid 4127 · Isolated Web Content" };
const SUMMARY: Words = { short: "3 · 310 MB", long: "92 threads · 2.1 GB resident · 4 % CPU" };

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

/** A process's threads, one a line: a section long enough to scroll past its header. */
function threads(): HTMLElement {
  const names = ["firefox", "Gecko_IOThread", "Timer", "Socket Thread", "JS Watchdog", "Backgro~Pool #1", "Backgro~Pool #2", "DOM Worker", "ImageIO", "Renderer", "Compositor", "Cache2 I/O", "dconf worker", "gmain"];
  return stack("s4", "start", ...names.map((name, index) => wrapped(`${4127 + index} · ${name} · sleeping`, "small", "on_surface_variant")));
}

/**
 * A panel's body that scrolls, holding the processes: the first open and longer than the view, so
 * closing it from the rail far down brings its header back.
 */
function scrolled(): HTMLElement {
  return el(
    "div",
    "page-scroll-view",
    stack(
      "s8",
      "start",
      collapsible({ title: "firefox · pid 4127", summary: "14 · 2.1 GB", content: threads(), open: true }),
      collapsible({ title: "kdeconnectd · pid 2210", summary: "3 devices", content: actions() }),
      collapsible({ title: "Hyprland · pid 1203", summary: "3 · 310 MB", content: details() }),
      collapsible({ title: "systemd · pid 1", content: details(), disabled: true }),
    ),
  );
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
  { name: "Closed", open: false, rail: false },
  { name: "Open", open: true, rail: false },
  { name: "The rail", open: true, rail: true },
] as const;

type MatrixState = "rest" | "lit" | "pressed" | "disabled";

/** A cell of the matrix: the header in `state`, or on the rail's row the rail in it. */
function cell(open: boolean, rail: boolean, state: MatrixState): HTMLElement {
  if (!rail || state === "rest" || state === "disabled") return part({ open, summary: SUMMARY.short, ...stateOptions(state as Shown) });
  return part({ open, summary: SUMMARY.short, railPinned: state });
}

/** An open part for the rail's moves, its hint at the rail's middle where the pointer is. */
function railed(): HTMLElement {
  return within(ROOM, collapsible({ title: "firefox · pid 4127", summary: "12 · 4.1 GB", content: actions(), open: true }));
}

/** An open part for closing from the rail: its room cuts what it holds, as it does while moving. */
function railedMoving(): HTMLElement {
  const room = railed();
  room.querySelector(".k-collapsible")?.removeAttribute("data-settled");
  return room;
}

/** Two headers, closed and open, in `state`: what the lighting moves play on. */
function pair(state: Shown = "rest"): () => HTMLElement {
  return () => stack("s8", "start", part(stateOptions(state)), part({ open: true, ...stateOptions(state) }));
}

const ROOM_PART = "the room, the chevron's turn and the rail";
const RAIL = ".k-collapsible-rail";
const RAIL_HINT = ".k-collapsible-rail > .k-hint";

const OPENS: Move = {
  name: "Opening",
  what: "Enter, Space or a press on the header: the room under it eases from nothing to what it holds over `medium` on `standard_decel`, as a list reordering settles, cutting what it holds and the rail as it goes, and the chevron turns a quarter, from right to down, over the same. Nothing moves above it; what is below moves down. With reduced motion the room is open at once, the chevron jumps, and what it holds fades in over `short` on `standard`.",
  tracks: [{ part: ROOM_PART, on: "a process", target: ".k-collapsible", property: "--collapsible-open", from: 0, to: 1, duration: "medium", curve: "standard_decel" }],
  reduced: [
    { part: ROOM_PART, on: "a process", target: ".k-collapsible", property: "--collapsible-open", from: 0, to: 1, duration: "instant", curve: "linear" },
    { part: "what it holds", on: "a process", target: ".k-collapsible-content", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: moving(true),
};

const CLOSES: Move = {
  name: "Closing",
  what: "Pressed again: the room eases shut over `short` on `standard_accel`, what it holds and the rail cut away from the bottom, the chevron turning back to point right. With reduced motion it is shut at once.",
  tracks: [{ part: ROOM_PART, on: "a process", target: ".k-collapsible", property: "--collapsible-open", from: 1, to: 0, duration: "short", curve: "standard_accel" }],
  reduced: [{ part: ROOM_PART, on: "a process", target: ".k-collapsible", property: "--collapsible-open", from: 1, to: 0, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: moving(true),
};

const RAIL_LIGHTS: Move = {
  name: "The rail lights",
  what: "The pointer onto the rail's strip: the line fades over `short` on `standard` from the unlit tone to the core with the content glow round it at night, the tone by day, and its hint, \"Close\" and the title, fades in beside it at the pointer's height, following the pointer down the rail. The pointer leaving plays it back.",
  tracks: [
    { part: "the rail's line", on: "an open process", target: RAIL, property: "--lit", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "the hint", on: "an open process", target: RAIL_HINT, property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  reduced: "same",
  back: "plays",
  sample: railed,
};

const RAIL_CLOSES: Move = {
  name: "Closing from the rail",
  what: "A press on the lit rail: the line goes toward `primary_flash` and its glow swells over `micro` at night, the tone tints the strip by day; letting go closes the part, the room easing shut over `short` on `standard_accel` with the rail cut away from the bottom and the chevron turning back to point right. Where the header has scrolled out of its scroll view, the view scrolls it back in at its nearest edge, smoothly, at once with reduced motion. With reduced motion the room is shut at once.",
  tracks: [
    { part: "the rail, pressed", on: "an open process", target: RAIL, property: "--press", from: 0, to: 1, duration: "micro", curve: "standard" },
    { part: ROOM_PART, on: "an open process", target: ".k-collapsible", property: "--collapsible-open", from: 1, to: 0, duration: "short", cue: "micro", curve: "standard_accel" },
  ],
  reduced: [
    { part: "the rail, pressed", on: "an open process", target: RAIL, property: "--press", from: 0, to: 1, duration: "micro", curve: "standard" },
    { part: ROOM_PART, on: "an open process", target: ".k-collapsible", property: "--collapsible-open", from: 1, to: 0, duration: "instant", cue: "micro", curve: "linear" },
  ],
  back: "cuts",
  sample: () => {
    const room = railedMoving();
    room.querySelector(RAIL)?.setAttribute("data-pinned", "lit");
    return room;
  },
};

const DESCRIBED =
  "The header is a control as tall as a row (`size.row`, 44) at the control radius, filling its room, `space.s12` inside its sides: a chevron (`expand_more`, `body_strong`'s 20) at its start in `on_surface_variant`, then `space.s12` on its title in `body_strong`, `on_surface`, on one line ending in an ellipsis where it doesn't fit. An optional summary sits at its end, `space.s12` from the title, in `label`, `on_surface_variant`, one line up to half the header's width, ending in an ellipsis: \"12 · 4.1 GB\", \"3 devices\". It stays while the part is open; with none, the end is empty. It has no fill. Open, what it holds sits `space.s8` under the header and `space.s12` in at the end, and at the start in line with the title, past the chevron, so it brings no inset of its own.";

const RAIL_DESCRIBED =
  "Open, an unlit rail of neon runs down what it holds, centred under the chevron, from its top to its foot inside its padding, square at both ends: the tone at the spinner's ring's `opacity.spinner_track`, `tabs.rail` thick at night (2) and `tabs.rail_day` by day (3), whole device pixels, as the tabs' rail. It grows and shrinks with the room, which cuts it. Under the pointer it lights over `short`: at night the core with the content glow round it, by day the tone, with the signs off solid glass; a hint, \"Close\" and the title, sits `space.s8` past its strip at the pointer's height and follows it down the rail. Its strip is `size.collapsible_rail` (12) wide, centred on the line; a press anywhere on it closes the part, and where the header has scrolled out of its scroll view, the view scrolls it back in at its nearest edge, smoothly unless reduced motion is on. Pressed, at night the line goes toward `primary_flash` and its glow swells; by day the tone tints the strip at `opacity.press_tint`. It is no stop for Tab: the keyboard has the header.";

export default page({
  level: "molecules",
  slug: "collapsible",
  title: "Collapsible",
  summary: "A titled part that opens to show more: a header with a chevron that turns, a room that eases open under it, and a rail down what it holds that closes it.",
  live: scrolled,
  description: [
    "Rust `collapsible::collapsible`, QML `Collapsible`. Press a header, or Tab to it and press Enter or Space. The first is open and longer than its scroll view: scroll down and press its rail, and it closes and scrolls its header back. The second holds two buttons, which Tab reaches only while it is open. The system monitor's process details.",
    DESCRIBED,
    RAIL_DESCRIBED,
    "The chevron points right while it is closed and turns a quarter to point down as it opens, over the same time and curve as the room. The room eases from nothing to what it holds over `medium` on `standard_decel` and back over `short` on `standard_accel`, cutting what it holds while it moves; fully open it cuts nothing, so a lit control inside keeps its whole glow. The header never moves: what is below it moves down and back. The owner keeps whether it is open.",
    "Its states follow the neon, as a Text button's: lit under the pointer or with the keyboard on it, the tube burns inside the header's edge, `primary` at `focus.tube`, with the glow outside at night; pressed, the tube runs white-hot and the glow brightens, and by day the tone tints the header (`opacity.press_tint`). The title and the chevron keep their colours. Disabled (no toggle), the title and the chevron are at `opacity.disabled_content`, and it takes no press and no stop for Tab; what it holds is the owner's to dim with it.",
  ],
  variants: [
    { name: "Closed", for: "Its title alone: what is behind it isn't needed until asked for.", sample: () => part() },
    { name: "Open", for: "What it holds under the header, in line with the title, the rail down its start.", sample: () => part({ open: true }) },
    { name: "With a summary", for: "What it holds, said in a few words at the header's end, closed and open.", sample: () => stack("s8", "start", part({ summary: SUMMARY.short }), part({ summary: SUMMARY.short, open: true })) },
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
    rows: nonEmpty(ROWS.map(({ name, open, rail }) => ({ name, cell: (state: MatrixState) => cell(open, rail, state) }))),
    described: {
      rest: "No fill; the title `on_surface`, the chevron and the summary `on_surface_variant`, the chevron right while closed and down while open. Open, the rail unlit, the tone at `opacity.spinner_track`. By day the same roles in daylight colours.",
      lit: "Under the pointer or with the keyboard on the header, here with the signs on: the tube burns inside the header's edge, `primary` at `focus.tube`, with the glow outside. By day the tube with no glow. On the rail's row the rail under the pointer instead: its line the core with the content glow round it, the tone by day, and its hint beside it.",
      pressed: "Lit and pressed: the tube runs white-hot, to the core, and the glow brightens to `focus.pressed_glow` times the lit one; by day the tone tints the header at `opacity.press_tint`. Letting go toggles it. On the rail's row the rail pressed: the line toward `primary_flash`, its glow swelled, and by day the strip tinted; letting go closes it.",
      disabled: "The title, the chevron and the summary at `opacity.disabled_content`, and open the rail too; never lit, no press, no stop for Tab, and the rail takes no pointer. What it holds is dimmed by its owner.",
    },
  },
  focus: {
    tubes: [{ name: "The header", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "At night at rest, the signs burn: the tube inside the header's edge at the control radius, the glow outside it, or the rail's line burning as the core with the content glow round it. Pressed, the tube runs white-hot and the rail goes toward the flash.",
      signsOff: "Still lit with the neon unpowered: by day always, and at night while a power change has the lights out. The tube and the rail's line are solid glass, `primary_glass`, with no glow.",
    },
    signsOff: (index: number) => {
      const { open, rail } = ROWS[index]!;
      return part({ open, summary: SUMMARY.short, ...(rail ? { railPinned: "signs-off" } : { pinned: "signs-off" }) });
    },
  },
  content: {
    text: [
      { name: "The title", words: TITLE, sample: (words: string) => within(ROOM, collapsible({ title: words, content: details() })) },
      { name: "The summary", words: SUMMARY, sample: (words: string) => within(ROOM, collapsible({ title: shown(TITLE), summary: words, content: details() })) },
    ],
    missingPicture: doesntApply("a collapsible draws no picture: its chevron is always there."),
    counts: [
      { name: "One line", for: "A single line under the header.", sample: () => within(ROOM, collapsible({ title: "Hyprland · pid 1203", content: wrapped("kuba · running · 18 threads · 310 MB", "small", "on_surface_variant"), open: true })) },
      { name: "Several lines", for: "A process's command and state.", sample: () => part({ open: true }) },
      { name: "Controls", for: "Buttons inside: Tab reaches them only while it is open, and their glow reaches past the room once it has settled.", sample: () => within(ROOM, collapsible({ title: "firefox · pid 4127", content: actions(), open: true })) },
      { name: "Longer than its view", for: "A process's threads in a scroll view: the rail runs the whole length, and closing from it far down scrolls the header back.", sample: scrolled },
    ],
  },
  motion: {
    arrives: doesntApply("a collapsible comes with the surface it is on."),
    leaves: doesntApply("it goes with the surface it is on."),
    between: [
      OPENS,
      CLOSES,
      RAIL_LIGHTS,
      RAIL_CLOSES,
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
    order: "The header is one stop in the surface's reading order; open, the controls it holds follow it. The rail is the pointer's alone and no stop. A disabled one is skipped.",
    lands: "On the header, open or shut: toggling never moves the keyboard into what it holds.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Open", for: "The header's height and padding, the chevron, the summary, what it holds under it and the rail beside it.", sample: () => part({ open: true, summary: SUMMARY.short }) },
      { name: "Closed", for: "The header alone.", sample: () => part() },
    ]),
    measures: [
      { what: "The header's height, a row's.", token: "size-row" },
      { what: "Inside the header's sides, between the chevron, the title and the summary, and what it holds in from its end; twice it and the chevron, what it holds in from its start.", token: "space-s12" },
      { what: "What it holds under the header, and the rail's top under it; the rail's hint past its strip.", token: "space-s8" },
      { what: "The title, `body_strong`'s line.", token: "type-body_strong-line" },
      { what: "The summary, `label`'s line.", token: "type-label-line" },
      { what: "The chevron, `body_strong`'s symbol.", token: "type-body_strong-icon" },
      { what: "The rail's strip, which takes the pointer.", token: "size-collapsible_rail" },
      { what: "The rail's line at night.", token: "tabs-rail" },
      { what: "The rail's line by day.", token: "tabs-rail_day" },
      { what: "The header's corners.", token: "radius-control" },
    ],
    reflow: {
      described: "A collapsible fills the room it is given at every window size; its title ends in an ellipsis where it doesn't fit, and what it holds wraps in the room.",
      sample: () => within("100%", stack("s8", "start", collapsible({ title: shown(TITLE), content: details(), open: true }), text("A collapsible as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
