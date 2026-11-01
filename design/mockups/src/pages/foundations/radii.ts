// The Radii foundation: the five radius roles and full, each shown as it is used, the padding each
// box takes, and the nesting that derives a row's corner from its box's, worked (`docs/DESIGN.md`,
// "Radii by role").

import { button } from "../../lib/button.ts";
import { buttonBar } from "../../lib/button-bar.ts";
import { raise } from "../../lib/elevation.ts";
import { row, stack } from "../../lib/layout.ts";
import { type Corner, concentric, role } from "../../lib/radius.ts";
import { text } from "../../lib/text.ts";
import { length } from "../../lib/tokens.ts";
import { el, label, prose } from "../../site/dom.ts";
import { doesntApply, foundation } from "../../site/page.ts";
import { atScale, table, token } from "../../site/parts/swatches.ts";
import { SCALES } from "../../site/settings.ts";

const ROLES = ["small", "row", "control", "content", "surface", "full"] as const;
type Role = (typeof ROLES)[number];

/** What each role is for. */
const FOR: Record<Role, string> = {
  small: "Marks inside controls: the checkbox, a keycap, a skeleton's line, a field's selection.",
  row: "A row inside a small or a content box, the box's corner less its padding: a menu's or a select's items, a list well's rows, a table's rows; and a hint.",
  control: "Everything you click or type in: buttons, fields, chips, a calendar's months and title; and the small floating things, raised over what opened them: a popover, a menu, a select's open list, a toast.",
  content: "A content box inside a window or a panel: a card, a list well, a scroll panel, a dialog. A step under a surface, so a box inside reads as a part of what holds it.",
  surface: "The shell's big surfaces alone: a panel, a window, the screen frame.",
  full: "Pills and circles: switches, icon buttons, radios, progress bars, sliders, avatars, badges. Half the shorter side.",
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
  return table(
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
  );
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
    nesting(role("content"), "s8", "A list well's rows: 16 less 8 is 8, the row radius."),
    nesting(role("control"), "s4", "A menu's or a select's items: 12 less 4 is 8, the row radius too."),
    nesting(role("surface"), "s8", "A window 8 inside the screen frame: its outer edge is 16, Hyprland's rounding 14 plus its 2 border."),
    nesting(role("full"), "s4", "A pill in a pill is a pill."),
    nesting(role("content"), "s16", "A picture filling a card less its 16: nothing is left to round, so `nested` refuses it, and the picture takes the row radius, 8."),
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
    ["space.s4", "4", "Around rows in a floating box: a menu, a select's open list."],
    ["space.s8", "8", "Around rows in a content box: a list well."],
    ["space.s12", "12", "Around words in a small floating thing: a popover holding words, a toast. A hint, the smallest, is 8 by 4."],
    ["space.s16", "16", "Around content in a content box: a card, a scroll panel."],
    ["space.s24", "24", "Around a dialog's content, the dialog alone."],
  ];
  return table(
    ["Padding", "px", "For"],
    steps.map(([name, px, what]) => [token(name), label(px, "mono", "on_surface"), prose(what)]),
  );
}

/** A button inside a card keeps the control radius: it isn't filling the card. */
function placedInside(): HTMLElement {
  return el(
    "div",
    "f-card-box",
    stack("s16", "start", text("Erase the disk?", "subheading", "on_surface"), buttonBar({ dismiss: "Cancel", actions: [{ label: "Erase" }], tone: "Danger" }).element),
  );
}

/** The focus tube at its control's corner, its glow concentric with it. */
function focusCorners(): HTMLElement {
  return row("s32", "start", button({ label: "Continue", kind: "Filled", size: "Md", pinned: "lit" }), button({ label: "Save log", kind: "Tonal", size: "Md", pinned: "lit" }));
}

/** Each radius in device pixels at every scale. */
function deviceRadii(): HTMLElement {
  return table(
    ["Role", ...SCALES.map((scale) => `${scale}×`)],
    ROLES.filter((name) => name !== "full").map((name) => [
      token(`radius.${name}`),
      ...SCALES.map((scale) => {
        const value = length(`radius-${name}`) * scale;
        return label(String(value), "mono", Number.isInteger(value) ? "on_surface" : "error");
      }),
    ]),
  );
}

export default foundation({
  level: "foundations",
  slug: "radii",
  title: "Radii",
  summary: "Five corner radii by what a shape is, and full for pills: one ladder of boxes, and rows that follow the box they sit in.",
  live: () => el("div", "f-well", ...["Wi-Fi", "Bluetooth", "Sound"].map((name, i) => wellRow(name, i === 0))),
  description: [
    "Every radius says what the shape is, on one ladder of boxes: small for marks inside controls; row for the rows of a box; control for everything you click or type in and the small floating things (a popover, a menu, a toast); content for a box inside a window (a card, a list well, a scroll panel, a dialog); surface for the shell's big surfaces alone (a panel, a window, the screen frame); and full for pills and circles. A component takes the one its role names, so the same thing has the same corner everywhere, and a corner that differs says it is a different thing.",
    "A row follows the box it sits in: its corner is the box's corner less the box's padding, so both corners share a centre. A menu is 12 with 4 inside and a list well 16 with 8 inside, so both rows are 8, the row radius. A control placed in content (a button in a card) isn't filling it, so it keeps its own role.",
    "A box's padding goes with what it holds: 4 around rows in a floating box, 8 around rows in a content box, 12 around words in a small floating thing, 16 around content in a content box, and 24 in a dialog.",
    "The live list is a well, a content box at 16, with its rows 8 in: they take 16 less 8, the row radius.",
  ],
  tokens: [
    {
      name: "The roles",
      described: "Each role's radius, its corner on a plain shape, and the role in use: a checkbox's box, a button, a dialog, a card, a switch's track.",
      sample: roles,
    },
  ],
  rules: [
    {
      name: "Padding goes with the box",
      described: "Each box's padding, by what it holds. A row's corner is the box's less this padding.",
      sample: paddings,
    },
    {
      name: "A row follows its box",
      described: "Each shape fills the box round it less the box's padding; its corner is the box's less the padding, and the dot is the centre both corners share. A well's rows and a menu's items both come to 8, the row radius. An inset that reaches the radius leaves nothing to round, and the nesting is refused.",
      sample: worked,
    },
    {
      name: "A control in content keeps its role",
      described: "A card's buttons sit inside it without filling it, so they keep the control radius; nesting is only for a shape that fills another less an inset.",
      sample: placedInside,
    },
    {
      name: "The focus tube takes its control's corner",
      described: "The tube is drawn at the control's own corner, 12 on a button, and each ring of its glow is concentric with it, its radius the control's plus its distance out. A pill's glow is a pill.",
      sample: focusCorners,
    },
  ],
  modes: doesntApply("a corner is the same in light and dark."),
  themes: doesntApply("a corner is the same in every theme."),
  scales: [
    {
      name: "Every radius in device pixels",
      described: "Each role's radius at each scale: a multiple of 4, so whole at all four. Full is half the shorter side, whatever that is.",
      sample: deviceRadii,
    },
    {
      name: "The same nesting at every scale",
      described: "A well's rows 8 in, drawn at each scale: the corners stay concentric, on whole device pixels.",
      sample: () => el("div", "f-scroll", el("div", "f-row", ...SCALES.map((scale) => atScale(scale, nesting(role("content"), "s8", ""))))),
    },
  ],
  content: doesntApply("a corner doesn't change with the words inside: a pill stays a pill however long its label."),
  motion: doesntApply("a corner doesn't animate: a shape keeps its role in every state."),
  checks: [
    { check: "kubaos-tokens", holds: "Refuses a radius that isn't a multiple of `grid`." },
    { check: "`nested`", holds: "Refuses an inset that reaches the outer radius; in a `const`, in Rust, the build stops. The mockups' `nested` (`lib/radius.ts`) fails the page instead." },
    { check: "kubaos-design lint", holds: "Fails on a radius written as a number, a shell radius that isn't a role, and a radius derived by hand (`outer - inset`)." },
  ],
});
