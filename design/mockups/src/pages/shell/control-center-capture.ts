// The control center's capture card and sliders (`CaptureRow.qml`, `Sliders.qml`,
// `components/PowerSlider.qml`): screenshot, annotate, record and the replay buffer on one card,
// then volume, brightness and the power stops as compact sliders.

import type { Words } from "../../lib/content.ts";
import { stack, within } from "../../lib/layout.ts";
import { slider } from "../../lib/slider.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { capture, type Capture, POWER_ICONS, POWER_STOPS, sliders } from "./control-center-parts.ts";

const ROOM = "calc(var(--size-panel) - 2 * var(--container-0-padding))";
const inRoom = (...parts: HTMLElement[]) => within(ROOM, stack("s16", "start", ...parts));

type Shown = "rest" | "lit" | "pressed" | "selected" | "error";

function both(state: Capture = "idle"): HTMLElement {
  return inRoom(capture(state), sliders());
}

const ELAPSED: Words = { short: "0:42", long: "1:12:42" };

export default page<Shown>({
  level: "shell",
  slug: "control-center-capture",
  title: "Capture and sliders",
  lede: "The control center's capture card, then volume, brightness and the power stops as compact sliders.",
  code: { qml: "CaptureRow" },
  stage: { live: () => both(), hint: "drag a slider; press a volume icon", daylight: true },
  variants: [
    { name: "Capture, idle", for: "four buttons, then the replay buffer", sample: () => inRoom(capture("idle")) },
    { name: "Recording", for: "the time in `mono`; Stop in Danger", sample: () => inRoom(capture("recording")) },
    { name: "Replay buffer on", for: "its button Filled", sample: () => inRoom(capture("replay")) },
    { name: "Muted", for: "the volume toggled: Muted, the fill `secondary`", sample: () => inRoom(sliders({ muted: true })) },
    { name: "Each power stop", for: "its symbol in the circle, its name at the end", sample: () => inRoom(...POWER_STOPS.map((_, index) => slider({ icon: POWER_ICONS[index]!, size: "Compact", stops: POWER_STOPS.length, index, names: POWER_STOPS }))) },
  ],
  sizes: doesntApply("compact sliders, `size.slider_compact`, the panel's width."),
  states: {
    shown: ["rest", "lit", "pressed", "selected", "error"],
    absent: {
      disabled: "never: every capture is always there",
      loading: "a capture starts once the panel has gone",
      empty: "the card and sliders always show",
    },
    rows: [
      { name: "Capture", cell: (state: Shown) => inRoom(capture(state === "selected" ? "replay" : state === "error" ? "recording" : "idle", "0:42", state === "lit" || state === "pressed" ? state : undefined)) },
      { name: "Volume", cell: (state: Shown) => inRoom(sliders({ ...(state === "lit" || state === "pressed" ? { pinned: state } : {}), muted: state === "selected", ...(state === "error" ? { volume: 0 } : {}) })) },
    ],
    looks: {
      rest: "Tonal buttons; sliders' fills `primary`",
      lit: "the button's tube, or the slider's pill tube",
      pressed: "the button hot; the fill brightens by `held`",
      selected: "replay buffer Filled; volume muted, fill `secondary`",
      error: "recording: the status and Stop in `error`; volume at nothing",
    },
  },
  focus: {
    tubes: [
      { name: "A capture button", tone: "accent", filled: true, content: false },
      { name: "A slider", tone: "accent", filled: false, content: false },
    ],
    looks: {
      lit: "the button's core; the slider's tube round its pill",
      signsOff: "each line as glass, no glow",
    },
    signsOff: (index: number) => (index === 0 ? inRoom(capture("idle", "0:42", "signs-off")) : inRoom(sliders({ pinned: "signs-off" }))),
  },
  content: {
    text: [{ name: "The recording's time", words: ELAPSED, sample: (words: string) => inRoom(capture("recording", words)) }],
    missingPicture: doesntApply("every symbol is fixed, or the power stop's own."),
    counts: [
      { name: "Three sliders", for: "a laptop: volume, brightness, power", sample: () => inRoom(sliders()) },
      { name: "Two sliders", for: "no backlight: brightness left out", sample: () => inRoom(sliders({ brightness: false })) },
    ],
  },
  motion: {
    arrives: doesntApply("they come with the panel."),
    leaves: doesntApply("they go with the panel; a capture closes it first."),
    between: [
      { name: "A capture button lights", what: "Pointer or keyboard: tube and glow fade in.", tracks: lights(".k-card .k-button:nth-child(2)", "the region button", 0, 1), reduced: "same", back: "plays", sample: () => inRoom(capture()) },
      { name: "A capture button pressed", what: "Its tube runs hot, then the panel closes.", tracks: press(".k-card .k-button:nth-child(2)", "the region button"), reduced: "same", back: "plays", sample: () => inRoom(capture("idle", "0:42", "lit")) },
      ...powerMoves(() => both()),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "each capture button, then each slider" },
      { keys: ["Enter", "Space"], does: "on a capture button: presses it" },
      { keys: ["←", "→"], does: "a slider: a twentieth, or a stop" },
      { keys: ["Home", "End"], does: "a slider's ends" },
      { keys: ["Space"], does: "volume: mutes; power: the next stop" },
    ],
    order: "Region, annotate, record, screen, replay; then volume, brightness, power.",
    lands: "Not on open: the Wi-Fi tile above takes the keyboard.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Card and sliders", for: "the card's inside, the buttons, the sliders' pitch", sample: () => both() },
      { name: "Recording", for: "the status in `mono`", sample: () => inRoom(capture("recording")) },
    ]),
    measures: [
      { part: "Card's inside", token: "space-s16" },
      { part: "Between buttons", token: "space-s4" },
      { part: "A capture button", token: "size-control_sm" },
      { part: "Title line", token: "type-body_strong-line" },
      { part: "Status line", token: "type-small-line" },
      { part: "Slider's height", token: "size-slider_compact" },
      { part: "Between sliders", token: "space-s8" },
      { part: "Card's corners", token: "radius-content" },
    ],
    reflow: doesntApply("the panel is always `size.panel` wide."),
  },
});
