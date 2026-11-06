// The Pill page: `docs/design/components/pill.md`. A group in the bar: a pill a small control tall
// holding symbols and short words side by side, the shell's alone. An interactive one takes the
// pointer and lights under it, but is never a stop for Tab.

import { badgedIcon } from "../../lib/badge.ts";
import type { Words } from "../../lib/content.ts";
import { type Shown, stateOptions } from "../../lib/control.ts";
import { icon } from "../../lib/icon.ts";
import { row, stack } from "../../lib/layout.ts";
import { makePill, type PillOptions, type PillTone, pill } from "../../lib/pill.ts";
import { text } from "../../lib/text.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

const LAYOUT: Words = { short: "PL", long: "Polish (programmer's)" };

/** A symbol in the pill's ink, the bar's 16. */
function glyph(name: string): SVGElement {
  return icon(name, "var(--icon-s16)", "var(--ink)");
}

/** Words in the pill's ink, `button_small`, as the bar sets them. */
function words(said: string): HTMLElement {
  const element = text(said, "button_small");
  element.style.color = "var(--ink)";
  return element;
}

const STATUS = (): Element[] => [glyph("wifi"), glyph("volume_up"), glyph("battery_full"), words("87%")];

/** The bar's groups, as its right end lays them out. */
function bar(state: Shown = "rest"): HTMLElement {
  return row(
    "s8",
    "centre",
    pill({ content: [glyph("screen_record"), words("0:42")], lone: false, tone: "recording", interactive: true, tooltip: "Stop recording", ...stateOptions(state) }),
    pill({ content: [words("Preview")], lone: false, tone: "preview" }),
    pill({ content: [words("PL")], lone: false, interactive: true, tooltip: "Keyboard layout", ...stateOptions(state) }),
    pill({ content: STATUS(), lone: false, interactive: true, tooltip: "Wi-Fi Home 5G · Volume 60% · Battery 87%", ...stateOptions(state) }),
    pill({ content: [badgedIcon({ name: "notifications", badged: true, accent: true })], lone: true, interactive: true, tooltip: "Notifications", ...stateOptions(state) }),
  );
}

/** A pill whose content changes on a press: it eases to its new width. */
function growing(): HTMLElement {
  const layouts = ["PL", "EN", "Polish (programmer's)"];
  let at = 0;
  const made = makePill({ content: [words(layouts[0]!)], lone: false, interactive: true, tooltip: "Keyboard layout: press to switch" });
  made.element.addEventListener("click", () => {
    at = (at + 1) % layouts.length;
    made.change([words(layouts[at]!)]);
  });
  return made.element;
}

function live(): HTMLElement {
  return stack("s24", "start", bar(), growing());
}

type MatrixState = "rest" | "lit" | "pressed";

const ROWS: readonly { name: string; options: Omit<PillOptions, "content">; content: () => Element[] }[] = [
  { name: "Symbols and words", options: { lone: false, interactive: true }, content: STATUS },
  { name: "A lone symbol", options: { lone: true, interactive: true }, content: () => [glyph("notifications")] },
  { name: "Recording", options: { lone: false, interactive: true, tone: "recording" }, content: () => [glyph("screen_record"), words("0:42")] },
];

function cell(index: number, state: Shown): HTMLElement {
  const each = ROWS[index]!;
  return pill({ ...each.options, content: each.content(), ...stateOptions(state) });
}

const TONES: readonly { name: string; tone: PillTone; for: string; content: () => Element[] }[] = [
  { name: "The bar's", tone: "neutral", for: "every group: `surface_container`, `on_surface`", content: STATUS },
  { name: "Recording", tone: "recording", for: "recording: `error_container`, `on_error_container`", content: () => [glyph("screen_record"), words("0:42")] },
  { name: "Preview", tone: "preview", for: "a preview build: `cold` tint, `cold` words", content: () => [words("Preview")] },
];

export default page({
  level: "molecules",
  slug: "pill",
  title: "Pill",
  lede: "A group in the bar: symbols and short words side by side in a pill, the shell's alone.",
  code: { qml: "Pill", unbuilt: "QML only; neon not built yet" },
  stage: { live, hint: "point at one; press PL to see it ease", daylight: true },
  variants: nonEmpty(TONES.map(({ name, tone, for: what, content }) => ({ name, for: what, sample: () => pill({ content: content(), lone: false, tone }) }))),
  sizes: [
    { name: "Symbols and words", for: "`space.s12` at each end", sample: () => pill({ content: STATUS(), lone: false }) },
    { name: "A lone symbol", for: "`space.s8` round it: a circle", sample: () => pill({ content: [glyph("notifications")], lone: true }) },
  ],
  states: {
    shown: ["rest", "lit", "pressed"],
    absent: {
      disabled: "a group with nothing to do just rests",
      selected: "the bar's picks are in its panels",
      loading: "a count it waits for shows on arrival",
      empty: "a group with nothing to show is left out",
      error: "its content or tone says so",
    },
    rows: nonEmpty(ROWS.map(({ name }, index) => ({ name, cell: (state: MatrixState) => cell(index, state) }))),
    looks: {
      rest: "its fill, its content in its ink",
      lit: "pointer only: tube in its tone, glow at night, name hint",
      pressed: "tube white-hot, glow × `focus.pressed_glow`; by day an ink tint",
    },
  },
  focus: {
    tubes: [
      { name: "The bar's", tone: "accent", filled: false, content: false },
      { name: "Recording", tone: "error", filled: false, content: false },
    ],
    looks: {
      lit: "the tube inside the pill's edge, the glow outside",
      signsOff: "the tube as its tone's glass, no glow",
    },
    signsOff: (index: number) => cell(index, "signs-off"),
  },
  content: {
    text: [{ name: "Its words", words: LAYOUT, sample: (said: string) => pill({ content: [words(said)], lone: false }) }],
    missingPicture: doesntApply("its symbols are KubaOS Symbols, always there."),
    counts: [
      { name: "One", for: "a lone symbol", sample: () => pill({ content: [glyph("notifications")], lone: true }) },
      { name: "Many", for: "symbols and words `space.s8` apart", sample: () => pill({ content: STATUS(), lone: false }) },
      { name: "A badged symbol", for: "a `BadgedIcon`'s room is part of its content", sample: () => pill({ content: [badgedIcon({ name: "system_update_alt", badged: true, count: 3 })], lone: true }) },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with the bar, or as its content says, at once."),
    leaves: doesntApply("it goes at once, as its content ends."),
    between: [
      {
        name: "Its content changes width",
        what: "It eases to its new width over `medium` on `standard_decel`, its content cut at its edge.",
        tracks: [{ part: "width", on: "the keyboard layout's pill", target: ".k-pill", property: "--grown", from: 0, to: 1, duration: "medium", curve: "standard_decel" }],
        reduced: [{ part: "width", on: "the keyboard layout's pill", target: ".k-pill", property: "--grown", from: 0, to: 1, duration: "instant", curve: "standard_decel" }],
        back: "plays",
        sample: () => {
          const made = makePill({ content: [words("Polish (programmer's)")], lone: false });
          made.element.style.setProperty("--from", "var(--size-control_sm)");
          made.element.style.setProperty("--to", "var(--size-thumbnail_height)");
          return made.element;
        },
      },
      { name: "Lit", what: "The pointer onto it: it lights up over `short` on `standard`.", tracks: lights(".k-pill", "every pill", 0, 1), reduced: "same", back: "cuts", sample: () => bar() },
      { name: "Back to rest", what: "The pointer off: back to rest over `short` on `standard`.", tracks: lights(".k-pill", "every pill", 1, 0), reduced: "same", back: "cuts", sample: () => bar("lit") },
      { name: "Pressed", what: "Over `micro` the tube runs white-hot and the glow brightens; by day its ink tints it.", tracks: press(".k-pill", "every pill"), reduced: "same", back: "plays", sample: () => bar("lit") },
      ...powerMoves(() => bar("lit")),
    ],
  },
  keyboard: doesntApply("never a stop for Tab: the bar's actions are Items in the Command menu."),
  measures: {
    samples: nonEmpty([
      { name: "Symbols and words", for: "height, ends, gaps", sample: () => pill({ content: STATUS(), lone: false }) },
      { name: "A lone symbol", for: "the ends round a lone symbol", sample: () => pill({ content: [glyph("notifications")], lone: true }) },
    ]),
    measures: [
      { part: "Height", token: "size-control_sm" },
      { part: "Ends, symbols and words", token: "space-s12" },
      { part: "Ends round a lone symbol, gaps", token: "space-s8" },
      { part: "Symbols", token: "icon-s16" },
      { part: "Words' line", token: "type-button_small-line" },
    ],
    reflow: doesntApply("as wide as its content at every window size."),
  },
});
