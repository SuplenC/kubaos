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
import { doesntApply, type Easing, type Move, nonEmpty, page, type Track } from "../../site/page.ts";
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
function alone(options: Partial<Parameters<typeof canvas>[0]> = {}, width = "var(--size-panel)", height = "calc(var(--size-panel) * 5 / 8)"): HTMLElement {
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
 * A canvas showing `shown`, its objects able to glide (`--fit`) from where `from` fits them to
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
    ? { part: "the objects", on: "every object", target, property: "--fit", from: 0, to: 1, duration: "instant", curve: "linear" }
    : { part: "the objects", on: "every object", target, property: "--fit", from: 0, to: 1, duration: "medium", curve: "standard_decel" };
}

/** A screen fading in or out over `medium`. */
function fading(on: string, target: string, from: 0 | 1, to: 0 | 1, reduced: boolean): Track<Easing> {
  return { part: "the screen", on, target, property: "opacity", from, to, duration: reduced ? "short" : "medium", curve: "standard" };
}

const MOVES: Move[] = [
  {
    name: "Picking another object",
    what: "← or → (h, l), or a press: the tube leaves the object it was on and lights on the new one, each over `short` on `standard` (`--lit`), as focus moving between controls does. The settings under the canvas follow at once.",
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
    what: "Shift+↓ moves the side screen to its next snap, and a dragged object is let go: the arrangement fits the canvas again and every object glides to its new box over `medium` on `standard_decel` (`--fit`, 0 to 1, each of its edges from its old place to its new, whole device pixels at the end). While an object is dragged nothing eases: it follows the pointer every frame, its guides on and off at once. A resize fits it again every frame, with no glide. With reduced motion the boxes are there at once.",
    tracks: [gliding(".k-cv-screen", false)],
    reduced: [gliding(".k-cv-screen", true)],
    back: "cuts",
    sample: () => fitting(snapped(), screens(), snapped(), 2),
  },
  {
    name: "A screen plugged in",
    what: "The hardware changes while the page shows: a screen plugged in fades in over `medium` on `standard` at its box in the new fit, as the others glide to theirs over `medium` on `standard_decel` (`--fit`). With reduced motion it fades in over `short` and the boxes are there at once.",
    tracks: [fading("the screen plugged in", '.k-cv-screen[data-n="4"]', 0, 1, false), gliding('.k-cv-screen:not([data-n="4"])', false)],
    reduced: [fading("the screen plugged in", '.k-cv-screen[data-n="4"]', 0, 1, true), gliding('.k-cv-screen:not([data-n="4"])', true)],
    back: "cuts",
    sample: () => fitting([...screens(), TABLET], screens(), [...screens(), TABLET]),
  },
  {
    name: "A screen unplugged",
    what: "A screen unplugged, or the lid closed on the laptop's: it fades out over `medium` on `standard` where it was, as the others glide to their boxes in the new fit (`--fit`). If it was selected, the selection goes to the first screen left. With reduced motion it fades out over `short` and the boxes are there at once.",
    tracks: [fading("the screen unplugged", '.k-cv-screen[data-n="3"]', 1, 0, false), gliding('.k-cv-screen:not([data-n="3"])', false)],
    reduced: [fading("the screen unplugged", '.k-cv-screen[data-n="3"]', 1, 0, true), gliding('.k-cv-screen:not([data-n="3"])', true)],
    back: "cuts",
    sample: () => fitting(screens(), screens(), screens().slice(0, 2)),
  },
  {
    name: "The canvas takes the keyboard",
    what: "Tab onto the canvas: its tube and the glow outside it fade in over `short` on `standard` (`--lit`), as a list's well does; the selected object keeps its own.",
    tracks: lights(".k-cv > .k-ring", "the canvas", 0, 1),
    reduced: "same",
    back: "plays",
    sample: () => alone(),
  },
  {
    name: "An object under the pointer",
    what: "The pointer comes onto an object that isn't selected: it lights over `short` on `standard` (`--lit`), its tube inside its edge; a press selects it and starts a drag.",
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
  summary: "A canvas of objects drawn to scale inside a SidebarPage page, with the selected one's settings under it or beside it: Settings › Displays' screens.",
  live,
  description: [
    "Rust `template::canvas_page`; iced only, inside a SidebarPage page. Drag a screen: its edges and centre snap to its neighbours', a `primary` guide showing while a snap holds, and its numbers under the canvas follow it live. Drag the window's corner: the canvas grows with it and the screens fit again every frame. Plug a tablet in and out under the window.",
    "The canvas is a well in `surface_container_lowest` at the content radius, its sunken hairline round it and a faint dot, the soft outline one device pixel square, on every `space.s16` of its grid from its corner. Its toolbar is its top band, `space.s12` in: the current desk as a chip, then the canvas's actions at the far end, Text sm buttons (Identify, Mirror). Its key hints are its bottom band, `space.s12` in.",
    "The objects are screens drawn to scale in logical pixels, fitted together into the room between the bands less `space.s24` all round, centred, every edge rounded to a whole device pixel so two screens that touch share their pixel. Each is `surface_container_high` at the control radius with its sunken hairline, its number in `headline` and its size in `mono` under it; one too small for its size shows its number alone. The selected one burns the neon tube inside its edge; one under the pointer lights as a control does.",
    "Dragging tracks the pointer every frame: the object's left, centre and right, and its top, middle and bottom, snap to any neighbour's within `space.s8` on the canvas, the nearest winning on each axis, with a `primary` hairline across the stage where the snap holds. Nothing eases while it moves. Letting go is when the change applies, and the arrangement fits the canvas again, every object gliding to its new box.",
    "On a page under `size.canvas_wide` the canvas spans the page at `aspect.canvas` of its width tall (42%), capped so the heading of the settings under it stays in view at the window's foot; the settings follow it down the page. From `size.canvas_wide` the settings go into a column `size.canvas_settings` wide beside the canvas, scrolling on their own, and the canvas takes all the height left above the page's foot. Either way the selected object's settings are ordinary sub-sections of the page: a heading of its name, a line of its numbers, then wells of rows, following the selection; the sidebar's sub-section takes its name.",
    "The hardware is live: a screen plugged in fades in at its place as the others glide to the new fit; one unplugged, or the laptop's with its lid closed, fades out. The template draws what the app tells it; applying a change and its revert countdown are the app's.",
  ],
  variants: [
    { name: "Under 1000 wide", for: "The settings under the canvas, which is 42% of the page's width tall: a window up to 1263 wide.", sample: () => still(displays(), "var(--size-app_width)", "var(--size-app_height)") },
    { name: "From 1000 wide", for: "The settings in a column beside the canvas, which takes all the height: a laptop's full screen and up.", sample: wide },
    { name: "Dragging, a snap held", for: "The side screen held where its top meets the big screen's middle: the guide across the stage.", sample: () => alone({ selected: 2, held: { index: 2, x: 2580, y: 760 } }, "calc(var(--size-panel) * 3 / 2)", "var(--size-panel)") },
  ],
  sizes: doesntApply("it has no sizes of its own: the canvas takes its page's width, its height the share or the room the page's width gives it (Measures, Reflow)."),
  states: {
    shown: ["rest", "lit", "pressed", "selected"],
    absent: {
      disabled: "an object that can't be moved (a mirrored screen) is still selected and set; it just doesn't drag, which its settings say.",
      loading: "the hardware is known as the page opens; nothing on the canvas waits.",
      empty: "there is always a screen: the one the window is on.",
      error: "an arrangement that can't apply is the app's, said in its settings with an `ErrorBlock`; the canvas shows what is.",
    },
    rows: [
      {
        name: "An object",
        cell: (state: MatrixState) => objectIn(state === "selected" ? "rest" : state, state === "selected"),
      },
    ],
    described: {
      rest: "`surface_container_high` with its sunken hairline, its number and size in `on_surface` and `on_surface_variant`. By day the same in daylight colours.",
      lit: "Under the pointer: the tube inside its edge with the glow outside at night; by day the tube as glass.",
      pressed: "Pressed to drag: the tube runs as a pressed control's, the glow `focus.pressed_glow` times the lit one, while it follows the pointer.",
      selected: "The selected object keeps its tube lit, by night with its glow, and the settings under the canvas are its own.",
    },
  },
  focus: {
    tubes: [
      { name: "The selected object", tone: "accent", filled: false, content: false },
      { name: "The canvas, with the keyboard on it", tone: "accent", filled: false, content: false },
    ],
    looks: {
      lit: "At night: the selected object's tube inside its edge with the glow outside, cut by the canvas's edge; with the keyboard on the canvas its own tube round the well too.",
      signsOff: "By day, and at night while a power change has the lights out: both tubes are glass, `primary_glass`, with no glow.",
    },
    signsOff: () => objectIn("signs-off"),
  },
  content: {
    text: [{ name: "A screen's name, its heading under the canvas", words: NAME, sample: () => still(displays(), "var(--size-app_width)", "var(--size-app_height)") }],
    missingPicture: doesntApply("an object is drawn, never a picture: its number and size."),
    counts: [
      { name: "One screen", for: "The laptop alone: it fills the canvas less its margin, centred.", sample: () => alone({ screens: [screens()[1]!] }) },
      { name: "Three screens", for: "The desk: the big screen, the laptop under it and a side screen.", sample: () => alone() },
      { name: "A screen too small for its size", for: "A tablet beside two big screens at a panel's width: it shows its number alone.", sample: () => alone({ screens: [...screens(), { ...TABLET, x: 4480, y: 0 }] }, "var(--size-panel)", "calc(var(--size-panel) / 2)") },
    ],
  },
  motion: { arrives: doesntApply("it comes with its page, which SidebarPage's change of section moves."), leaves: doesntApply("it goes with its page."), between: MOVES },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "Into the canvas, as a stop of the page, then its toolbar's buttons, then the settings' wells." },
      { keys: ["←", "→", "h", "l"], does: "Picks the object before or after, left to right, round from either end; the settings follow." },
      { keys: ["Shift+←", "Shift+→", "Shift+↑", "Shift+↓", "H", "J", "K", "L"], does: "Moves the selected object to its next snap that way: the nearest place where one of its edges or its centre meets a neighbour's. The arrangement fits again." },
      { keys: ["Enter"], does: "Into the settings under it, at the first row of the selected object's." },
    ],
    order: "The canvas is the page's first stop after its header, then its toolbar, then the settings.",
    lands: "Tab into the canvas lands on the selected object, the first screen as the page opens.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The canvas", for: "Its bands `space.s12` in, the objects fitted `space.s24` in from the stage's edges.", sample: () => alone() },
      { name: "Displays at the default", for: "A page of 856: the canvas 42% of 808 tall, the settings below.", sample: () => still(displays(), "var(--size-app_width)", "var(--size-app_height)") },
    ]),
    measures: [
      { what: "The canvas's corners.", token: "radius-content" },
      { what: "The dot grid's step.", token: "space-s16" },
      { what: "The bands' padding.", token: "space-s12" },
      { what: "The objects' margin from the stage's edges.", token: "space-s24" },
      { what: "How near an edge snaps, on the canvas.", token: "space-s8" },
      { what: "An object's corners.", token: "radius-control" },
      { what: "The page width the settings go beside the canvas from.", token: "size-canvas_wide" },
      { what: "The settings' column beside the canvas.", token: "size-canvas_settings" },
      { what: "The canvas's height under that width, to its width.", token: "aspect-canvas" },
      { what: "Between the canvas and the settings, as between the page's parts.", token: "space-s32" },
    ],
    reflow: {
      described:
        "Displays in a window of the toolbar's size. At the 720 minimum the page is 456 and the canvas 424 wide, 178 tall; at the 1120 default 808 wide and 339 tall, capped where a short window would push the settings' heading out of view; from a page of 1000 (a 1264 window) the settings are a 400 column beside the canvas, which takes all the height. The objects fit again on every frame of a resize.",
      sample: () => displays(),
    },
  },
});

