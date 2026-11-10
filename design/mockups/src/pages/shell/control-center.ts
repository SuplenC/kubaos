// The control center as a whole (`config/quickshell/modules/controlcenter/ControlCenter.qml`): a
// sheet on the frame's right edge, from the bar to the frame's foot, holding the header, the quick
// toggles, the capture card and the sliders, then the four tabs and the tab's well in what is left.
// Each part has a page of its own; this one is the panel, its tabs, its motion and its keys.

import { powerMoves } from "../../site/moves.ts";
import { doesntApply, type Easing, type Move, type NonEmpty, nonEmpty, page, type Track } from "../../site/page.ts";
import { ALL } from "../../site/settings.ts";
import { connectForm, controlCenter, DATE, onScreen, type PanelOptions, radioWell, WIFI_FORM, wellOf, wifiOff, wifiRows } from "./control-center-parts.ts";

type Shown = "rest" | "lit" | "selected" | "loading" | "empty" | "error";

const SHORT = "calc(var(--size-app_min_height) + var(--space-s64) * 2)";

function shot(options: PanelOptions = {}, height?: string): HTMLElement {
  return onScreen(controlCenter(options), height);
}

/** The stage's panel: the tabs swap their wells, as the build does. */
function live(): HTMLElement {
  const room = shot();
  if (!ALL) requestAnimationFrame(() => room.querySelector<HTMLElement>(".k-tile")?.focus({ preventScroll: true }));
  return room;
}

const SHEET = "the sheet";
const SCRIM = "the desktop's dim";

const OPENING: Move = {
  name: "Opening",
  what: "Its pill, Super+A or the Command menu: the sheet grows from the right edge, content riding on it.",
  tracks: [
    { part: SHEET, on: "the control center", target: ".k-sheet-part", property: "--reveal", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
    { part: SCRIM, on: "the control center", target: ".k-screen-dim", property: "--panel-dim", from: 0, to: 1, duration: "medium", curve: "standard" },
  ],
  reduced: [
    { part: SHEET, on: "the control center", target: ".k-sheet-part", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" },
    { part: SCRIM, on: "the control center", target: ".k-screen-dim", property: "--panel-dim", from: 0, to: 1, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => shot(),
};

const CLOSING: Move = {
  name: "Closing",
  what: "Esc, a press outside, its pill, or a capture button: the sheet goes back into the edge.",
  tracks: [
    { part: SHEET, on: "the control center", target: ".k-sheet-part", property: "--reveal", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" },
    { part: SCRIM, on: "the control center", target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "medium", curve: "standard" },
  ],
  reduced: [
    { part: SHEET, on: "the control center", target: ".k-sheet-part", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" },
    { part: SCRIM, on: "the control center", target: ".k-screen-dim", property: "--panel-dim", from: 1, to: 0, duration: "short", curve: "standard" },
  ],
  back: "cuts",
  sample: () => shot(),
};

/** A pick along the tabs: the lit stretch's edges, its flash, the two choices' words. */
function picking(reduced: boolean): NonEmpty<Track<Easing>> {
  const edge = (part: string, property: string, travel: "short" | "medium", cue?: "micro"): Track<Easing> =>
    reduced
      ? { part, on: "the lit stretch", target: ".k-tabs-lit", property, from: 0, to: 1, duration: "instant", curve: "linear" }
      : { part, on: "the lit stretch", target: ".k-tabs-lit", property, from: 0, to: 1, duration: travel, ...(cue ? { cue } : {}), curve: "emphasized_decel" };
  return [
    edge("leading edge", "--lead", "short"),
    edge("trailing edge", "--trail", "medium", "micro"),
    { part: "flash", on: "the lit stretch", target: ".k-tabs-lit", property: "--flash", from: 0, to: 1, duration: "micro", curve: "standard" },
    { part: "flash", on: "the lit stretch", target: ".k-tabs-lit", property: "--flash", from: 1, to: 0, duration: reduced ? "short" : "long", cue: "micro", curve: "standard" },
    { part: "words", on: "the tab picked", target: '.k-tab[aria-selected="true"]', property: "--picked", from: 0, to: 1, duration: "short", curve: "standard" },
    { part: "words", on: "the tab it leaves", target: ".k-tab[data-was]", property: "--picked", from: 1, to: 0, duration: "short", curve: "standard" },
  ];
}

const PICK: Move = {
  name: "Another tab",
  what: "A tab, `1`–`4`, or h/l on the tabs: the stretch glides there; the well swaps at once.",
  tracks: picking(false),
  reduced: picking(true),
  back: "cuts",
  sample: () => shot({ tab: 1, tabsFrom: 0 }),
};

const ROWS = [{ name: "Wi-Fi tab" }] as const;

function cell(state: Shown): HTMLElement {
  switch (state) {
    case "rest":
      return shot();
    case "lit":
      return shot({ headerLit: "lit" });
    case "selected":
      return shot({ tab: 3 });
    case "loading":
      return shot({ content: () => radioWell(wifiRows("scanning")) });
    case "empty":
      return shot({ toggles: { wifi: { glyph: "wifi_off", subtitle: "Off", on: false } }, content: () => wellOf(wifiOff()) });
    case "error":
      return shot({ content: () => wellOf(connectForm({ ...WIFI_FORM, value: "hunter22", status: "Wrong password", failed: true, ready: true })) });
  }
}

export default page<Shown>({
  level: "shell",
  slug: "control-center",
  title: "Control center",
  lede: "The right-edge sheet of quick settings: header, toggles, capture, sliders, then four tabs over a well.",
  code: { qml: "ControlCenter" },
  stage: { live, hint: "click a tab; 1–4 jump; Esc closes", daylight: true },
  variants: [
    { name: "Wi-Fi", for: "networks Known and Nearby", sample: () => shot({ tab: 0 }) },
    { name: "Bluetooth", for: "devices, and the unnamed drawer", sample: () => shot({ tab: 1 }) },
    { name: "Output", for: "the default audio output", sample: () => shot({ tab: 2 }) },
    { name: "Keyboard", for: "layouts, their order and accents", sample: () => shot({ tab: 3 }) },
  ],
  sizes: [
    { name: "A 1000-high screen", for: "the shots' screen: the well takes the rest", sample: () => shot() },
    { name: "A short screen", for: "the well shrinks to what is left", sample: () => shot({}, SHORT) },
  ],
  states: {
    shown: ["rest", "lit", "selected", "loading", "empty", "error"],
    absent: {
      pressed: "its parts press on their own pages",
      disabled: "open or gone; its controls disable themselves",
    },
    rows: nonEmpty(ROWS.map(({ name }) => ({ name, cell }))),
    looks: {
      rest: "`surface` at the frame's glass; nothing lit",
      lit: "a header button lit with the keyboard",
      selected: "another tab picked: its well in place",
      loading: "a scan's skeletons under Nearby",
      empty: "the radio off: an `EmptyState` in the well",
      error: "the form keeps its place, the error under the field",
    },
  },
  focus: doesntApply("no line of its own; each control lights on its page, and the sheet's edge cuts no glow."),
  content: {
    text: [{ name: "The date and battery line", words: { short: DATE, long: "Tuesday, 27 October · 64% 3h 12m left, charging slowly on a weak adapter" }, sample: (words: string) => shot({ date: words }) }],
    missingPicture: doesntApply("it draws no picture; every symbol is the service's."),
    counts: [
      { name: "No Proxy profile", for: "the keyboard tile takes a row", sample: () => shot({ toggles: { proxy: false } }) },
      { name: "Proxy detail open", for: "the sections grow; the well gives way", sample: () => shot({ toggles: { detail: {} } }) },
      { name: "No brightness", for: "a desktop: two sliders", sample: () => shot({ sliders: { brightness: false } }) },
    ],
  },
  motion: {
    arrives: [OPENING],
    leaves: [CLOSING],
    between: [PICK, ...powerMoves(() => shot({ headerLit: "lit" }))],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "round the controls, header to the well" },
      { keys: ["1", "2", "3", "4"], does: "straight to that tab, from anywhere" },
      { keys: ["h", "l", "←", "→"], does: "on the tabs: the tab before or after" },
      { keys: ["j", "k", "↑", "↓"], does: "in a well: through its rows" },
      { keys: ["Esc"], does: "a form or the add view back; else closes" },
    ],
    order: "Header's buttons, the tiles, capture, the sliders, the tabs, the well.",
    lands: "Opening, on the Wi-Fi tile, or on an open form's first field.",
  },
  measures: {
    samples: nonEmpty([
      { name: "On the Wi-Fi tab", for: "width, padding, the sections' gaps", sample: () => shot() },
      { name: "On a short screen", for: "the well in what is left", sample: () => shot({}, SHORT) },
    ]),
    measures: [
      { part: "Panel's width", token: "size-panel" },
      { part: "Top, foot, between sections", token: "space-s16" },
      { part: "Sides, the container's padding", token: "container-0-padding" },
      { part: "Header's buttons", token: "size-control_md" },
      { part: "Between header's buttons", token: "space-s8" },
      { part: "Tabs to the well", token: "space-s12" },
      { part: "Tabs' height", token: "size-control_sm" },
      { part: "Corners, fillets", token: "radius-surface" },
      { part: "The bar, the frame's top", token: "size-bar" },
    ],
    reflow: {
      note: "It keeps `size.panel`; the screen's height goes to the well alone.",
      sample: () => shot({}, SHORT),
    },
  },
});
