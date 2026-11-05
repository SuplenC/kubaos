// The Button bar page: `docs/design/components/button-bar.md`. The row of buttons that ends a
// surface, holding the rules of their order, kinds and spacing in one place, and stacking where its
// room is too narrow for one line.

import { type ButtonBarOptions, buttonBar } from "../../lib/button-bar.ts";
import { shown, type Words } from "../../lib/content.ts";
import { stack, within } from "../../lib/layout.ts";
import { text } from "../../lib/text.ts";
import { doesntApply, nonEmpty } from "../../site/page.ts";
import { page } from "../../site/legacy.ts";

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
  summary: "The row of buttons that ends a surface: Cancel at the start, the actions at the end, the one Enter presses last.",
  live,
  description: [
    "Rust `button_bar::button_bar`, QML `ButtonBar`. The row of buttons that ends a surface (a dialog, a form in a well), holding the rules of their order, kinds and spacing in one place, so every surface ends the same way.",
    "The dismiss button (Cancel, a Text button) at the start; the actions at the end, the one Enter presses last: Filled, the screen's one, and any other action before it Tonal (Shut down beside Restart). On a Danger surface that action is a Danger button naming what it does (Erase and install). An action alone sits at the end. Buttons are `space.s8` apart, regular (`md`) unless the surface's controls are small.",
    "It has no padding of its own: it sits in its surface's padding like any other content, as wide as the room its surface gives it, a step of the surface's own spacing under what is above it (`space.s24` in a dialog).",
    "A Text button at its start or end hangs its own padding outside it (`space.s12` at sm and md, `space.s16` at lg), so its label sits on the content's edge like everything above it, and its tube and press fill reach into the surface's padding; every other button keeps its box on the edge. A card's padding and a banner's rail lane have no room for it, so there the box stays on the edge. Stacked, nothing hangs.",
    "Where its buttons don't fit on one line (a narrow room, a long translation), they stack `space.s8` apart, each the bar's full width: the one Enter presses on top, the dismiss button at the bottom. They lay out on one line again as soon as they fit.",
  ],
  variants: [
    { name: "Dismiss and one action", for: "Cancel and what the surface asks for: Authenticate.", sample: () => inRoom(ONE) },
    { name: "Dismiss and two actions", for: "Another way out, Tonal, before the one Enter presses: Shut down, Restart.", sample: () => inRoom(TWO) },
    { name: "One action alone", for: "Nothing to back out of: the action at the end.", sample: () => inRoom(ALONE) },
    { name: "Danger", for: "Before something is destroyed: the action a Danger button naming it.", sample: () => inRoom(DANGER) },
  ],
  sizes: [
    { name: "md", for: "A surface's own actions: a dialog's, a page's foot. As tall as a field.", sample: () => inRoom(ONE) },
    { name: "sm", for: "A surface whose controls are small: a form in a panel's well.", sample: () => inRoom(SMALL, PANEL) },
  ],
  states: {
    shown: ["rest"],
    absent: {
      lit: "the bar isn't a control: its buttons light, each as the Button page says.",
      pressed: "a press is its button's, which shows it.",
      disabled: "an action that can't be taken yet is a disabled button; the bar itself is there or not.",
      selected: "it holds no choice of its own.",
      loading: "what its buttons do is known as it shows; a step that takes a moment shows on its surface.",
      empty: "a bar always holds at least one button.",
      error: "an error belongs to what the surface asks for (a field's Invalid state), never to its buttons.",
    },
    rows: [
      { name: "Dismiss and one action", cell: () => inRoom(ONE) },
      { name: "Dismiss and two actions", cell: () => inRoom(TWO) },
      { name: "Danger", cell: () => inRoom(DANGER) },
    ],
    described: {
      rest: "Its buttons at rest, the same by night and by day; each keeps its own states and looks.",
    },
  },
  focus: doesntApply("the bar has no line of its own: each button lights as the Button page says."),
  content: {
    text: [{ name: "The action", words: ACTION, sample: (words: string) => inRoom({ ...TWO, actions: [TWO.actions[0]!, { label: words, icon: "restart_alt" }] }) }],
    missingPicture: doesntApply("a button's icon is optional; a bar without icons is its labels alone."),
    counts: [
      { name: "One", for: "An action alone, at the end.", sample: () => inRoom(ALONE) },
      { name: "Two", for: "Cancel and the action.", sample: () => inRoom(ONE) },
      { name: "Three", for: "Cancel and two actions.", sample: () => inRoom(TWO) },
      { name: "Three in a small room", for: "A panel's body, too narrow for one line: stacked, Restart on top, Cancel at the bottom, each full width.", sample: () => inRoom({ ...TWO, actions: [TWO.actions[0]!, { label: shown(ACTION), icon: "restart_alt" }] }, PANEL) },
    ],
  },
  motion: {
    arrives: doesntApply("it arrives with its surface and moves with it."),
    leaves: doesntApply("it leaves with its surface."),
    between: doesntApply("stacking is its layout at a width, not a move: it cuts from one line to the stack as the room narrows."),
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "Through its buttons in order, as its surface's Tab goes." },
      { keys: ["Enter", "Space"], does: "Presses the button the keyboard is on." },
    ],
    order: "The dismiss button, then the actions in order; a disabled action is skipped. Stacked, the order stays, so it runs from the bottom up.",
    lands: "Nowhere of its own: its surface says where the keyboard lands, and what Enter and Esc do there.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Dismiss and two actions", for: "The gaps between buttons and the room they leave between Cancel and the actions.", sample: () => inRoom(TWO) },
      { name: "Stacked", for: "Each button the room's full width, the gaps between them.", sample: () => inRoom({ ...TWO, actions: [TWO.actions[0]!, { label: ACTION.long, icon: "restart_alt" }] }, PANEL) },
    ]),
    measures: [
      { what: "Between buttons, side by side and stacked.", token: "space-s8" },
      { what: "How far a Text button at the start or end hangs past the bar's edge, at sm and md: its own padding.", token: "space-s12" },
      { what: "A button's height, regular.", token: "size-control_md" },
      { what: "A button's height, small.", token: "size-control_sm" },
    ],
    reflow: {
      described: "As wide as its room, from a panel's body up to a dialog's: one line while its buttons fit, Cancel at the start and the actions at the end; stacked, each full width, once they don't.",
      sample: () => within(`min(100%, ${DIALOG})`, buttonBar({ ...TWO, actions: [TWO.actions[0]!, { label: shown(ACTION), icon: "restart_alt" }] }).element),
    },
  },
});
