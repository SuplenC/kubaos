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
  box.className = "k-scroll-panel";
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
  summary: "A page's or a panel's content taller than its room, reaching past its box so nothing at its edge is cut, and the fade that keeps a header's gap clear.",
  live: () => panel(),
  description: [
    "Rust `layout::scroll`, QML `Scroll` and `HeaderFade`; iced has no fade yet. Scroll the notifications: as they leave their top, the gap under the header stays clear and they fade into the panel under it. Tab to Clear all at the top: its glow shows whole.",
    "A page's or a panel's scroll has no edge of its own, so it reaches past its box into the room around it: `focus.far` (20) at its sides, as far as a glow reaches, and above and below up to the part of the panel there (`room_above`, `room_below`: the gap under the header, the panel's padding), with its content that far in. So the content lines up with what is around it, a resting control at its edge keeps its whole glow, content scrolled past its box shows in that room and stops at the part beside it, and the panel's own edge cuts whatever passes it. A scroll with an edge of its own (a list's well, a log, a menu) clips at that edge instead: the ScrollingList page.",
    "The thumb is `size.scrollbar` (4) wide on the box's end, a pill in `outline_variant`, `outline` under the pointer and while dragged, floating over the content along the whole room the scroll shows, its length the share of the content shown. Content that runs to the scroll's end leaves `space.s16` there for it, so the thumb never covers it and never takes the pointer from it. It shows only while the content is taller than its room. A move the owner makes (scrolling a row into view) glides over `medium` on `standard`; the wheel and the touchpad move it as they turn.",
    "A HeaderFade lies over the gap between a panel's header (or the Command menu's field) and the scroll under it: while the content is scrolled from its top, the gap (`gap`, a panel's step of `space.s16`) stays clear in `surface` and the content fades into the panel's surface over the next `space.s16`, rather than being cut flat under the header's words. It spans the scroll's box but the thumb's width at its end, so the thumb is never covered. It fades in and out over `micro` on `standard` as the content leaves its top and comes back, so at rest a focused first control keeps its whole glow. The system monitor's processes, the notifications panel and the Command menu's forms.",
    "A Dialog marks the same \"more this way\" differently: its middle has edges of its own, between its title and its buttons inside its padding, so it draws a soft hairline at whichever end has more, fading over `short`. A panel's scroll has no such edge to draw a line on, so it fades the content under its header instead.",
  ],
  variants: [
    { name: "At its top", for: "At rest: no fade, the first control's whole glow shows past the box.", sample: glowing },
    { name: "Scrolled", for: "Content passing under the header: the gap clear, the fade under it, the thumb half way.", sample: () => panel({ down: 0.5, fadePinned: true }) },
    { name: "At its end", for: "The last line in the panel's padding above its edge.", sample: () => panel({ down: 1, fadePinned: true }) },
  ],
  sizes: doesntApply("a scroll is as big as its place; its thumb is always `size.scrollbar` wide."),
  states: {
    shown: ["rest", "lit"],
    absent: {
      pressed: "the scroll takes no press; dragging the thumb is lit.",
      disabled: "a scroll is never disabled; what it holds may be.",
      selected: "nothing of its own is picked.",
      loading: "what it holds shows its own loading.",
      empty: "content that fits doesn't scroll: no thumb, no fade.",
      error: "what it holds shows its own errors.",
    },
    rows: nonEmpty(MATRIX.map(({ name, down }) => ({ name, cell: (state: MatrixState) => cell(down, state) }))),
    described: {
      rest: "The thumb in `outline_variant`; scrolled, the header fade over the gap in `surface`. By day the same roles in daylight colours.",
      lit: "The pointer on the thumb, or dragging it: `outline`. No line and no glow: a scroll isn't neon.",
    },
  },
  focus: doesntApply("a scroll takes no keyboard of its own: the controls in it light as their pages say, and keep their whole glow at its edges."),
  content: {
    text: [{ name: "A notification", words: LINE, sample: (words: string) => panel({}, content(LINES, words)) }],
    missingPicture: doesntApply("it draws no pictures."),
    counts: [
      { name: "Fits", for: "Three lines: nothing scrolls, so no thumb and never a fade.", sample: () => panel({}, content(LINES.slice(0, 3))) },
      { name: "Taller than its room", for: "Ten lines: the thumb, and the fade once scrolled.", sample: () => panel({ down: 0.3, fadePinned: true }) },
    ],
  },
  motion: {
    arrives: doesntApply("a scroll comes with its panel."),
    leaves: doesntApply("it goes with its panel."),
    between: [
      {
        name: "The fade comes",
        what: "The content leaves its top: the header fade fades in over `micro` on `standard`; back at the top, it fades out the same way.",
        tracks: [{ part: "the header fade", on: "the notifications", target: ".k-header-fade", property: "--faded", from: 0, to: 1, duration: "micro", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => panel({ down: 0.3 }),
      },
      {
        name: "The thumb lights",
        what: "The pointer onto the thumb: `outline_variant` to `outline` over `micro` on `standard`, and back as it leaves.",
        tracks: [{ part: "the thumb", on: "the notifications", target: ".k-scroll-thumb", property: "background-color", from: { color: "outline_variant" }, to: { color: "outline" }, duration: "micro", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => panel({ down: 0.3, fadePinned: true }),
      },
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "To the controls it holds, in reading order; one out of view scrolls in, gliding over `medium`." },
      { keys: ["↑", "↓", "Page Up", "Page Down"], does: "The owner's: a panel that scrolls by keys moves its content a line or a room at a time." },
    ],
    order: "A scroll is no stop of its own: Tab goes through what it holds.",
    lands: "Where its panel puts the keyboard; a control scrolled into view lands whole, its glow inside the scroll's reach.",
  },
  measures: {
    samples: nonEmpty([
      { name: "At its top", for: "The reach past the box at the sides and above, the thumb's room at the end, the glow kept whole.", sample: glowing },
      { name: "Scrolled", for: "The header fade's gap and fade, and the thumb's width it leaves.", sample: () => panel({ down: 0.5, fadePinned: true }) },
    ]),
    measures: [
      { what: "How far it reaches past its sides: a glow's reach.", token: "focus-far" },
      { what: "The room it reaches into above and below here, the gap under the header and the panel's padding; the fade's gap, then its fade.", token: "space-s16" },
      { what: "The thumb's width, which the fade leaves.", token: "size-scrollbar" },
      { what: "The panel's width.", token: "size-panel" },
    ],
    reflow: doesntApply("a scroll fills its place at every window size; the room it reaches into is its owner's, as above."),
  },
});
