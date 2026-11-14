// The CanvasPage page: `docs/design/components/canvas-page.md`. A canvas of objects drawn to
// scale inside a SidebarPage page, with the selected object's settings under it or beside it:
// Settings › Displays' screens. Live in a window that resizes, from the app's minimum to a 1080p
// screen, with screens to plug in and out.

import { button } from "../../lib/button.ts";
import type { Shown } from "../../lib/control.ts";
import { shown, type Words } from "../../lib/content.ts";
import { type Screen, canvas, canvasOf, canvasRegion } from "../../lib/canvas-page.ts";
import { row, stack } from "../../lib/layout.ts";
import type { ListRowSpec } from "../../lib/list.ts";
import { type Section, sidebarPage } from "../../lib/sidebar-page.ts";
import { el } from "../../site/dom.ts";
import { lights, powerMoves } from "../../site/moves.ts";
import { doesntApply, type Easing, type Move, nonEmpty, type Track, page } from "../../site/page.ts";
import { outline } from "./settings.ts";
import { resizable, still } from "./window.ts";

const NAME: Words = { short: "Dell U2723QE", long: "Dell U2723QE, on the arm to the left of the desk" };

/** The desk's screens: a 4K screen on an arm, the laptop's under it, and a side screen. */
function screens(): Screen[] {
  return [
    { id: "dell", name: shown(NAME), number: 1, width: 3840, height: 2160, scale: 1.5, x: 0, y: 0 },
    { id: "laptop", name: "Built-in display", number: 2, width: 2880, height: 1800, scale: 1.5, x: 320, y: 1440 },
    { id: "side", name: "LG 24MP60G", number: 3, width: 1920, height: 1080, scale: 1, x: 2560, y: 360 },
  ];
}

/** A tablet plugged in by its cable: the screen the live window's Plug in adds. */
const TABLET: Screen = { id: "tablet", name: "Tablet over USB-C", number: 4, width: 2000, height: 1200, scale: 2, x: 2560, y: 1440 };

/** The selected screen's settings, its numbers in its rows. */
function settingsOf(screen: Screen): ListRowSpec[][] {
  return [
    [
      { title: "Resolution", subtitle: "The screen's own mode", detail: `${screen.width} × ${screen.height}` },
      { title: "Refresh rate", subtitle: "As fast as it goes", detail: screen.number === 2 ? "120 Hz" : "60 Hz" },
      { title: "Scale", subtitle: "How big things are drawn on it", detail: `${screen.scale}×` },
    ],
    [
      { title: "Rotation", subtitle: "Turn it on its side", detail: "None" },
      { title: "Main screen", subtitle: "Where the bar and new windows go", toggle: screen.number === 1 },
    ],
  ];
}

const NIGHT: Section["subs"][number] = {
  title: "Night light",
  wells: [[{ title: "Night light", subtitle: "Warmer colours after sunset", toggle: true }]],
};

/** Settings on Displays, its region the canvas: the template in its place. */
function displays(options: { readonly selected?: number; readonly keyboard?: boolean } = {}): HTMLElement {
  const sections = outline((host) =>
    canvasRegion(host, { screens: screens(), desk: "Desk at home", settings: settingsOf, after: [NIGHT], ...options }),
  );
  return sidebarPage({ outline: sections, section: 1 });
}

/** The canvas alone in a room `width` × `height`, standing as `options` say. */
function alone(options: Partial<Parameters<typeof canvas>[0]> = {}, width = "calc(var(--size-panel) * 3 / 2)", height = "calc(var(--size-panel) * 15 / 16)"): HTMLElement {
  const drawn = canvas({ screens: screens(), desk: "Desk at home", ...options });
  drawn.element.style.width = width;
  drawn.element.style.height = height;
  return drawn.element;
}

/** The canvas with object `n` (its number) pinned in `state`. */
function objectIn(state: Shown, selected = false): HTMLElement {
  const element = alone(selected ? { selected: 2 } : { selected: 0 });
  requestAnimationFrame(() => {
    const target = element.querySelector<HTMLElement>('.k-cv-screen[data-n="3"]');
    if (target && state !== "rest" && state !== "disabled") target.dataset.pinned = state;
  });
  return element;
}

/**
 * A canvas showing `shown`, its objects able to glide (`--canvas-fit`) from where `from` fits them to
 * where `to` does: each fitted in a hidden twin of the same size, read back once laid out. An
 * object missing from one keeps the other's box.
 */
function fitting(shown: Screen[], from: Screen[], to: Screen[], selected = 0): HTMLElement {
  const room = el("span", "t-fit");
  const make = (list: Screen[]) => alone({ screens: list, selected });
  const visible = make(shown);
  const twins = [make(from), make(to)];
  for (const twin of twins) twin.classList.add("t-twin");
  room.append(visible, ...twins);
  requestAnimationFrame(() =>
    requestAnimationFrame(() => {
      const boxes = (twin: HTMLElement) => new Map([...twin.querySelectorAll<HTMLElement>(".k-cv-screen")].map((each) => [each.dataset.n, each.style]));
      const [before, after] = twins.map(boxes) as [Map<string | undefined, CSSStyleDeclaration>, Map<string | undefined, CSSStyleDeclaration>];
      for (const each of visible.querySelectorAll<HTMLElement>(".k-cv-screen")) {
        const a = before.get(each.dataset.n) ?? after.get(each.dataset.n);
        const b = after.get(each.dataset.n) ?? a;
        if (!a || !b) continue;
        for (const side of ["left", "top", "width", "height"] as const) {
          each.style.setProperty(`--from-${side}`, a[side]);
          each.style.setProperty(`--to-${side}`, b[side]);
        }
      }
      visible.dataset.fitting = "";
      for (const twin of twins) twin.remove();
    }),
  );
  return room;
}

/** The arrangement with the side screen moved down to its next snap: its top at the big screen's middle. */
function snapped(): Screen[] {
  return screens().map((each) => (each.id === "side" ? { ...each, y: 720 } : each));
}

/** The glide of every object to its new box as the arrangement fits again. */
function gliding(target: string, reduced: boolean): Track<Easing> {
  return reduced
    ? { part: "the objects", on: "every object", target, property: "--canvas-fit", from: 0, to: 1, duration: "instant", curve: "linear" }
    : { part: "the objects", on: "every object", target, property: "--canvas-fit", from: 0, to: 1, duration: "medium", curve: "standard_decel" };
}

/** A screen fading in or out over `medium`. */
function fading(on: string, target: string, from: 0 | 1, to: 0 | 1, reduced: boolean): Track<Easing> {
  return { part: "the screen", on, target, property: "opacity", from, to, duration: reduced ? "short" : "medium", curve: "standard" };
}

const MOVES: Move[] = [
  {
    name: "Picking another object",
    what: "← → or a press: the tube leaves one object and lights on the next; the settings follow.",
    tracks: [
      { part: "the object left", on: "the first screen", target: '.k-cv-screen[data-n="1"]', property: "--lit", from: 1, to: 0, duration: "short", curve: "standard" },
      { part: "the object picked", on: "the third screen", target: '.k-cv-screen[data-n="3"]', property: "--lit", from: 0, to: 1, duration: "short", curve: "standard" },
    ],
    reduced: "same",
    back: "plays",
    sample: () => alone({ selected: 2 }),
  },
  {
    name: "Fitted again",
    what: "A snap key, or a drag let go: every object glides to its new box; a resize never glides.",
    tracks: [gliding(".k-cv-screen", false)],
    reduced: [gliding(".k-cv-screen", true)],
    back: "cuts",
    sample: () => fitting(snapped(), screens(), snapped(), 2),
  },
  {
    name: "A screen plugged in",
    what: "It fades in at its new box as the others glide to theirs.",
    tracks: [fading("the screen plugged in", '.k-cv-screen[data-n="4"]', 0, 1, false), gliding('.k-cv-screen:not([data-n="4"])', false)],
    reduced: [fading("the screen plugged in", '.k-cv-screen[data-n="4"]', 0, 1, true), gliding('.k-cv-screen:not([data-n="4"])', true)],
    back: "cuts",
    sample: () => fitting([...screens(), TABLET], screens(), [...screens(), TABLET]),
  },
  {
    name: "A screen unplugged",
    what: "It fades out where it was as the others glide; a selection on it goes to the first left.",
    tracks: [fading("the screen unplugged", '.k-cv-screen[data-n="3"]', 1, 0, false), gliding('.k-cv-screen:not([data-n="3"])', false)],
    reduced: [fading("the screen unplugged", '.k-cv-screen[data-n="3"]', 1, 0, true), gliding('.k-cv-screen:not([data-n="3"])', true)],
    back: "cuts",
    sample: () => fitting(screens(), screens(), screens().slice(0, 2)),
  },
  {
    name: "The canvas takes the keyboard",
    what: "Tab onto the canvas: its tube and glow fade in; the selected object keeps its own.",
    tracks: lights(".k-cv > .k-ring", "the canvas", 0, 1),
    reduced: "same",
    back: "plays",
    sample: () => alone(),
  },
  {
    name: "An object under the pointer",
    what: "It lights with its tube inside its edge; a press selects it and starts a drag.",
    tracks: lights('.k-cv-screen[data-n="3"]', "the third screen", 0, 1),
    reduced: "same",
    back: "plays",
    sample: () => alone(),
  },
  ...powerMoves(() => alone({ keyboard: true })),
];

/** The live window, and the hardware changing under it: a tablet plugged in and out. */
function live(): HTMLElement {
  const window = displays();
  const plug = button({ label: "Plug in a tablet", kind: "Tonal", size: "Sm" });
  const unplug = button({ label: "Unplug the side screen", kind: "Text", size: "Sm" });
  const region = () => canvasOf(window.querySelector(".k-cvp"));
  plug.addEventListener("click", () => {
    const drawn = region();
    if (!drawn) return;
    if (drawn.screens().some((each) => each.id === TABLET.id)) drawn.unplug(TABLET.id);
    else drawn.plug(TABLET);
  });
  unplug.addEventListener("click", () => {
    const drawn = region();
    if (!drawn) return;
    if (drawn.screens().some((each) => each.id === "side")) drawn.unplug("side");
    else drawn.plug(screens()[2]!);
  });
  return stack("s16", "start", resizable(window), row("s8", "centre", plug, unplug));
}

/** Displays at a laptop's 1280 × 800: a page of 1016, the settings beside the canvas. */
function wide(): HTMLElement {
  return el("div", "t-room", still(displays(), "calc(1280 * var(--px))", "calc(800 * var(--px))"));
}

type MatrixState = "rest" | "lit" | "pressed" | "selected";

export default page({
  level: "templates",
  slug: "canvas-page",
  title: "CanvasPage",
  lede: "A canvas of objects drawn to scale, with the selected one's settings: Settings › Displays' screens.",
  code: { unbuilt: "Rust template::canvas_page: not built yet" },
  stage: {
    live,
    hint: "drag a screen or the window's corner; plug in below",
    daylight: true,
  },
  variants: [
    { name: "Under 1000 wide", for: "the settings under the canvas, `aspect.canvas` tall", sample: () => still(displays(), "var(--size-app_width)", "var(--size-app_height)") },
    { name: "From 1000 wide", for: "the settings in a column; the canvas takes the height", sample: wide },
    { name: "Dragging, a snap held", for: "the `primary` guide across the stage", sample: () => alone({ selected: 2, held: { index: 2, x: 2580, y: 760 } }, "calc(var(--size-panel) * 3 / 2)", "var(--size-panel)") },
  ],
  sizes: doesntApply("none of its own: its page's width sets the canvas's height."),
  states: {
    shown: ["rest", "lit", "pressed", "selected"],
    absent: {
      disabled: "a mirrored screen is still selected; it just doesn't drag",
      loading: "the hardware is known as the page opens",
      empty: "there is always the screen the window is on",
      error: "the app says it, an `ErrorBlock` in the settings",
    },
    rows: [
      {
        name: "An object",
        cell: (state: MatrixState) => objectIn(state === "selected" ? "rest" : state, state === "selected"),
      },
    ],
    looks: {
      rest: "`surface_container_high`, a sunken hairline, its number and size",
      lit: "the tube inside its edge, glow at night",
      pressed: "the tube runs hot, glow × `focus.pressed_glow`, following the pointer",
      selected: "the tube stays lit; the settings are its own",
    },
  },
  focus: {
    tubes: [
      { name: "The selected object", tone: "accent", filled: false, content: false },
      { name: "The canvas, with the keyboard", tone: "accent", filled: false, content: false },
    ],
    looks: {
      lit: "the object's tube inside its edge, its glow cut by the canvas",
      signsOff: "both tubes as `primary_glass`, no glow",
    },
    signsOff: () => objectIn("signs-off"),
  },
  content: {
    text: [{ name: "A screen's name, under the canvas", words: NAME, sample: () => still(displays(), "var(--size-app_width)", "var(--size-app_height)") }],
    missingPicture: doesntApply("an object is drawn, never a picture."),
    counts: [
      { name: "One screen", for: "it fills the canvas less its margin", sample: () => alone({ screens: [screens()[1]!] }) },
      { name: "Three screens", for: "the desk: big screen, laptop under it, side screen", sample: () => alone() },
      { name: "A screen too small", for: "it shows its number alone", sample: () => alone({ screens: [...screens(), { ...TABLET, x: 4480, y: 0 }] }, "var(--size-panel)", "calc(var(--size-panel) / 2)") },
    ],
  },
  motion: { arrives: doesntApply("it comes with its page, in SidebarPage's change of section."), leaves: doesntApply("it goes with its page."), between: MOVES },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "into the canvas, its toolbar, then the settings' wells" },
      { keys: ["←", "→", "h", "l"], does: "the object before or after, round; the settings follow" },
      { keys: ["Shift+←", "Shift+→", "Shift+↑", "Shift+↓", "H", "J", "K", "L"], does: "moves the object to its next snap that way" },
      { keys: ["Enter"], does: "into the settings, at the object's first row" },
    ],
    order: "The page's first stop after its header, then its toolbar, then the settings.",
    lands: "On the selected object: the first screen as the page opens.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The canvas", for: "its bands and the objects' margin", sample: () => alone() },
      { name: "Displays at the default", for: "the canvas over its settings", sample: () => still(displays(), "var(--size-app_width)", "var(--size-app_height)") },
    ]),
    measures: [
      { part: "Canvas's corners", token: "radius-content" },
      { part: "Dot grid's step", token: "space-s16" },
      { part: "Bands' padding", token: "space-s12" },
      { part: "Objects' margin from the stage", token: "space-s24" },
      { part: "Snap's reach", token: "space-s8" },
      { part: "Object's corners", token: "radius-control" },
      { part: "Settings go beside from", token: "size-canvas_wide" },
      { part: "Settings' column", token: "size-canvas_settings" },
      { part: "Canvas's height to width", token: "aspect-canvas" },
      { part: "Canvas → settings", token: "space-s32" },
    ],
    reflow: {
      note: "The objects fit again every frame; from a page of `size.canvas_wide` the settings go beside the canvas.",
      sample: () => displays(),
    },
  },
});
