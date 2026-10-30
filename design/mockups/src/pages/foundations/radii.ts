// The Radii foundation: the three radius roles and full, each shown as it is used, and the
// nesting that derives every other corner, worked (`docs/DESIGN.md`, "Radii by role").

import { button } from "../../lib/button.ts";
import { row, stack } from "../../lib/layout.ts";
import { type Corner, concentric, nested, role } from "../../lib/radius.ts";
import { text } from "../../lib/text.ts";
import { length } from "../../lib/tokens.ts";
import { el, label, prose } from "../../site/dom.ts";
import { doesntApply, foundation } from "../../site/page.ts";
import { atScale, table, token } from "../../site/parts/swatches.ts";
import { SCALES } from "../../site/settings.ts";

const ROLES = ["small", "control", "surface", "full"] as const;
type Role = (typeof ROLES)[number];

/** What each role is for. */
const FOR: Record<Role, string> = {
  small: "Marks inside controls: the checkbox, a hint, focus details, inline tags.",
  control: "Everything you click or type in: buttons, fields, list rows, menu items, chips.",
  surface: "Everything that holds content: cards, dialogs, popups, toasts, panels, list wells, the screen frame.",
  full: "Pills and circles: switches, icon buttons, radios, progress bars, sliders, avatars, badges. Half the shorter side.",
};

/** A shape with `corner`, of the class that sizes it. */
function shape(corner: Corner, kind: string): HTMLElement {
  const box = el("div", `f-shape ${kind}`);
  box.style.setProperty("--corner", corner.css);
  return box;
}

/** Each role as it is used: a checkbox's box, a button, a card, a switch's track. */
function inUse(name: Role): HTMLElement {
  switch (name) {
    case "small":
      return shape(role("small"), "f-shape-check");
    case "control":
      return button({ label: "Continue", kind: "Filled", size: "Md" });
    case "surface":
      return el("div", "f-card-box", stack("s4", "start", text("A card", "subheading", "on_surface"), text("Its corner says it holds content", "body", "on_surface_variant")));
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

/** The worked examples: a well's rows, a card in a surface, a window in the frame, a pill in a pill, one refused. */
function worked(): HTMLElement {
  const sixteen = nested(role("surface"), "s8");
  return el(
    "div",
    "f-row",
    nesting(role("surface"), "s12", "A list well's rows: 24 less 12 is 12, the control radius, so a row is both concentric and a control."),
    nesting(role("surface"), "s8", "A card filling a surface 8 in."),
    nesting(sixteen, "s4", "4 in from that 16."),
    nesting(role("surface"), "s8", "A window 8 inside the screen frame: its outer edge is 16, Hyprland's rounding 14 plus its 2 border."),
    nesting(role("full"), "s4", "A pill in a pill is a pill."),
    nesting(role("control"), "s12", "An inset that reaches the radius: `nested` refuses it, and in a `const` the build stops."),
  );
}

/** A button inside a card keeps the control radius: it isn't filling the card. */
function placedInside(): HTMLElement {
  return el(
    "div",
    "f-card-box",
    stack("s16", "start", text("Erase the disk?", "subheading", "on_surface"), row("s8", "start", button({ label: "Cancel", kind: "Tonal", size: "Md" }), button({ label: "Erase", kind: "Danger", size: "Md" }))),
  );
}

/** The focus tube at its control's corner, its glow concentric with it. */
function focusCorners(): HTMLElement {
  return row("s32", "start", button({ label: "Continue", kind: "Filled", size: "Md", pinned: "focused" }), button({ label: "Save log", kind: "Tonal", size: "Md", pinned: "focused" }));
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
  summary: "Three corner radii by what a shape is, and full for pills; every other corner is derived by nesting, never picked.",
  live: () =>
    el(
      "div",
      "f-well",
      ...["Wi-Fi", "Bluetooth", "Sound"].map((name, i) => {
        const rowBox = el("div", "f-well-row", text(name, "body_strong", i === 0 ? "on_secondary_container" : "on_surface"));
        rowBox.style.setProperty("--corner", nested(role("surface"), "s12").css);
        rowBox.toggleAttribute("data-selected", i === 0);
        return rowBox;
      }),
    ),
  description: [
    "Every radius says what the shape is: small for marks inside controls, control for everything you click or type in, surface for everything that holds content, and full for pills and circles. A component takes the one its role names, so the same thing has the same corner everywhere, and a corner that differs says it is a different thing.",
    "Any other radius comes out of nesting: a shape that fills another less an inset takes the outer radius less the inset, so both corners share a centre. 8, 16 and 20 are never chosen; they appear only where nesting makes them. A control placed in content (a button in a card) isn't filling it, so it keeps its own role.",
    "The live list is a well, a surface, with its rows 12 in: they take 24 less 12, the control radius, so a row is concentric with the well and a control at once.",
  ],
  tokens: [
    {
      name: "The roles",
      described: "Each role's radius, its corner on a plain shape, and the role in use: a checkbox's box, a button, a card, a switch's track.",
      sample: roles,
    },
  ],
  rules: [
    {
      name: "Nesting is derived",
      described: "Each shape fills the one round it less a spacing step; its corner is the outer's less the step, and the dot is the centre both corners share. An inset that reaches the radius leaves nothing to round, and the nesting is refused.",
      sample: worked,
    },
    {
      name: "A control in content keeps its role",
      described: "A dialog's buttons sit inside its surface without filling it, so they keep the control radius; nesting is only for a shape that fills another less an inset.",
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
      described: "A well's rows 12 in, drawn at each scale: the corners stay concentric, on whole device pixels.",
      sample: () => el("div", "f-scroll", el("div", "f-row", ...SCALES.map((scale) => atScale(scale, nesting(role("surface"), "s12", ""))))),
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
