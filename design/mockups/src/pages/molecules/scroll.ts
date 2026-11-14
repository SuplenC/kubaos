// The Scroll and HeaderFade page: `docs/design/components/scroll.md`. A panel's content taller than
// its room, reaching past its box so a resting control's glow is never cut, and the fade that keeps
// the gap under the panel's header clear while it is scrolled.

import { button } from "../../lib/button.ts";
import type { Words } from "../../lib/content.ts";
import { row, stack } from "../../lib/layout.ts";
import { type ScrollOptions, scroll } from "../../lib/scroll.ts";
import { text, wrapped } from "../../lib/text.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

/** The scroll's room in the panel: six rows tall. */
const TALL = "calc(6 * var(--size-row))";

const LINES = [
  "Screenshot saved to Pictures",
  "Firefox finished downloading kubaos-0.4.iso",
  "Battery at 20%: plug in soon",
  "Kissa asked to share your screen",
  "Updates installed: restart to finish",
  "Hoshi 5G connected",
  "Backup to the NAS done in 4 minutes",
  "Bluetooth: WH-1000XM5 connected",
  "Calendar: stand-up in 10 minutes",
  "Disk nearly full: 6 GB left",
];

const LINE: Words = { short: "Saved", long: "Firefox finished downloading kubaos-0.4-x86_64-installer-with-everything.iso to Downloads" };

/** What the notifications panel holds: a button at its top, whose glow the scroll keeps, then lines. */
function content(lines: readonly string[] = LINES, first?: string): HTMLElement {
  const clear = button({ label: "Clear all", kind: "Tonal", size: "Sm" });
  const words = (first !== undefined ? [first, ...lines.slice(1)] : lines).map((each) => wrapped(each, "body", "on_surface_variant"));
  return stack("s12", "start", clear, ...words);
}

/** The notifications panel: its header, then its content in a scroll, a header fade between them. */
function panel(options: Partial<ScrollOptions> = {}, held = content()): HTMLElement {
  const box = document.createElement("span");
  box.className = "k-scroll-panel k-sunken";
  const header = row("s8", "centre", text("Notifications", "heading"));
  const { box: scrolled } = scroll(held, { height: TALL, above: "s16", below: "s16", fade: "s16", ...options });
  box.append(header, scrolled);
  return box;
}

const MATRIX = [
  { name: "At its top", down: 0 },
  { name: "Scrolled", down: 0.5 },
] as const;

type MatrixState = "rest" | "lit";

function cell(down: number, state: MatrixState): HTMLElement {
  const lit = state === "lit";
  return panel({ down, ...(lit ? { thumbPinned: true } : {}), ...(down > 0 ? { fadePinned: true } : {}) });
}

/** The panel with its first control lit, at rest at the scroll's top: its whole glow shows. */
function glowing(): HTMLElement {
  const box = panel();
  box.querySelector<HTMLElement>(".k-button")!.dataset.pinned = "lit";
  return box;
}

export default page({
  level: "molecules",
  slug: "scroll",
  title: "Scroll and HeaderFade",
  lede: "A page's or panel's content taller than its room: it reaches past its box, and a fade keeps the header's gap clear.",
  code: { rust: "layout::scroll", qml: "Scroll, HeaderFade", unbuilt: "Rust HeaderFade: not built yet" },
  stage: { live: () => panel(), hint: "scroll it; Tab to Clear all", daylight: true },
  variants: [
    { name: "At its top", for: "no fade; the first control's whole glow shows", sample: glowing },
    { name: "Scrolled", for: "the gap clear, the fade under it, the thumb half way", sample: () => panel({ down: 0.5, fadePinned: true }) },
    { name: "At its end", for: "the last line in the panel's padding", sample: () => panel({ down: 1, fadePinned: true }) },
  ],
  sizes: doesntApply("as big as its place; its thumb is `size.scrollbar` wide."),
  states: {
    shown: ["rest", "lit"],
    absent: {
      pressed: "dragging the thumb is lit",
      disabled: "what it holds may be, never it",
      selected: "nothing of its own is picked",
      loading: "what it holds shows its own",
      empty: "content that fits: no thumb, no fade",
      error: "what it holds shows its own",
    },
    rows: nonEmpty(MATRIX.map(({ name, down }) => ({ name, cell: (state: MatrixState) => cell(down, state) }))),
    looks: {
      rest: "thumb `outline_variant`; scrolled, the fade over the gap in `surface`",
      lit: "thumb `outline`, under the pointer or dragged; no glow",
    },
  },
  focus: doesntApply("no keyboard of its own: the controls in it light, their glow kept whole."),
  content: {
    text: [{ name: "A notification", words: LINE, sample: (words: string) => panel({}, content(LINES, words)) }],
    missingPicture: doesntApply("it draws no pictures."),
    counts: [
      { name: "Fits", for: "three lines: no thumb, no fade", sample: () => panel({}, content(LINES.slice(0, 3))) },
      { name: "Taller than its room", for: "ten lines: the thumb, and the fade once scrolled", sample: () => panel({ down: 0.3, fadePinned: true }) },
    ],
  },
  motion: {
    arrives: doesntApply("a scroll comes with its panel."),
    leaves: doesntApply("it goes with its panel."),
    between: [
      {
        name: "The fade comes",
        what: "Leaving its top, the fade comes in over `micro` on `standard`; back at the top it goes.",
        tracks: [{ part: "the header fade", on: "the notifications", target: ".k-scroll", property: "--faded", from: 0, to: 1, duration: "micro", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => panel({ down: 0.3 }),
      },
      {
        name: "The thumb lights",
        what: "The pointer onto the thumb: `outline_variant` to `outline` over `micro`, and back as it leaves.",
        tracks: [{ part: "the thumb", on: "the notifications", target: ".k-scroll-thumb", property: "--scroll-thumb-lit", from: 0, to: 1, duration: "micro", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => panel({ down: 0.3, fadePinned: true }),
      },
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "through its controls; one out of view glides in" },
      { keys: ["↑", "↓", "Page Up", "Page Down"], does: "its owner's: a line or a room at a time" },
    ],
    order: "No stop of its own: Tab goes through what it holds.",
    lands: "Where its panel puts the keyboard; a control scrolled into view lands whole.",
  },
  measures: {
    samples: nonEmpty([
      { name: "At its top", for: "reach past the box, thumb's room, glow kept whole", sample: glowing },
      { name: "Scrolled", for: "the fade's gap and fade, the thumb's width", sample: () => panel({ down: 0.5, fadePinned: true }) },
    ]),
    measures: [
      { part: "Reach past its sides", token: "focus-far" },
      { part: "Room above and below, fade", token: "space-s16" },
      { part: "Thumb width", token: "size-scrollbar" },
      { part: "Panel width", token: "size-panel" },
    ],
    reflow: doesntApply("it fills its place at every window size."),
  },
});
