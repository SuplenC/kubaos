// The Divider page: `docs/design/components/layout.md`, "Divider". A hairline across all the room
// it is given, in the soft outline, one device pixel at every scale. It isn't a control: it rests,
// by night and by day, and carries no meaning, so no contrast minimum holds it.

import { keyHints } from "../../lib/keycap.ts";
import { divider, row, stack, within } from "../../lib/layout.ts";
import { clamped, text } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

/** Two blocks with a divider between them, in a room `width` wide, `gap` clear of each. */
function between(width = "var(--size-menu)", gap: string | 0 = "s12"): HTMLElement {
  return within(width, stack(gap, "start", text("Wi-Fi", "body_strong"), divider(), text("Bluetooth", "body_strong")));
}

/** A menu's two groups with the separator between them: the hairline `space.s4` clear of each. */
function menuGroups(): HTMLElement {
  return within("var(--size-menu)", stack("s4", "start", clamped("Open in a new window", "body"), clamped("Copy the link", "body"), divider(), clamped("Delete", "body", "error")));
}

/** The Command menu's footer under its hairline. */
function footer(): HTMLElement {
  return within("var(--size-menu)", stack("s12", "start", divider(), keyHints([["↵", "Open"], ["Ctrl+K", "Actions"]])));
}

export default page({
  level: "atoms",
  slug: "divider",
  title: "Divider",
  lede: "A hairline across all the room it is given, between two parts a gap can't keep apart.",
  code: { rust: "layout::divider", qml: "Divider" },
  stage: { live: () => row("s48", "start", between(), stack("s24", "start", menuGroups(), footer())), hint: "try Scale: always one device pixel", daylight: true },
  variants: [
    { name: "Between two parts", for: "in a stack, its gap either side", sample: () => between() },
    { name: "Between a menu's groups", for: "`space.s4` clear of each group", sample: menuGroups },
    { name: "Over a footer", for: "the Command menu's footer", sample: footer },
  ],
  sizes: doesntApply("one device pixel tall at every scale, as wide as its room."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "no pointer and no focus",
      pressed: "it takes no press",
      disabled: "unchanged beside disabled controls",
      selected: "it holds no choice",
      loading: "it comes with its surface",
      empty: "with nothing on one side it isn't drawn",
      error: "it has no value to be wrong",
    },
    rows: [
      { name: "Between two parts", cell: () => between() },
      { name: "Between a menu's groups", cell: menuGroups },
    ],
    looks: {
      rest: "`outline_variant` at `opacity.outline_soft`; decoration, so no contrast minimum",
    },
  },
  focus: doesntApply("not a control and not neon: no line and no glow."),
  content: {
    text: doesntApply("it holds no words."),
    missingPicture: doesntApply("it is drawn."),
    counts: [
      { name: "One", for: "between two parts", sample: () => between() },
      {
        name: "Many",
        for: "between each group; two never meet",
        sample: () => within("var(--size-menu)", stack("s12", "start", text("Network", "body_strong"), divider(), text("Sound", "body_strong"), divider(), text("Display", "body_strong"))),
      },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its surface."),
    leaves: doesntApply("it goes with its surface."),
    between: powerMoves(() => between()),
  },
  keyboard: doesntApply("no focus and no key, and it isn't read out."),
  measures: {
    samples: nonEmpty([
      { name: "Between two parts", for: "the hairline and the stack's gap", sample: () => between() },
      { name: "Between a menu's groups", for: "the separator's room", sample: menuGroups },
    ]),
    measures: [
      { part: "Height", token: "stroke-hairline_device_px" },
      { part: "Menu separator's room", token: "space-s4" },
      { part: "The samples' stack gap", token: "space-s12" },
      { part: "The samples' room", token: "size-menu" },
    ],
    reflow: {
      note: "It runs the whole width of its room at every window size.",
      sample: () => within("100%", stack("s12", "start", row("s12", "centre", text("KubaOS", "body_strong")), divider(), text("The page's body", "body", "on_surface_variant"))),
    },
  },
});
