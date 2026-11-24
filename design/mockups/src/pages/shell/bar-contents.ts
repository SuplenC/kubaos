// The Bar contents page: the pills `config/quickshell/modules/bar/` carries, but the tray, the
// recording and the workspaces, which have pages of their own. Each is the library's Pill holding
// its own content.

import type { Words } from "../../lib/content.ts";
import type { Shown } from "../../lib/control.ts";
import { makePill, pillChanging } from "../../lib/pill.ts";
import { text } from "../../lib/text.ts";
import { pillSwap } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import {
  bellPill,
  BRANCH,
  onGlass,
  clockPill,
  idlePill,
  keyboardPill,
  LOADS,
  morePill,
  previewPill,
  proxyPill,
  quietBell,
  READINGS,
  rimPill,
  shedStatus,
  statusPill,
  sysmonContent,
  sysmonPill,
  updatePill,
} from "./bar-parts.ts";

type State = "rest" | "hovered" | "pressed";

const ROWS: readonly { name: string; make: (state: Shown) => HTMLElement }[] = [
  { name: "Clock", make: (state) => clockPill({ state }) },
  { name: "Status", make: (state) => statusPill({ state }) },
  { name: "Bell", make: (state) => bellPill(false, state) },
  { name: "Bell, unread", make: (state) => bellPill(false, state, true) },
  { name: "Keyboard layout", make: (state) => keyboardPill("US", state) },
  { name: "Keep awake", make: (state) => idlePill({ state }) },
  { name: "Proxy", make: (state) => proxyPill(false, state) },
  { name: "Update", make: (state) => updatePill(undefined, state) },
  { name: "Live preview", make: (state) => previewPill(BRANCH, state) },
  { name: "System monitor", make: (state) => sysmonPill({ state }) },
  { name: "System monitor, compact", make: (state) => rimPill(LOADS, state) },
  { name: "More", make: (state) => morePill(state) },
];

const LAYOUT: Words = { short: "US", long: "PL-prog" };

/** The clock with its card open on `view`: the capsule filled, the edge light under that half. */
function clockOpen(view: "clock" | "weather"): HTMLElement {
  const clock = clockPill({ open: view });
  const strip = onGlass(clock);
  const half = clock.querySelector<HTMLElement>(view === "weather" ? ".k-clock-pill-weather" : ".k-clock-pill-when");
  half?.setAttribute("data-pinned", "pressed");
  return strip;
}

/** Every pill side by side on a piece of the glass, as the bar lays them out, and the clock. */
function all(state: Shown = "rest"): HTMLElement {
  return onGlass(...ROWS.map(({ make }) => make(state)));
}

/** The system monitor, which offers Hide on a right click until the pointer leaves. */
function liveSysmon(): HTMLElement {
  const made = makePill({ content: sysmonContent(), lone: false, gap: "s8", interactive: true, opens: true, tooltip: "CPU 37%  ·  61 °C  ·  RAM 11.2 GB of 15.6 GB" });
  let offering = false;
  made.element.addEventListener("contextmenu", (event) => {
    event.preventDefault();
    offering = !offering;
    made.change(sysmonContent({ offering }));
  });
  made.element.addEventListener("pointerleave", () => {
    if (!offering) return;
    offering = false;
    made.change(sysmonContent());
  });
  return made.element;
}

/** The keyboard layout's pill, switching on a click. */
function liveLayout(): HTMLElement {
  const codes = ["US", "PL", "IT"];
  let at = 0;
  const made = makePill({ content: [text(codes[0]!, "button_small", "on_surface")], lone: false, interactive: true, tooltip: "Click or SUPER+SHIFT+Space for the next layout" });
  made.element.addEventListener("click", () => {
    at = (at + 1) % codes.length;
    made.change([text(codes[at]!, "button_small", "on_surface")]);
  });
  return made.element;
}

function live(): HTMLElement {
  return onGlass(previewPill(), idlePill(), proxyPill(), liveLayout(), updatePill(), clockPill(), morePill(), liveSysmon(), rimPill(), bellPill(), statusPill());
}

const SYSMON_SAMPLE = () => onGlass(sysmonPill({ readings: READINGS.slice(0, 2) }));

export default page<State>({
  level: "shell",
  slug: "bar-contents",
  title: "Bar contents",
  lede: "The items the bar carries, flat on its glass: clock, status, bell, ⋯, system monitor, layout, Keep awake, Proxy, update, preview.",
  code: { qml: "Clock, Status, Bell, KeyboardLayout, IdleIndicator, ProxyIndicator, UpdateIndicator, PreviewIndicator, SysmonPill" },
  from: ["pill"],
  stage: { live, hint: "point at one; click US; right-click the monitor", daylight: true },
  variants: [
    { name: "Clock", for: "seal, date, time, weather, flat at rest", sample: () => onGlass(clockPill()) },
    { name: "Clock, its calendar open", for: "the time in its `primary` capsule; the light under the time", sample: () => clockOpen("clock") },
    { name: "Clock, its weather open", for: "the capsule; the light under the weather", sample: () => clockOpen("weather") },
    { name: "Clock, no home city", for: "no weather", sample: () => onGlass(clockPill({ weather: false })) },
    { name: "Status", for: "power, network, Bluetooth, battery", sample: () => onGlass(statusPill()) },
    { name: "Status, wired desktop", for: "no Bluetooth, no battery", sample: () => onGlass(statusPill({ net: "lan", bluetooth: null, battery: null })) },
    { name: "Status, battery low", for: "the battery in `error`", sample: () => onGlass(statusPill({ battery: { percent: 8, low: true } })) },
    { name: "Status, charging", for: "the battery in `primary`", sample: () => onGlass(statusPill({ battery: { percent: 64, charging: true } })) },
    { name: "Bell", for: "while there is a notification, all read", sample: () => onGlass(bellPill()) },
    { name: "Bell, unread", for: "the accent's dot while anything is unread", sample: () => onGlass(bellPill(false, "rest", true)) },
    { name: "Bell, Do not disturb", for: "crossed out; the dot still says unread", sample: () => onGlass(bellPill(true, "rest", true)) },
    { name: "Bell, nothing unread", for: "its hint says so; no peek", sample: () => onGlass(quietBell("hovered")) },
    { name: "Keyboard layout", for: "two layouts or more", sample: () => onGlass(keyboardPill()) },
    { name: "Proxy", for: "on: a locked shield, `tertiary`", sample: () => onGlass(proxyPill()) },
    { name: "Proxy failing", for: "a key with the error dot", sample: () => onGlass(proxyPill(true)) },
    { name: "Update, a release", for: "a dot", sample: () => onGlass(updatePill()) },
    { name: "Update, security", for: "counted", sample: () => onGlass(updatePill(3)) },
    { name: "Live preview", for: "its symbol in `cold`, flat; the branch in its hint", sample: () => onGlass(previewPill()) },
    { name: "More", for: "⋯, a new symbol: tray apps, update, preview", sample: () => onGlass(morePill()) },
    { name: "System monitor, numbers", for: "the default", sample: () => onGlass(sysmonPill()) },
    { name: "System monitor, bars", for: "a compact Meter each", sample: () => onGlass(sysmonPill({ style: "bars" })) },
    { name: "System monitor, sparklines", for: "the last minute each", sample: () => onGlass(sysmonPill({ style: "sparklines" })) },
    { name: "Close to a limit", for: "symbol and words in `warning`", sample: () => onGlass(sysmonPill({ readings: [READINGS[0]!, { ...READINGS[2]!, text: "82°", level: 1 }] })) },
    { name: "Past a limit, a full disk", for: "`error`, and a red drive", sample: () => onGlass(sysmonPill({ readings: [{ ...READINGS[0]!, text: "97%", level: 2 }, READINGS[1]!], fullDisk: true })) },
    { name: "Offering Hide", for: "a right click, until the pointer leaves", sample: () => onGlass(sysmonPill({ offering: true, state: "hovered" })) },
    { name: "System monitor, compact", for: "CPU rim, RAM, temperature: all `primary` at ease", sample: () => onGlass(rimPill()) },
    { name: "Compact, RAM near its limit", for: "its ring in `warning`", sample: () => onGlass(rimPill([LOADS[0]!, { share: 0.88, level: 1 }, LOADS[2]!])) },
    { name: "Compact, CPU past its limit", for: "the rim in `error`", sample: () => onGlass(rimPill([{ share: 0.97, level: 2 }, LOADS[1]!, LOADS[2]!])) },
    { name: "Status, shedding", for: "the share gone, then symbols from the left", sample: () => onGlass(shedStatus({ shed: 3 })) },
    { name: "Clock, compacting", for: "weather and date gone, the seal kept", sample: () => onGlass(clockPill({ date: "none", weather: false })) },
    { name: "Clock while recording", for: "recording merged in: its colours, its dot leading", sample: () => onGlass(clockPill({ seal: false, date: "none", weather: false, recording: true })) },
  ],
  sizes: [
    { name: "Symbols and words", for: "`space.s12` ends, `space.s8` gaps", sample: () => onGlass(statusPill()) },
    { name: "A lone symbol", for: "`space.s8` round it: bell, Proxy, update", sample: () => onGlass(bellPill()) },
    { name: "Compact system monitor", for: "a `size.control_sm` circle, rings edge to edge", sample: () => onGlass(rimPill()) },
    { name: "System monitor", for: "`space.s8` between readings", sample: SYSMON_SAMPLE },
  ],
  states: {
    shown: ["rest", "hovered", "pressed"],
    absent: {
      lit: "no keyboard focus: the bar is never a Tab stop",
      disabled: "a pill with nothing to do is gone",
      selected: "no pill holds a pick",
      loading: "it shows once its service answers",
      empty: "a pill with nothing to show is gone",
      error: "in its content: Variants' failing and past-limit rows",
    },
    rows: nonEmpty(ROWS.map(({ name, make }) => ({ name, cell: (state: State) => onGlass(make(state)) }))),
    looks: {
      rest: "flat on the glass, content in its roles; rings `primary`",
      hovered: "an opener's edge light, `opacity.track_lit`; its hint",
      pressed: "its content dips; the edge light full",
    },
  },
  focus: doesntApply("the bar takes no keyboard focus, so a pill never lights its tube."),
  content: {
    text: [
      { name: "The layout's code", words: LAYOUT, sample: (words: string) => keyboardPill(words) },
      { name: "The branch, in the hint", words: { short: "fix", long: BRANCH }, sample: (words: string) => previewPill(words, "hovered") },
    ],
    missingPicture: doesntApply("its symbols are KubaOS Symbols, always there."),
    counts: [
      { name: "One reading", for: "the system monitor at its least", sample: () => onGlass(sysmonPill({ readings: READINGS.slice(0, 1) })) },
      { name: "Three readings", for: "CPU, RAM, temperature", sample: () => onGlass(sysmonPill()) },
      { name: "Status at its least", for: "power, network", sample: () => onGlass(statusPill({ bluetooth: null, battery: null })) },
    ],
  },
  motion: {
    arrives: doesntApply("a pill shows at once as its service turns on; the row eases."),
    leaves: doesntApply("it goes at once as its service turns off."),
    between: [
      {
        name: "Its content changes",
        what: "Hide offered, or a layout switched: the old fades as its room closes, the new as its opens.",
        ...pillSwap("the system monitor"),
        back: "plays",
        sample: () => pillChanging({ lone: false, gap: "s8" }, sysmonContent(), sysmonContent({ offering: true })),
      },
      { name: "Hovered", what: "The pointer onto an opener: the edge light comes in under it over `short`.", tracks: [{ part: "the edge light", on: "the pill under the pointer", target: ".k-edge-hover", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }], reduced: "same", back: "cuts", sample: () => all("hovered") },
      { name: "Back to rest", what: "The pointer off: the light goes over `short`.", tracks: [{ part: "the edge light", on: "the pill it leaves", target: ".k-edge-hover", property: "opacity", from: 1, to: 0, duration: "short", curve: "standard" }], reduced: "same", back: "cuts", sample: () => all("hovered") },
      { name: "Pressed", what: "Over `micro` its content dips and the edge light goes to full.", tracks: [{ part: "the content", on: "the pill pressed", target: ".k-bar-dip", property: "scale", from: 1, to: { scale: "press_dip" }, duration: "micro", curve: "standard" }, { part: "the edge light", on: "the pill pressed", target: ".k-edge-hover", property: "--rail-lit", from: 0, to: 1, duration: "micro", curve: "standard" }], reduced: "same", back: "plays", sample: () => all("hovered") },
    ],
  },
  keyboard: doesntApply("never a stop for Tab; SUPER+SHIFT+Space switches the layout, the rest are Command menu Items."),
  measures: {
    samples: nonEmpty([
      { name: "Clock", for: "seal, date, capsule, weather", sample: () => onGlass(clockPill()) },
      { name: "Status", for: "symbols, battery's gap", sample: () => onGlass(statusPill()) },
      { name: "System monitor", for: "readings, their gaps", sample: SYSMON_SAMPLE },
      { name: "Bars", for: "a compact meter", sample: () => onGlass(sysmonPill({ style: "bars", readings: READINGS.slice(0, 1) })) },
    ]),
    measures: [
      { part: "Height", token: "size-control_sm" },
      { part: "Ends", token: "space-s12" },
      { part: "Between readings", token: "space-s8" },
      { part: "Ends round a lone symbol; gaps", token: "space-s8" },
      { part: "Symbol to its words", token: "space-s4" },
      { part: "Symbols", token: "icon-s16" },
      { part: "Time's line", token: "type-body-line" },
      { part: "Readings' line", token: "type-mono-line" },
      { part: "A meter's width", token: "size-control_sm" },
      { part: "A meter's height", token: "size-progress" },
      { part: "Compact rings, middle to middle", token: "space-s4" },
    ],
    reflow: doesntApply("as wide as its content at every window size."),
  },
});
