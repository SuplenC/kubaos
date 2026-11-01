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
  return stack("s24", "start", bar(), row("s12", "centre", growing(), text("Press it: it eases to its new width.", "small", "on_surface_variant")));
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
  { name: "The bar's", tone: "neutral", for: "Every group: `surface_container`, its content `on_surface`.", content: STATUS },
  { name: "Recording", tone: "recording", for: "The screen being recorded: `error_container`, `on_error_container`.", content: () => [glyph("screen_record"), words("0:42")] },
  { name: "Preview", tone: "preview", for: "A preview build: the `cold` tint at `opacity.tint`, its words `cold`.", content: () => [words("Preview")] },
];

export default page({
  level: "molecules",
  slug: "pill",
  title: "Pill",
  summary: "A group in the bar: symbols and short words side by side in a pill a small control tall, the shell's alone.",
  live,
  description: [
    "QML `Pill`; the shell's alone: the bar is the shell's and no app has one, so it is on the QML specimen's \"Shell only\" page. The status pill (Wi-Fi, volume, battery), the keyboard layout, the notifications' bell, the recording and preview pills are each one.",
    "A pill `size.control_sm` (32) tall at the full radius, on `surface_container`; its content a row `space.s8` apart, centred on device pixels, with `space.s12` at each end between the curve and the content, or `space.s8` round a lone symbol (the bell, the proxy, the updates). Its symbols are the bar's 16 and its words `button_small`. A pill of its own tone takes its fill and its ink together: the recording pill's `error_container` with `on_error_container`, the preview pill's `cold` tint with `cold`.",
    "It isn't a `Chip` or a `Badge`. A chip is a fact or a pick inside a surface, `size.chip` (24) tall, a `label`'s words at the control radius; a badge is a count or a dot hung on a symbol's corner. A pill is the bar's own group, as tall as the bar's controls, holding the bar's symbols and words, with the bar's actions behind it.",
    "An interactive pill takes the pointer: a click, a right click and the wheel (a step per notch). Its states follow the neon, lit by the pointer alone: lit, its tube burns inside its edge in its tone (the accent, or the recording pill's error) with the glow outside at night; pressed, the tube runs white-hot, the glow brightens, and by day its ink tints it (`opacity.press_tint`). It is never a stop for Tab and the keyboard never lights it, so a panel's keyboard focus never wanders into the bar: the bar's actions are Items in the Command menu. While lit, its name shows in a `Hint` under it, moved sideways as far as keeps it whole in the window, `space.s8` from its edge, and kept there as its words change.",
    "When its content changes width, it eases to the new width over `medium` on `standard_decel`, as a list reorders, and cuts its content at its own edge meanwhile, so nothing draws over its neighbours. Content that animates its own width every frame isn't put in a pill: the workspaces draw a pill of their own.",
  ],
  variants: nonEmpty(TONES.map(({ name, tone, for: what, content }) => ({ name, for: what, sample: () => pill({ content: content(), lone: false, tone }) }))),
  sizes: [
    { name: "Symbols and words", for: "`space.s12` at each end.", sample: () => pill({ content: STATUS(), lone: false }) },
    { name: "A lone symbol", for: "`space.s8` round it: a circle as tall as the bar's controls.", sample: () => pill({ content: [glyph("notifications")], lone: true }) },
  ],
  states: {
    shown: ["rest", "lit", "pressed"],
    absent: {
      disabled: "the bar shows only what can be used: a group with nothing to do takes no pointer, and rests.",
      selected: "a pill holds no choice; the bar's picks are in the panels it opens.",
      loading: "what a pill shows is known as the bar draws it; a count it waits for shows when it arrives.",
      empty: "a group with nothing to show is no pill: the bar leaves it out.",
      error: "a pill says something is wrong with its content (a badge, a symbol) or its tone (recording).",
    },
    rows: nonEmpty(ROWS.map(({ name }, index) => ({ name, cell: (state: MatrixState) => cell(index, state) }))),
    described: {
      rest: "Its fill and its content in its ink; a pill that takes no pointer always rests.",
      lit: "The pointer on it, here with the signs on: its tube inside its edge in its tone, the glow outside; its name in a hint under it. By day the tube with no glow.",
      pressed: "Lit and pressed: the tube runs white-hot, the glow `focus.pressed_glow` times the lit one; by day its ink tints it at `opacity.press_tint`.",
    },
  },
  focus: {
    tubes: [
      { name: "The bar's", tone: "accent", filled: false, content: false },
      { name: "Recording", tone: "error", filled: false, content: false },
    ],
    looks: {
      lit: "At night, under the pointer, the sign burns: the tube inside the pill's edge, a pill itself, the glow outside it.",
      signsOff: "By day always, and at night while a power change has the lights out: the tube is its tone's glass, with no glow.",
    },
    signsOff: (index: number) => cell(index, "signs-off"),
  },
  content: {
    text: [{ name: "Its words", words: LAYOUT, sample: (said: string) => pill({ content: [words(said)], lone: false }) }],
    missingPicture: doesntApply("its symbols are KubaOS Symbols, always there; an app's tray icon is the tray's, on its own page."),
    counts: [
      { name: "One", for: "A lone symbol.", sample: () => pill({ content: [glyph("notifications")], lone: true }) },
      { name: "Many", for: "Symbols and words `space.s8` apart.", sample: () => pill({ content: STATUS(), lone: false }) },
      { name: "A badged symbol", for: "A `BadgedIcon`'s room on both sides is part of its content.", sample: () => pill({ content: [badgedIcon({ name: "system_update_alt", badged: true, count: 3 })], lone: true }) },
    ],
  },
  motion: {
    arrives: doesntApply("a pill comes with the bar, or as the bar's content says (recording starting), at once."),
    leaves: doesntApply("it goes at once, as its content ends."),
    between: [
      {
        name: "Its content changes width",
        what: "New content (a longer layout's name): the pill eases from its old width to its new over `medium` on `standard_decel`, its content cut at its edge meanwhile.",
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
      { name: "Lit", what: "The pointer comes onto it: it lights up over `short` on `standard`.", tracks: lights(".k-pill", "every pill", 0, 1), reduced: "same", back: "cuts", sample: () => bar() },
      { name: "Back to rest", what: "The pointer leaves: back to rest over `short` on `standard`.", tracks: lights(".k-pill", "every pill", 1, 0), reduced: "same", back: "cuts", sample: () => bar("lit") },
      { name: "Pressed", what: "A lit pill pressed: over `micro` the tube runs white-hot and the glow brightens; by day its ink tints it.", tracks: press(".k-pill", "every pill"), reduced: "same", back: "plays", sample: () => bar("lit") },
      ...powerMoves(() => bar("lit")),
    ],
  },
  keyboard: doesntApply("a pill is never a stop for Tab: the bar's actions are Items in the Command menu, so none needs the bar's keyboard focus."),
  measures: {
    samples: nonEmpty([
      { name: "Symbols and words", for: "Its height, the ends, and the gaps between its content.", sample: () => pill({ content: STATUS(), lone: false }) },
      { name: "A lone symbol", for: "The ends round a lone symbol.", sample: () => pill({ content: [glyph("notifications")], lone: true }) },
    ]),
    measures: [
      { what: "Its height.", token: "size-control_sm" },
      { what: "At each end, round symbols and words.", token: "space-s12" },
      { what: "At each end round a lone symbol, and between its content.", token: "space-s8" },
      { what: "Its symbols.", token: "icon-s16" },
      { what: "Its words' line.", token: "type-button_small-line" },
    ],
    reflow: doesntApply("a pill is as wide as its content at every window size; the bar gives each group its room."),
  },
});
