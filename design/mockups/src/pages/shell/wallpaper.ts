// The wallpaper picker (`config/quickshell/modules/wallpaper/WallpaperPicker.qml` and
// `WallpaperSettings.qml`): a sheet rising out of the frame's bottom edge, a tab per folder over
// its thumbnails, the settings as the last tab. The keyboard's thumbnail previews on the desktop.

import { lights, powerMoves, press } from "../../site/moves.ts";
import { type Move, nonEmpty, page } from "../../site/page.ts";
import { ALL } from "../../site/settings.ts";
import { hung, live as liven, NARROW_WIDTH } from "./screen-parts.ts";
import { FOLDERS, landIn, type PickerOptions, picker } from "./wallpaper-parts.ts";

type Shown = "rest" | "lit" | "pressed" | "selected" | "loading" | "empty" | "error";

/** The picker open on the shots' screen, drawn still. */
function shot(options: PickerOptions = {}, width?: number, reveal?: number): HTMLElement {
  return hung(picker(options), { edge: "bottom", ...(width !== undefined ? { width } : {}), ...(reveal !== undefined ? { reveal } : {}) }).screen.element;
}

/** The stage: the picker live, previewing on the desktop; Esc shuts it and a click opens it again. */
function live(): HTMLElement {
  let set: (open: boolean) => void = () => {};
  let picture: HTMLImageElement | null = null;
  const content = picker({
    live: {
      preview: (source) => {
        if (picture) picture.src = source;
      },
      close: () => set(false),
    },
  });
  const made = hung(content, { edge: "bottom" });
  picture = made.screen.element.querySelector<HTMLImageElement>(".k-screen-wallpaper");
  const applied = picture?.src ?? "";
  set = liven(made.sheet, true);
  made.screen.element.addEventListener("click", (event) => {
    if (made.sheet.parts[0]!.hasAttribute("data-shown") || content.contains(event.target as Node)) return;
    if (picture) picture.src = applied;
    set(true);
    landIn(content);
  });
  if (!ALL) landIn(content);
  return made.screen.element;
}

const SHEET = "the sheet";

const OPENING: Move = {
  name: "Opening",
  what: "Super+Ctrl+Space, the control center's Wallpaper or the Command menu: the sheet rises.",
  tracks: [{ part: SHEET, on: "the picker", target: ".k-sheet-part", property: "--reveal", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }],
  reduced: [{ part: SHEET, on: "the picker", target: ".k-sheet-part", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
  back: "cuts",
  sample: () => shot(),
};

const CLOSING: Move = {
  name: "Closing",
  what: "Esc, Enter's apply, or a press outside: the sheet sinks back into the edge.",
  tracks: [{ part: SHEET, on: "the picker", target: ".k-sheet-part", property: "--reveal", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" }],
  reduced: [{ part: SHEET, on: "the picker", target: ".k-sheet-part", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" }],
  back: "cuts",
  sample: () => shot(),
};

const RESIZING: Move = {
  name: "Another folder",
  what: "Tab to a folder of two: the sheet narrows to its thumbnails over `medium`.",
  tracks: [{ part: "the span", on: "the picker", target: ".k-sheet-part", property: "--sheet-span", from: { length: "size-panel" }, to: { length: "size-dialog" }, duration: "medium", curve: "standard_decel" }],
  reduced: [{ part: "the span", on: "the picker", target: ".k-sheet-part", property: "--sheet-span", from: { length: "size-panel" }, to: { length: "size-dialog" }, duration: "instant", curve: "standard_decel" }],
  back: "plays",
  sample: () => shot({ tab: 2 }),
};

const ROWS = [{ name: "Picker" }] as const;

function cell(state: Shown): HTMLElement {
  switch (state) {
    case "rest":
      return shot();
    case "lit":
      return shot({ at: 2, pinned: "lit" });
    case "pressed":
      return shot({ at: 2, pinned: "pressed" });
    case "selected":
      return shot({ tab: FOLDERS.length });
    case "loading":
      return shot({ folders: [{ name: "Shinjuku", pictures: FOLDERS[0]!.pictures.map((each, index) => (index > 2 ? { ...each, arrival: "loading" as const } : each)) }, ...FOLDERS.slice(1)] });
    case "empty":
      return shot({ folders: [{ name: "Mine", pictures: [] }, ...FOLDERS] });
    case "error":
      return shot({ folders: [{ name: "Shinjuku", pictures: FOLDERS[0]!.pictures.map((each, index) => (index === 4 ? { ...each, arrival: "broken" as const } : each)) }, ...FOLDERS.slice(1)] });
  }
}

const MANY = [...FOLDERS[0]!.pictures, ...FOLDERS[1]!.pictures];

export default page<Shown>({
  level: "shell",
  slug: "wallpaper",
  title: "Wallpaper picker",
  lede: "A bottom-edge sheet: a tab per folder over its thumbnails, the settings last; the keyboard's one previews.",
  code: { qml: "WallpaperPicker, WallpaperSettings" },
  from: ["panel-page", "tabs", "picture-choice", "keycap", "empty-state"],
  stage: { live, hint: "arrows preview, Enter applies, Tab folders, S settings", daylight: true },
  variants: [
    { name: "A folder of five", for: "four a row, the rest under", sample: () => shot() },
    { name: "A folder of two", for: "two a row: a narrower sheet", sample: () => shot({ tab: 2 }) },
    { name: "Settings", for: "motion's three choices; ← → change", sample: () => shot({ tab: FOLDERS.length }) },
  ],
  sizes: [
    { name: "The shots' screen", for: "1600 by 1000", sample: () => shot() },
    { name: "A narrow screen", for: "never wider than between the frame's corners", sample: () => shot({}, NARROW_WIDTH) },
  ],
  states: {
    shown: ["rest", "lit", "pressed", "selected", "loading", "empty", "error"],
    absent: { disabled: "open or gone; nothing in it disables" },
    rows: nonEmpty(ROWS.map(({ name }) => ({ name, cell }))),
    looks: {
      rest: "the applied one ringed and marked Current",
      lit: "the keyboard's one: the cold tube",
      pressed: "the cold tube white-hot",
      selected: "the settings tab: one row, motion's choices",
      loading: "a placeholder until the picture fades in",
      empty: "an `EmptyState` saying where to drop them",
      error: "a picture it can't read: the placeholder stays",
    },
  },
  focus: {
    tubes: [
      { name: "The keyboard's thumbnail", tone: "cold", filled: false, content: false },
      { name: "The applied one", tone: "accent", filled: false, content: false },
    ],
    looks: { lit: "the cold tube inside the edge, its glow outside", signsOff: "the tube as glass, no glow" },
    signsOff: () => shot({ at: 2, pinned: "signs-off" }),
  },
  content: {
    text: [
      { name: "A folder's name", words: { short: "Shinjuku", long: "Shinjuku after the last train" }, sample: (words: string) => shot({ folderName: words }) },
    ],
    missingPicture: [{ name: "Unreadable", for: "the placeholder holds the box", sample: () => cell("error") }],
    counts: [
      { name: "None", for: "an `EmptyState`, two thumbnails wide", sample: () => cell("empty") },
      { name: "One", for: "two columns still", sample: () => shot({ folders: [{ name: "Japanese", pictures: FOLDERS[2]!.pictures.slice(0, 1) }] }) },
      { name: "Many", for: "two rows show; a row scrolls at a time", sample: () => shot({ folders: [{ name: "Everything", pictures: MANY }] }) },
    ],
  },
  motion: {
    arrives: [OPENING],
    leaves: [CLOSING],
    between: [
      RESIZING,
      { name: "Lit", what: "The keyboard moves on: the cold tube lights over `short`.", tracks: lights(".k-picture-tile", "a thumbnail", 0, 1), reduced: "same", back: "cuts", sample: () => shot({ at: 2 }) },
      { name: "Pressed", what: "Over `micro` the tube runs white-hot.", tracks: press(".k-picture-tile", "a thumbnail"), reduced: "same", back: "plays", sample: () => shot({ at: 2, pinned: "lit" }) },
      ...powerMoves(() => shot({ at: 2, pinned: "lit" })),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Super+Ctrl+Space"], does: "opens or closes it" },
      { keys: ["h", "j", "k", "l", "←", "↓", "↑", "→"], does: "the next thumbnail; it previews on the desktop" },
      { keys: ["Enter"], does: "applies it and recolours everything" },
      { keys: ["Tab", "Shift+Tab"], does: "the next or the last folder, round to Settings" },
      { keys: ["S"], does: "the settings" },
      { keys: ["←", "→", "Space"], does: "in the settings: motion's choice" },
      { keys: ["Esc"], does: "from the settings, back; else closes" },
    ],
    order: "Tab switches folders; the thumbnails are one stop.",
    lands: "Opening, on the applied thumbnail; a folder, on its applied one or its first.",
  },
  measures: {
    samples: nonEmpty([
      { name: "A folder of five", for: "the padding, the header, the grid", sample: () => shot() },
      { name: "Settings", for: "the row and its choices", sample: () => shot({ tab: FOLDERS.length }) },
    ]),
    measures: [
      { part: "Top and foot", token: "space-s24" },
      { part: "Sides, the container's padding", token: "container-2-padding" },
      { part: "Header to the thumbnails", token: "space-s16" },
      { part: "Tabs to the key hints", token: "space-s16" },
      { part: "Thumbnail", token: "size-thumbnail_width" },
      { part: "Thumbnail's height", token: "size-thumbnail_height" },
      { part: "Between thumbnails", token: "space-s16" },
      { part: "Settings row's padding", token: "space-s12" },
      { part: "Corners, fillets", token: "radius-surface" },
    ],
    reflow: { note: "Never wider than between the frame's corners; the padding steps down.", sample: () => shot({}, NARROW_WIDTH) },
  },
});
