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

const DESCRIBED =
  "One device pixel tall at every scale (`stroke.hairline_device_px`), the one length off the grid, across all the room it is given: `outline_variant` at `opacity.outline_soft`. It is decoration and carries no meaning, so WCAG's contrast minimum doesn't hold it.";

export default page({
  level: "atoms",
  slug: "divider",
  title: "Divider",
  summary: "A hairline across all the room it is given, in the soft outline: between two parts that would otherwise run together.",
  live: () => stack("s32", "start", between(), menuGroups()),
  description: [
    "Rust `layout::divider`, QML `Divider`. It parts two things that would otherwise run together where space alone can't: a window's bar from its body, a menu's groups, the Command menu's footer from its results. Where a gap is enough, there is no divider.",
    DESCRIBED,
    "It takes no room of its own besides its pixel: what keeps it clear of the parts beside it is the gap of the stack it is in, or a separator's padding (a menu's, `space.s4` above and below). It always runs across; there is no upright divider.",
  ],
  variants: [
    { name: "Between two parts", for: "In a stack, its gap either side: a window's bar and its body.", sample: () => between() },
    { name: "Between a menu's groups", for: "A menu separator: the hairline `space.s4` clear of each group.", sample: menuGroups },
    { name: "Over a footer", for: "The Command menu's footer: the hairline across the panel, the hints under it.", sample: footer },
  ],
  sizes: doesntApply("a divider is one device pixel tall at every scale, and as wide as its room."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "a divider takes no pointer and no focus, and never lights.",
      pressed: "it takes no press.",
      disabled: "it has nothing to disable: beside disabled controls it is unchanged.",
      selected: "it holds no choice.",
      loading: "it comes with its surface.",
      empty: "a divider has no content; a divider with nothing on one side isn't drawn.",
      error: "it has no value to be wrong.",
    },
    rows: [
      { name: "Between two parts", cell: () => between() },
      { name: "Between a menu's groups", cell: menuGroups },
    ],
    described: {
      rest: `Always. ${DESCRIBED} By day the same role in daylight colours: a darker line on the pale surface.`,
    },
  },
  focus: doesntApply("a divider isn't a control and isn't neon: it has no line of its own and no glow, by night or by day."),
  content: {
    text: doesntApply("a divider holds no words."),
    missingPicture: doesntApply("a divider is drawn."),
    counts: [
      { name: "One", for: "Between two parts.", sample: () => between() },
      {
        name: "Many",
        for: "Between each of several groups, each the stack's gap clear of what is beside it; two never meet.",
        sample: () => within("var(--size-menu)", stack("s12", "start", text("Network", "body_strong"), divider(), text("Sound", "body_strong"), divider(), text("Display", "body_strong"))),
      },
    ],
  },
  motion: {
    arrives: doesntApply("a divider comes with the surface it is on."),
    leaves: doesntApply("a divider goes with the surface it is on."),
    between: powerMoves(() => between()),
  },
  keyboard: doesntApply("a divider takes no focus and no key, and isn't read out."),
  measures: {
    samples: nonEmpty([
      { name: "Between two parts", for: "The hairline and the stack's gap either side.", sample: () => between() },
      { name: "Between a menu's groups", for: "The separator's room either side.", sample: menuGroups },
    ]),
    measures: [
      { what: "Its height: one device pixel at every scale.", token: "stroke-hairline_device_px" },
      { what: "A menu separator's room above and below the hairline.", token: "space-s4" },
      { what: "The gap either side in the samples' stack.", token: "space-s12" },
      { what: "The room the samples are given, a menu's width.", token: "size-menu" },
    ],
    reflow: {
      described: "Across the window at every size: the hairline always runs the whole width of its room.",
      sample: () => within("100%", stack("s12", "start", row("s12", "centre", text("KubaOS", "body_strong")), divider(), text("The page's body", "body", "on_surface_variant"))),
    },
  },
});
