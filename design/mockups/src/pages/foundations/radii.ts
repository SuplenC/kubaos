// The Radii foundation: the five radius roles and full, each shown as it is used, the padding each
// box takes, and the nesting that derives a row's corner from its box's, worked (`docs/DESIGN.md`,
// "Radii by role").

import { button } from "../../lib/button.ts";
import { buttonBar } from "../../lib/button-bar.ts";
import { card } from "../../lib/card.ts";
import { raise } from "../../lib/elevation.ts";
import { row, stack, within } from "../../lib/layout.ts";
import { type Corner, concentric, role } from "../../lib/radius.ts";
import { text } from "../../lib/text.ts";
import { length } from "../../lib/tokens.ts";
import { el, label, prose } from "../../site/dom.ts";
import { doesntApply, foundation } from "../../site/page.ts";
import { atScale, table, token } from "../../site/parts/swatches.ts";
import { SCALES } from "../../site/settings.ts";
import { across } from "./across.ts";

const ROLES = ["small", "row", "control", "content", "surface", "full"] as const;
type Role = (typeof ROLES)[number];

/** What each role is for. */
const FOR: Record<Role, string> = {
  small: "Marks inside controls: a checkbox, a keycap, a skeleton's line",
  row: "A box's rows: a menu's items, a list well's rows; a hint",
  control: "What you click or type in; a popover, a menu, a toast",
  content: "A box in a window or panel: a card, a well, a dialog",
  surface: "The shell's big surfaces: a panel, a window, the screen frame",
  full: "Pills and circles: half the shorter side",
};

/** A shape with `corner`, of the class that sizes it. */
function shape(corner: Corner, kind: string): HTMLElement {
  const box = el("div", `f-shape ${kind}`);
  box.style.setProperty("--corner", corner.css);
  return box;
}

/** Each role as it is used: a checkbox's box, a row, a button, a dialog, a panel, a switch's track. */
function inUse(name: Role): HTMLElement {
  switch (name) {
    case "small":
      return shape(role("small"), "f-shape-check");
    case "row":
      return wellRow("Wi-Fi", true);
    case "control":
      return button({ label: "Continue", kind: "Filled", size: "Md" });
    case "content":
      return raise(el("div", "f-card-box f-dialog-box", stack("s4", "start", text("A dialog", "heading", "on_surface"), text("A box inside the window", "body", "on_surface_variant"))), 3);
    case "surface":
      return el("div", "f-card-box f-surface-box", stack("s4", "start", text("A panel", "subheading", "on_surface"), text("The shell's own surface", "body", "on_surface_variant")));
    case "full":
      return shape(role("full"), "f-shape-switch");
  }
}

function roles(): HTMLElement {
  return across(table(
    ["Role", "Radius", "Its corner", "In use", "For"],
    ROLES.map((name) => {
      const corner = role(name);
      return [
        token(`radius.${name}`),
        label(corner.px === null ? "half the shorter side" : String(corner.px), "mono", "on_surface"),
        shape(corner, "f-shape-wide"),
        inUse(name),
        prose(FOR[name]),
      ];
    }),
  ));
}

/**
 * A shape with `outer`'s corner, and inside it one filling it less `inset`, its corner nested, the
 * centre both corners share marked; or, where the inset reaches the radius, the nesting refused.
 */
function nesting(outer: Corner, inset: string, what: string): HTMLElement {
  const inner = concentric(outer, inset);
  const box = el("div", "f-nest", el("div", "f-nest-inner"), el("div", "f-centre"));
  box.style.setProperty("--outer", outer.css);
  box.style.setProperty("--inset", `var(--space-${inset})`);
  box.style.setProperty("--inner", inner?.css ?? "0px");
  box.toggleAttribute("data-refused", inner === null);
  // A pill's corners are centred half its height in.
  box.style.setProperty("--centre", outer.px === null ? "calc(var(--size-thumbnail_height) / 2)" : outer.css);
  const sum = inner
    ? `${outer.words} ${outer.px ?? "full"} − space.${inset} ${length(`space-${inset}`)} = ${inner.px ?? "full"}`
    : `${outer.words} ${outer.px} − space.${inset} ${length(`space-${inset}`)}: refused, nothing is left to round`;
  return el("div", "f-swatch-cell f-nest-cell", box, token(sum), ...(what ? [prose(what, "body", inner ? "on_surface" : "error")] : []));
}

/** The worked examples: a well's rows, a menu's items, a window in the frame, a pill in a pill, one refused. */
function worked(): HTMLElement {
  return el(
    "div",
    "f-row",
    nesting(role("content"), "s8", "A list well's rows"),
    nesting(role("control"), "s4", "A menu's items"),
    nesting(role("surface"), "s8", "A window in the screen frame"),
    nesting(role("full"), "s4", "A pill in a pill"),
    nesting(role("content"), "s16", "A picture in a card: it takes `radius.row`"),
  );
}

/** A list well's row, at the row radius, lit when `lit`. */
function wellRow(name: string, lit: boolean): HTMLElement {
  const rowBox = el("div", "f-well-row", text(name, "body_strong", lit ? "on_secondary_container" : "on_surface"));
  rowBox.style.setProperty("--corner", role("row").css);
  rowBox.toggleAttribute("data-selected", lit);
  return rowBox;
}

/** The padding each box takes, by what it holds. */
function paddings(): HTMLElement {
  const steps: readonly [string, string, string][] = [
    ["space.s4", "4", "Rows in a floating box: a menu, a select's list"],
    ["space.s8", "8", "Rows in a content box: a list well"],
    ["space.s12", "12", "Words in a small floating thing: a popover, a toast"],
    ["space.s16", "16", "Content in a card or a shell panel"],
    ["space.s24", "24", "A dialog's content"],
  ];
  return across(table(
    ["Padding", "px", "For"],
    steps.map(([name, px, what]) => [token(name), label(px, "mono", "on_surface"), prose(what)]),
  ));
}

/** A button inside a card keeps the control radius: it isn't filling the card. */
function placedInside(): HTMLElement {
  return within("var(--size-column)", card({ header: { title: "Erase the disk?" }, actions: buttonBar({ dismiss: "Cancel", actions: [{ label: "Erase" }], tone: "Danger" }).element }));
}

/** The focus tube at its control's corner, its glow concentric with it. */
function focusCorners(): HTMLElement {
  return row("s32", "start", button({ label: "Continue", kind: "Filled", size: "Md", pinned: "lit" }), button({ label: "Save log", kind: "Tonal", size: "Md", pinned: "lit" }));
}

/** Each radius in device pixels at every scale. */
function deviceRadii(): HTMLElement {
  return across(table(
    ["Role", ...SCALES.map((scale) => `${scale}×`)],
    ROLES.filter((name) => name !== "full").map((name) => [
      token(`radius.${name}`),
      ...SCALES.map((scale) => {
        const value = length(`radius-${name}`) * scale;
        return label(String(value), "mono", Number.isInteger(value) ? "on_surface" : "error");
      }),
    ]),
  ));
}

/** A well's rows beside every other role, each as it is used, under its token. */
function ladder(): HTMLElement {
  const used = (name: Role) => el("div", "f-swatch-cell", label(`radius.${name}`, "mono", "on_surface_variant"), inUse(name));
  const well = el("div", "f-well", ...["Wi-Fi", "Bluetooth", "Sound"].map((name, i) => wellRow(name, i === 0)));
  return el("div", "f-row", el("div", "f-swatch-cell", label("radius.content, radius.row", "mono", "on_surface_variant"), well), ...(["small", "full", "control", "content", "surface"] as const).map(used));
}

export default foundation({
  level: "foundations",
  slug: "radii",
  title: "Radii",
  lede: "Five corner radii by what a shape is, and full for pills; a row follows the box it sits in.",
  stage: { live: ladder, hint: "pick a Scale: the corners stay concentric", daylight: true },
  tokens: [{ name: "The roles", note: "Each role's radius, its corner on a plain shape, and the role in use.", sample: roles }],
  rules: [
    { name: "Padding goes with the box", note: "Each box's padding, by what it holds; a row's corner is the box's less this.", sample: paddings },
    { name: "A row follows its box", note: "The row's corner is the box's less its padding; the dot is the centre they share.", sample: worked },
    { name: "A control keeps its role", note: "A card's button doesn't fill it, so it keeps `radius.control`.", sample: placedInside },
    { name: "The tube takes its corner", note: "The tube is drawn at the control's corner; each ring of its glow is concentric with it.", sample: focusCorners },
  ],
  modes: doesntApply("a corner is the same in light and dark."),
  themes: doesntApply("a corner is the same in every theme."),
  scales: [
    { name: "Every radius in device pixels", note: "Multiples of 4, whole at all four; full is half the shorter side.", sample: deviceRadii },
    {
      name: "The same nesting",
      note: "A well's rows 8 in: concentric, on whole device pixels.",
      sample: () => el("div", "f-scroll", el("div", "f-row", ...SCALES.map((scale) => atScale(scale, nesting(role("content"), "s8", ""))))),
    },
  ],
  content: doesntApply("a pill stays a pill however long its label."),
  motion: doesntApply("a shape keeps its corner in every state."),
  checks: [
    { check: "kubaos-tokens", holds: "refuses a radius off `grid`" },
    { check: "`nested`", holds: "refuses an inset reaching the radius: Rust's build stops, the page fails" },
    { check: "kubaos-design lint", holds: "fails a radius as a number or derived by hand" },
  ],
});
