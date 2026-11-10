// The Frame page: `config/quickshell/modules/frame/Frame.qml`. The screen's frame: the bar along the
// top and `space.s8` down the other edges, its inner corners at the surface radius, one glass with
// every panel, which retracts under a fullscreen window and dims the desktop under an open panel.

import "./frame.css";
import { hang, sheet } from "../../lib/sheet.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { powerMoves } from "../../site/moves.ts";
import { WINDOWS, windowSize } from "../../site/settings.ts";
import { system } from "../templates/panels.ts";
import { barScreen, CLOCK } from "./bar-parts.ts";

type State = "rest" | "open";

const LAPTOP = WINDOWS[2]!;
const SMALL = { width: "var(--size-app_min_width)", height: "var(--size-app_min_height)" };

interface FrameOptions {
  readonly width?: string;
  readonly height?: string;
  readonly fullscreen?: boolean;
  /** How far shown, 0 to 1: a picture of it on its way. */
  readonly revealed?: number;
  readonly open?: boolean;
  readonly live?: boolean;
}

function frame(options: FrameOptions = {}): HTMLElement {
  const shown = barScreen(options.width ?? SMALL.width, options.height ?? SMALL.height, { clock: { countdown: CLOCK.countdown } });
  const element = shown.element;
  element.classList.add("shell-frame");
  element.toggleAttribute("data-fullscreen", options.fullscreen ?? false);
  if (options.revealed !== undefined) element.style.setProperty("--revealed", String(options.revealed));
  if (options.open) {
    element.dataset.open = "";
    hang(shown, sheet({ content: system().element, edge: "right" }));
  }
  if (options.live) {
    element.dataset.live = "";
    element.addEventListener("click", (event) => {
      if ((event.target as Element).closest(".k-bar")) return;
      element.toggleAttribute("data-fullscreen");
    });
  }
  return element;
}

const RETRACTS: Move = {
  name: "Retracting",
  what: "A window goes fullscreen with no panel open: the frame goes to nothing over `medium` on `emphasized_accel`.",
  tracks: [{ part: "the frame", on: "the frame", target: ".shell-frame", property: "--revealed", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" }],
  reduced: "same",
  back: "cuts",
  sample: () => frame(),
};

const RETURNS: Move = {
  name: "Coming back",
  what: "The window leaves fullscreen, or a panel opens: the frame comes back over `long` on `emphasized_decel`.",
  tracks: [{ part: "the frame", on: "the frame", target: ".shell-frame", property: "--revealed", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }],
  reduced: "same",
  back: "cuts",
  sample: () => frame({ fullscreen: true }),
};

const DIMS: Move = {
  name: "The desktop dims",
  what: "A panel opens: `scrim` fades to `opacity.panel_scrim` over `medium` on `standard`.",
  tracks: [{ part: "the dim", on: "the desktop", target: ".k-screen-dim", property: "--panel-dim", from: 0, to: 1, duration: "medium", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: () => frame({ open: true }),
};

export default page<State>({
  level: "shell",
  slug: "frame",
  title: "Frame",
  lede: "The screen's frame: the bar on top, a thin edge round the rest, one glass with every panel.",
  code: { qml: "Frame" },
  stage: { live: () => frame({ width: LAPTOP.width, height: LAPTOP.height, live: true }), hint: "click the desktop: a window goes fullscreen", daylight: true },
  variants: [
    { name: "Shown", for: "the desktop, windows tiled inside it", sample: () => frame() },
    { name: "A panel open", for: "one glass with the frame; the desktop dims", sample: () => frame({ open: true }) },
    { name: "Retracted", for: "a fullscreen window and no panel", sample: () => frame({ fullscreen: true }) },
    { name: "Half way", for: "the reveal at a half", sample: () => frame({ revealed: 0.5 }) },
  ],
  sizes: doesntApply("its edges are tokens at every screen size: the reflow shows them."),
  states: {
    shown: ["rest", "open"],
    absent: {
      lit: "takes no pointer; its bar's pills light",
      pressed: "a click on it does nothing",
      disabled: "always there, or retracted",
      selected: "nothing to pick",
      loading: "drawn with the shell",
      empty: "the bar always has its clock",
      error: "nothing it can fail at",
    },
    rows: nonEmpty([{ name: "The frame", cell: (state: State) => frame({ open: state === "open" }) }]),
    looks: {
      rest: "`surface` at `opacity.frame`, `shadow.panel` once",
      open: "the panel joins the glass; `scrim` at `opacity.panel_scrim`",
    },
  },
  focus: doesntApply("no line of its own; a panel's controls light inside it."),
  content: {
    text: doesntApply("no words of its own; the bar's are the Bar page's."),
    missingPicture: doesntApply("the wallpaper is always there; the frame draws none."),
    counts: doesntApply("one frame per screen."),
  },
  motion: {
    arrives: [RETURNS],
    leaves: [RETRACTS],
    between: [DIMS, ...powerMoves(() => frame())],
  },
  keyboard: {
    keys: [{ keys: ["Esc"], does: "closes the open panel" }],
    order: "The frame takes no Tab; an open panel holds the keyboard on its screen.",
    lands: "A panel closing gives the keyboard back to the window.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Shown", for: "the bar, the sides, the corners", sample: () => frame() },
      { name: "A panel open", for: "the panel against the frame", sample: () => frame({ open: true }) },
    ]),
    measures: [
      { part: "Top, the bar", token: "size-bar" },
      { part: "Sides and foot", token: "space-s8" },
      { part: "Inner corners", token: "radius-surface" },
      { part: "Shadow's drop", token: "shadow-panel-y" },
      { part: "Shadow's blur", token: "shadow-panel-blur" },
    ],
    reflow: {
      note: "The edges keep their tokens at every size; the bar's middle stays centred.",
      sample: () => {
        const size = windowSize();
        return frame({ width: size.width, height: size.height });
      },
    },
  },
});
