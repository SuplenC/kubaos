// The OSD (`config/quickshell/modules/osd/Osd.qml`): the volume or the brightness as a strip rising
// out of the bottom edge of the focused screen while the keys change it: its symbol, a progress
// bar and the level. It takes no keyboard and dims nothing.

import { icon } from "../../lib/icon.ts";
import { fill, filling, inset, row } from "../../lib/layout.ts";
import { progressBar, setProgress } from "../../lib/progress.ts";
import { advance, text } from "../../lib/text.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { hung, live as liven } from "./screen-parts.ts";

export type Kind = "volume" | "brightness";

export interface Level {
  readonly kind: Kind;
  /** 0 to 1. */
  readonly value: number;
  readonly muted?: boolean;
}

/** The symbol for a level, as the services pick it. */
function symbolOf(level: Level): string {
  const value = level.muted ? 0 : level.value;
  if (level.kind === "brightness") return value < 0.34 ? "brightness_low" : value < 0.67 ? "brightness_medium" : "brightness_high";
  return level.muted || value <= 0 ? "volume_off" : value < 0.34 ? "volume_mute" : value < 0.67 ? "volume_down" : "volume_up";
}

/** The level in `body_strong`, at the end of a room as wide as "100", so the bar keeps one length. */
function figure(level: Level): HTMLElement {
  const shown = Math.round((level.muted ? 0 : level.value) * 100);
  const room = row(0, "centre", fill(), text(String(shown), "body_strong"));
  room.style.width = `calc(${advance("body_strong", "100")} * var(--px))`;
  return room;
}

/** The strip: the symbol in `primary`, the bar across the rest, the level; `space.s16` inside, `size.rail` wide. */
export function osd(level: Level): HTMLElement {
  const bar = filling(progressBar({ fraction: level.muted ? 0 : level.value }));
  const strip = inset(row("s12", "centre", icon(symbolOf(level), "var(--icon-s24)", "var(--color-primary)"), bar, figure(level)), "s16", "s16");
  strip.style.width = "var(--size-rail)";
  return strip;
}

function shot(level: Level, reveal?: number): HTMLElement {
  return hung(osd(level), { edge: "bottom", corner: true, ...(reveal !== undefined ? { reveal } : {}) }).screen.element;
}

const VOLUME: Level = { kind: "volume", value: 0.62 };
const BRIGHTNESS: Level = { kind: "brightness", value: 0.8 };

/** The stage: ↑ and ↓ turn the volume, b the brightness, m mutes; the strip follows. */
function live(): HTMLElement {
  let level: Level = VOLUME;
  let content = osd(level);
  const made = hung(content, { edge: "bottom", corner: true });
  const room = made.screen.element;
  const set = liven(made.sheet, true);
  room.tabIndex = 0;
  const redraw = (next: Level) => {
    const kindChanged = next.kind !== level.kind || next.muted !== level.muted;
    level = next;
    if (kindChanged) {
      const fresh = osd(level);
      content.replaceWith(fresh);
      content = fresh;
    } else {
      setProgress(content.querySelector<HTMLElement>(".k-progress")!, level.value);
      content.lastElementChild!.replaceWith(figure(level));
    }
    set(true);
  };
  room.addEventListener("keydown", (event) => {
    const step = event.key === "ArrowUp" ? 0.05 : event.key === "ArrowDown" ? -0.05 : 0;
    if (step !== 0) redraw({ ...level, muted: false, value: Math.min(1, Math.max(0, Math.round((level.value + step) * 20) / 20)) });
    else if (event.key === "b") redraw(level.kind === "brightness" ? VOLUME : BRIGHTNESS);
    else if (event.key === "m") redraw({ ...VOLUME, value: level.kind === "volume" ? level.value : VOLUME.value, muted: !(level.kind === "volume" && level.muted) });
    else if (event.key === "Escape") set(false);
    else return;
    event.preventDefault();
  });
  return room;
}

const STRIP = "the strip";

const ARRIVING: Move = {
  name: "Arriving",
  what: "A volume or brightness key: the strip rises from the bottom edge.",
  tracks: [{ part: STRIP, on: "the OSD", target: ".k-sheet-part", property: "--reveal", from: 0, to: 1, duration: "long", curve: "emphasized_decel" }],
  reduced: [{ part: STRIP, on: "the OSD", target: ".k-sheet-part", property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" }],
  back: "cuts",
  sample: () => shot(VOLUME),
};

const LEAVING: Move = {
  name: "Leaving",
  what: "A beat after the last key: the strip sinks back into the edge.",
  tracks: [{ part: STRIP, on: "the OSD", target: ".k-sheet-part", property: "--reveal", from: 1, to: 0, duration: "medium", curve: "emphasized_accel" }],
  reduced: [{ part: STRIP, on: "the OSD", target: ".k-sheet-part", property: "opacity", from: 1, to: 0, duration: "short", curve: "emphasized_accel" }],
  back: "cuts",
  sample: () => shot(VOLUME),
};

const FOLLOWING: Move = {
  name: "A key again",
  what: "The level changes: the fill follows over `short` on `linear`, the number at once.",
  tracks: [{ part: "the fill", on: "the bar", target: ".k-progress", property: "--progress", from: 0.62, to: 0.67, duration: "short", curve: "linear" }],
  reduced: "same",
  back: "plays",
  sample: () => shot({ kind: "volume", value: 0.67 }),
};

type Shown = "rest" | "empty";

const ROWS = [
  { name: "Volume", level: VOLUME },
  { name: "Brightness", level: BRIGHTNESS },
] as const;

export default page<Shown>({
  level: "shell",
  slug: "osd",
  title: "OSD",
  lede: "The volume or the brightness as it changes: a strip rising from the bottom edge.",
  code: { qml: "Osd" },
  from: ["panel-page", "progress-bar"],
  stage: { live, hint: "click it; ↑ ↓ volume, b brightness, m mute", daylight: true },
  variants: [
    { name: "Volume", for: "the speaker's symbol by its level", sample: () => shot(VOLUME) },
    { name: "Brightness", for: "the sun's symbol by its level", sample: () => shot(BRIGHTNESS) },
    { name: "Muted", for: "the crossed speaker; the bar at nothing", sample: () => shot({ ...VOLUME, muted: true }) },
  ],
  sizes: doesntApply("one size: `size.rail` wide, as tall as its row."),
  states: {
    shown: ["rest", "empty"],
    absent: {
      lit: "takes no pointer or keyboard",
      pressed: "takes no press",
      disabled: "shown only while a key changes it",
      selected: "nothing to pick",
      loading: "the level is always known",
      error: "a missing backlight shows no brightness",
    },
    rows: nonEmpty(ROWS.map(({ name, level }) => ({ name, cell: (state: Shown) => shot(state === "empty" ? { ...level, value: 0 } : level) }))),
    looks: { rest: "the symbol, the bar part full, the level", empty: "at nothing: the lowest symbol, the bar's track, 0" },
  },
  focus: doesntApply("it takes no pointer or keyboard, so nothing in it lights."),
  content: {
    text: doesntApply("no words: a symbol and a number up to 100."),
    missingPicture: doesntApply("its symbols are the font's."),
    counts: [
      { name: "0", for: "the lowest symbol, an empty bar", sample: () => shot({ ...VOLUME, value: 0 }) },
      { name: "7", for: "one digit at the room's end", sample: () => shot({ ...VOLUME, value: 0.07 }) },
      { name: "100", for: "the widest number: the room's width", sample: () => shot({ ...VOLUME, value: 1 }) },
    ],
  },
  motion: { arrives: [ARRIVING], leaves: [LEAVING], between: [FOLLOWING] },
  keyboard: {
    keys: [
      { keys: ["XF86AudioRaiseVolume", "XF86AudioLowerVolume"], does: "the volume, through wpctl; it shows" },
      { keys: ["XF86AudioMute"], does: "mutes; the level shows 0" },
      { keys: ["Alt+XF86AudioRaiseVolume", "Alt+XF86AudioLowerVolume"], does: "the volume by 1% rather than 5%" },
      { keys: ["XF86MonBrightnessUp", "XF86MonBrightnessDown"], does: "the brightness, through brightnessctl" },
    ],
    order: "Never a stop for Tab; it takes no keyboard.",
    lands: "Nothing moves the keyboard as it shows or goes.",
  },
  measures: {
    samples: nonEmpty([{ name: "Volume", for: "width, padding, gaps", sample: () => shot(VOLUME) }]),
    measures: [
      { part: "Width", token: "size-rail" },
      { part: "Padding", token: "space-s16" },
      { part: "Between parts", token: "space-s12" },
      { part: "Symbol", token: "icon-s24" },
      { part: "Bar's height", token: "size-progress" },
      { part: "Level's line", token: "type-body_strong-line" },
      { part: "Corners, fillets", token: "radius-surface" },
    ],
    reflow: doesntApply("one size, centred on the focused screen's bottom edge."),
  },
});
