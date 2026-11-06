// The Collapsible page: `docs/design/components/collapsible.md`. A titled part that opens to show
// more: the system monitor's process details. The header lights as any control does; the room
// under it eases open and shut as the chevron turns, and the rail down what it holds closes it from
// wherever the pointer is, scrolling the header back into view.

import { button } from "../../lib/button.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { type CollapsibleOptions, collapsible } from "../../lib/collapsible.ts";
import { shown, type Words } from "../../lib/content.ts";
import { filling, flow, row, stack, within } from "../../lib/layout.ts";
import { wrapped } from "../../lib/text.ts";
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
  what: "The room eases open over `medium` on `standard_decel` as the chevron turns a quarter; only what is below moves.",
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
  what: "The room eases shut over `short` on `standard_accel`, what it holds cut from the bottom.",
  tracks: [{ part: ROOM_PART, on: "a process", target: ".k-collapsible", property: "--collapsible-open", from: 1, to: 0, duration: "short", curve: "standard_accel" }],
  reduced: [{ part: ROOM_PART, on: "a process", target: ".k-collapsible", property: "--collapsible-open", from: 1, to: 0, duration: "instant", curve: "linear" }],
  back: "cuts",
  sample: moving(true),
};

const RAIL_LIGHTS: Move = {
  name: "The rail lights",
  what: "The line fades to the core over `short` on `standard`, its hint beside the pointer.",
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
  what: "A press flares the line over `micro`, then it closes; a header scrolled away scrolls back into view.",
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

export default page({
  level: "molecules",
  slug: "collapsible",
  title: "Collapsible",
  lede: "A titled part that opens to show more: a turning chevron, a room easing open, and a rail that closes it.",
  code: { rust: "collapsible::collapsible", qml: "Collapsible" },
  stage: {
    live: () => within("100%", filling(flow("s32", scrolled(), part({ open: true, summary: SUMMARY.short })))),
    hint: "press a header; scroll and press the rail; or Tab and Enter",
    daylight: true,
  },
  variants: [
    { name: "Closed", for: "its title alone", sample: () => part() },
    { name: "Open", for: "what it holds in line with the title, the rail beside", sample: () => part({ open: true }) },
    { name: "With a summary", for: "a few words at the header's end, closed and open", sample: () => stack("s8", "start", part({ summary: SUMMARY.short }), part({ summary: SUMMARY.short, open: true })) },
  ],
  sizes: doesntApply("the header is always `size.row` tall, the part fills its room."),
  states: {
    shown: ["rest", "lit", "pressed", "disabled"],
    absent: {
      selected: "open is its own row here",
      loading: "what it holds shows its own skeleton",
      empty: "with nothing to show, its owner leaves it out",
      error: "what it holds shows its own errors",
    },
    rows: nonEmpty(ROWS.map(({ name, open, rail }) => ({ name, cell: (state: MatrixState) => cell(open, rail, state) }))),
    looks: {
      rest: "no fill; open, the rail unlit at `opacity.spinner_track`",
      lit: "the `primary` tube inside the header; the rail burns as the core",
      pressed: "white-hot, glow × `focus.pressed_glow`; the rail toward `primary_flash`",
      disabled: "words and chevron at `opacity.disabled_content`, no press, no Tab stop",
    },
  },
  focus: {
    tubes: [{ name: "The header", tone: "accent", filled: false, content: false }],
    looks: {
      lit: "the tube at the control radius, glow outside; the rail's line burning",
      signsOff: "the tube and rail as `primary_glass`, no glow",
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
    missingPicture: doesntApply("it draws no picture: its chevron is always there."),
    counts: [
      { name: "One line", for: "a single line under the header", sample: () => within(ROOM, collapsible({ title: "Hyprland · pid 1203", content: wrapped("kuba · running · 18 threads · 310 MB", "small", "on_surface_variant"), open: true })) },
      { name: "Several lines", for: "a process's command and state", sample: () => part({ open: true }) },
      { name: "Controls", for: "Tab reaches them only while it is open", sample: () => within(ROOM, collapsible({ title: "firefox · pid 4127", content: actions(), open: true })) },
      { name: "Longer than its view", for: "closing from the rail scrolls the header back", sample: scrolled },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its surface."),
    leaves: doesntApply("it goes with its surface."),
    between: [
      OPENS,
      CLOSES,
      RAIL_LIGHTS,
      RAIL_CLOSES,
      { name: "Focus arrives", what: "The tube and glow fade in over `short` on `standard`.", tracks: lights(".k-collapsible-header", "two headers", 0, 1), reduced: "same", back: "cuts", sample: pair() },
      { name: "Focus leaves", what: "Back to rest over `short` on `standard`.", tracks: lights(".k-collapsible-header", "two headers", 1, 0), reduced: "same", back: "cuts", sample: pair("lit") },
      { name: "Pressed", what: "The tube runs white-hot and the glow brightens over `micro`, and back as it comes up.", tracks: press(".k-collapsible-header", "two headers"), reduced: "same", back: "plays", sample: pair("lit") },
      ...powerMoves(pair("lit")),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Enter", "Space"], does: "on the header, opens or closes it" },
      { keys: ["Tab", "Shift+Tab"], does: "the header, then what it holds while open" },
    ],
    order: "The header, then its controls while open; the rail is the pointer's alone, a disabled one skipped.",
    lands: "On the header: toggling never moves the keyboard inside.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Open", for: "header, chevron, summary, what it holds, the rail", sample: () => part({ open: true, summary: SUMMARY.short }) },
      { name: "Closed", for: "the header alone", sample: () => part() },
    ]),
    measures: [
      { part: "Header height", token: "size-row" },
      { part: "Header inset, its gaps", token: "space-s12" },
      { part: "Header → content, rail's hint", token: "space-s8" },
      { part: "Title line", token: "type-body_strong-line" },
      { part: "Summary line", token: "type-label-line" },
      { part: "Chevron", token: "type-body_strong-icon" },
      { part: "Rail's strip", token: "size-collapsible_rail" },
      { part: "Rail at night", token: "tabs-rail" },
      { part: "Rail by day", token: "tabs-rail_day" },
      { part: "Header corner", token: "radius-control" },
    ],
    reflow: {
      note: "It fills its room: the title ends in an ellipsis, and what it holds wraps.",
      sample: () => within("100%", collapsible({ title: shown(TITLE), content: details(), open: true })),
    },
  },
});
