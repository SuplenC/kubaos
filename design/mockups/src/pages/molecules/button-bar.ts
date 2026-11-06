// The Button bar page: `docs/design/components/button-bar.md`. The row of buttons that ends a
// surface, holding the rules of their order, kinds and spacing in one place, and stacking where its
// room is too narrow for one line.

import { type ButtonBarOptions, buttonBar } from "../../lib/button-bar.ts";
import { shown, type Words } from "../../lib/content.ts";
import { filling, flow, stack, within } from "../../lib/layout.ts";
import { text } from "../../lib/text.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

/** The room a dialog gives its bar: its width less its padding on both sides. */
const DIALOG = "calc(var(--size-dialog) - 2 * var(--space-s24))";
/** The room a panel's body gives it: the panel's width less the container's padding on both sides. */
const PANEL = "calc(var(--size-panel) - 2 * var(--container-0-padding))";

const ONE: ButtonBarOptions = { dismiss: "Cancel", actions: [{ label: "Authenticate" }] };
const TWO: ButtonBarOptions = {
  dismiss: "Cancel",
  actions: [
    { label: "Shut down", icon: "power_settings_new" },
    { label: "Restart", icon: "restart_alt" },
  ],
};
const ALONE: ButtonBarOptions = { actions: [{ label: "Done" }] };
const DANGER: ButtonBarOptions = { dismiss: "Cancel", actions: [{ label: "Erase and install", icon: "warning" }], tone: "Danger" };
const SMALL: ButtonBarOptions = { dismiss: "Cancel", actions: [{ label: "Connect" }], size: "Sm" };

const ACTION: Words = { short: "Restart", long: "Restart and install the updates" };

/** `options`' bar in a room `width` wide. */
function inRoom(options: ButtonBarOptions, width = DIALOG): HTMLElement {
  return within(width, buttonBar(options).element);
}

/** A dialog's bar that says which button was pressed, or that Esc was. */
function live(): HTMLElement {
  const said = text("Press a button.", "small", "on_surface_variant");
  const bar = buttonBar(TWO);
  const say = (label: string) => {
    said.textContent = `${label} pressed.`;
  };
  bar.dismiss?.addEventListener("click", () => say("Cancel"));
  bar.actions.forEach((each, index) => each.addEventListener("click", () => say(TWO.actions[index]!.label)));
  return within(DIALOG, stack("s24", "start", said, bar.element));
}

export default page({
  level: "molecules",
  slug: "button-bar",
  title: "Button bar",
  lede: "The row of buttons that ends a surface: Cancel at the start, the actions at the end, the one Enter presses last.",
  code: { rust: "button_bar::button_bar", qml: "ButtonBar", unbuilt: "Not built yet" },
  stage: {
    live: () => within("100%", filling(flow("s32", live(), inRoom(DANGER, PANEL)))),
    hint: "click a button, or Tab and Enter",
    daylight: true,
  },
  variants: [
    { name: "Dismiss and one action", for: "Cancel and what the surface asks", sample: () => inRoom(ONE) },
    { name: "Dismiss and two actions", for: "another way out, Tonal, before the Enter one", sample: () => inRoom(TWO) },
    { name: "One action alone", for: "nothing to back out of", sample: () => inRoom(ALONE) },
    { name: "Danger", for: "a Danger button naming what it destroys", sample: () => inRoom(DANGER) },
  ],
  sizes: [
    { name: "md", for: "a dialog's or a page's foot", sample: () => inRoom(ONE) },
    { name: "sm", for: "a form in a panel's well", sample: () => inRoom(SMALL, PANEL) },
  ],
  states: {
    shown: ["rest"],
    absent: {
      lit: "not a control: its buttons light",
      pressed: "a press is its button's",
      disabled: "a disabled action is a disabled button",
      selected: "holds no choice",
      loading: "a slow step shows on its surface",
      empty: "always holds a button",
      error: "an error belongs to what the surface asks",
    },
    rows: [
      { name: "Dismiss and one action", cell: () => inRoom(ONE) },
      { name: "Dismiss and two actions", cell: () => inRoom(TWO) },
      { name: "Danger", cell: () => inRoom(DANGER) },
    ],
    looks: {
      rest: "its buttons at rest, each with its own states",
    },
  },
  focus: doesntApply("no line of its own: each button lights as the Button page says."),
  content: {
    text: [{ name: "The action", words: ACTION, sample: (words: string) => inRoom({ ...TWO, actions: [TWO.actions[0]!, { label: words, icon: "restart_alt" }] }) }],
    missingPicture: doesntApply("a button's icon is optional; without, its label alone."),
    counts: [
      { name: "One", for: "alone, at the end", sample: () => inRoom(ALONE) },
      { name: "Two", for: "Cancel and the action", sample: () => inRoom(ONE) },
      { name: "Three", for: "Cancel and two actions", sample: () => inRoom(TWO) },
      { name: "Three in a small room", for: "stacked, Enter's on top, each full width", sample: () => inRoom({ ...TWO, actions: [TWO.actions[0]!, { label: shown(ACTION), icon: "restart_alt" }] }, PANEL) },
    ],
  },
  motion: {
    arrives: doesntApply("it arrives with its surface."),
    leaves: doesntApply("it leaves with its surface."),
    between: doesntApply("stacking is its layout at a width: it cuts as the room narrows."),
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "through its buttons, in its surface's order" },
      { keys: ["Enter", "Space"], does: "presses the button the keyboard is on" },
    ],
    order: "Dismiss, then the actions; a disabled one is skipped. Stacked, the same order, so bottom up.",
    lands: "Nowhere of its own: its surface says.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Dismiss and two actions", for: "gaps, and the room between Cancel and the actions", sample: () => inRoom(TWO) },
      { name: "Stacked", for: "each button full width, the gaps between", sample: () => inRoom({ ...TWO, actions: [TWO.actions[0]!, { label: ACTION.long, icon: "restart_alt" }] }, PANEL) },
    ]),
    measures: [
      { part: "Between buttons", token: "space-s8" },
      { part: "Text button's hang, sm and md", token: "space-s12" },
      { part: "Button, md", token: "size-control_md" },
      { part: "Button, sm", token: "size-control_sm" },
    ],
    reflow: {
      note: "One line while its buttons fit; stacked, each full width, once they don't.",
      sample: () => within(`min(100%, ${DIALOG})`, buttonBar({ ...TWO, actions: [TWO.actions[0]!, { label: shown(ACTION), icon: "restart_alt" }] }).element),
    },
  },
});
