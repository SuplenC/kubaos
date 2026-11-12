// The Bar contents page: the pills `config/quickshell/modules/bar/` carries, but the tray, the
// recording and the workspaces, which have pages of their own. Each is the library's Pill holding
// its own content.

import type { Words } from "../../lib/content.ts";
import type { Shown } from "../../lib/control.ts";
import { row } from "../../lib/layout.ts";
import { makePill } from "../../lib/pill.ts";
import { text } from "../../lib/text.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import {
  bellPill,
  BRANCH,
  CLOCK,
  clockPill,
  countdownWords,
  compactClock,
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

type State = "rest" | "lit" | "pressed";

const ROWS: readonly { name: string; make: (state: Shown) => HTMLElement }[] = [
  { name: "Clock", make: (state) => clockPill({ countdown: CLOCK.countdown, state }) },
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
const COUNTDOWN: Words = { short: "Exam 2 d", long: "Vacation in the mountains 52 d 2 h" };

/** Every pill side by side, as the bar's right end lays them out, and the clock. */
function all(state: Shown = "rest"): HTMLElement {
  return row("s8", "centre", ...ROWS.map(({ make }) => make(state)));
}

/** The system monitor, which offers Hide on a right click until the pointer leaves. */
function liveSysmon(): HTMLElement {
  const made = makePill({ content: sysmonContent(), lone: false, gap: "s8", interactive: true, tooltip: "CPU 37%  ·  61 °C  ·  RAM 11.2 GB of 15.6 GB" });
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
  return row("s8", "centre", previewPill(), idlePill(), proxyPill(), liveLayout(), updatePill(), clockPill({ countdown: CLOCK.countdown }), morePill(), liveSysmon(), rimPill(), bellPill(), statusPill());
}

const SYSMON_SAMPLE = () => sysmonPill({ readings: READINGS.slice(0, 2) });

export default page<State>({
  level: "shell",
  slug: "bar-contents",
  title: "Bar contents",
  lede: "The pills the bar carries, each built from Pill: clock, status, bell, ⋯, system monitor, layout, Keep awake, Proxy, update, preview.",
  code: { qml: "Clock, Status, Bell, KeyboardLayout, IdleIndicator, ProxyIndicator, UpdateIndicator, PreviewIndicator, SysmonPill" },
  from: ["pill"],
  stage: { live, hint: "point at one; click US; right-click the monitor", daylight: true },
  variants: [
    { name: "Clock", for: "kanji `tertiary`, date and time `body`, the time held widest", sample: () => clockPill() },
    { name: "Clock with a countdown", for: "the pinned countdown after the time, `tertiary`", sample: () => clockPill({ countdown: CLOCK.countdown }) },
    { name: "A countdown under a minute", for: "m:ss, never its name alone", sample: () => clockPill({ countdown: countdownWords("Vacation", 45) }) },
    { name: "Status", for: "power, network, Bluetooth, battery", sample: () => statusPill() },
    { name: "Status, wired desktop", for: "no Bluetooth, no battery", sample: () => statusPill({ net: "lan", bluetooth: null, battery: null }) },
    { name: "Status, battery low", for: "the battery in `error`", sample: () => statusPill({ battery: { percent: 8, low: true } }) },
    { name: "Status, charging", for: "the battery in `primary`", sample: () => statusPill({ battery: { percent: 64, charging: true } }) },
    { name: "Bell", for: "while there is a notification, all read", sample: () => bellPill() },
    { name: "Bell, unread", for: "the accent's dot while anything is unread", sample: () => bellPill(false, "rest", true) },
    { name: "Bell, Do not disturb", for: "crossed out; the dot still says unread", sample: () => bellPill(true, "rest", true) },
    { name: "Bell, nothing unread", for: "its hint says so; no peek", sample: () => quietBell("lit") },
    { name: "Keyboard layout", for: "two layouts or more", sample: () => keyboardPill() },
    { name: "Proxy", for: "on: a locked shield, `tertiary`", sample: () => proxyPill() },
    { name: "Proxy failing", for: "a key with the error dot", sample: () => proxyPill(true) },
    { name: "Update, a release", for: "a dot", sample: () => updatePill() },
    { name: "Update, security", for: "counted", sample: () => updatePill(3) },
    { name: "Live preview", for: "its symbol on the cold tint, the branch in its hint", sample: () => previewPill() },
    { name: "More", for: "⋯, a new symbol: tray apps, update, preview", sample: () => morePill() },
    { name: "System monitor, numbers", for: "the default", sample: () => sysmonPill() },
    { name: "System monitor, bars", for: "a compact Meter each", sample: () => sysmonPill({ style: "bars" }) },
    { name: "System monitor, sparklines", for: "the last minute each", sample: () => sysmonPill({ style: "sparklines" }) },
    { name: "Close to a limit", for: "symbol and words in `warning`", sample: () => sysmonPill({ readings: [READINGS[0]!, { ...READINGS[2]!, text: "82°", level: 1 }] }) },
    { name: "Past a limit, a full disk", for: "`error`, and a red drive", sample: () => sysmonPill({ readings: [{ ...READINGS[0]!, text: "97%", level: 2 }, READINGS[1]!], fullDisk: true }) },
    { name: "Offering Hide", for: "a right click, until the pointer leaves", sample: () => sysmonPill({ offering: true, state: "lit" }) },
    { name: "System monitor, compact", for: "CPU rim, RAM, temperature: all `primary` at ease", sample: () => rimPill() },
    { name: "Compact, RAM near its limit", for: "its ring in `warning`", sample: () => rimPill([LOADS[0]!, { share: 0.88, level: 1 }, LOADS[2]!]) },
    { name: "Compact, CPU past its limit", for: "the rim in `error`", sample: () => rimPill([{ share: 0.97, level: 2 }, LOADS[1]!, LOADS[2]!]) },
    { name: "Status, shedding", for: "the share gone, then symbols from the left", sample: () => shedStatus({ shed: 3 }) },
    { name: "Clock, compacting", for: "the date gone, the kanji kept", sample: () => compactClock({ date: "none" }) },
    { name: "Clock while recording", for: "recording merged in: its look, its dot leading", sample: () => compactClock({ kanji: false, date: "none", recording: true }) },
  ],
  sizes: [
    { name: "Symbols and words", for: "`space.s12` ends, `space.s8` gaps", sample: () => statusPill() },
    { name: "A lone symbol", for: "`space.s8` round it: bell, Proxy, update", sample: () => bellPill() },
    { name: "Compact system monitor", for: "a `size.control_sm` circle, rings edge to edge", sample: () => rimPill() },
    { name: "System monitor", for: "`space.s8` between readings", sample: SYSMON_SAMPLE },
  ],
  states: {
    shown: ["rest", "lit", "pressed"],
    absent: {
      disabled: "a pill with nothing to do is gone",
      selected: "no pill holds a pick",
      loading: "it shows once its service answers",
      empty: "a pill with nothing to show is gone",
      error: "in its content: Variants' failing and past-limit rows",
    },
    rows: nonEmpty(ROWS.map(({ name, make }) => ({ name, cell: (state: State) => make(state) }))),
    looks: {
      rest: "`surface_container`, content in its roles; rings `primary`",
      lit: "pointer only: tube in its tone, glow at night, hint",
      pressed: "tube white-hot; by day an ink tint",
    },
  },
  focus: {
    tubes: [
      { name: "Every pill", tone: "accent", filled: false, content: false },
      { name: "Live preview", tone: "cold", filled: false, content: false },
      { name: "Clock while recording", tone: "error", filled: false, content: false },
    ],
    looks: { lit: "the tube inside the edge, the glow outside", signsOff: "the tube as glass, no glow" },
    signsOff: (index: number) => ROWS[index]!.make("signs-off"),
  },
  content: {
    text: [
      { name: "The layout's code", words: LAYOUT, sample: (words: string) => keyboardPill(words) },
      { name: "The branch, in the hint", words: { short: "fix", long: BRANCH }, sample: (words: string) => previewPill(words, "lit") },
      { name: "The countdown", words: COUNTDOWN, sample: (words: string) => clockPill({ countdown: words }) },
    ],
    missingPicture: doesntApply("its symbols are KubaOS Symbols, always there."),
    counts: [
      { name: "One reading", for: "the system monitor at its least", sample: () => sysmonPill({ readings: READINGS.slice(0, 1) }) },
      { name: "Three readings", for: "CPU, RAM, temperature", sample: () => sysmonPill() },
      { name: "Status at its least", for: "power, network", sample: () => statusPill({ bluetooth: null, battery: null }) },
    ],
  },
  motion: {
    arrives: doesntApply("a pill shows at once as its service turns on; the row eases."),
    leaves: doesntApply("it goes at once as its service turns off."),
    between: [
      {
        name: "Its content changes width",
        what: "Hide offered, or a layout switched: it eases to its new width over `medium` on `standard_decel`.",
        tracks: [{ part: "width", on: "the system monitor", target: ".k-pill", property: "--grown", from: 0, to: 1, duration: "medium", curve: "standard_decel" }],
        reduced: [{ part: "width", on: "the system monitor", target: ".k-pill", property: "--grown", from: 0, to: 1, duration: "instant", curve: "standard_decel" }],
        back: "plays",
        sample: () => {
          const made = makePill({ content: sysmonContent({ offering: true }), lone: false, gap: "s8" });
          made.element.style.setProperty("--from", "var(--size-thumbnail_height)");
          made.element.style.setProperty("--to", "var(--size-control_sm)");
          return made.element;
        },
      },
      { name: "Lit", what: "The pointer onto it: it lights over `short` on `standard`.", tracks: lights(".k-pill", "every pill", 0, 1), reduced: "same", back: "cuts", sample: () => all() },
      { name: "Back to rest", what: "The pointer off: back to rest over `short`.", tracks: lights(".k-pill", "every pill", 1, 0), reduced: "same", back: "cuts", sample: () => all("lit") },
      { name: "Pressed", what: "Over `micro` the tube runs white-hot; by day its ink tints it.", tracks: press(".k-pill", "every pill"), reduced: "same", back: "plays", sample: () => all("lit") },
      ...powerMoves(() => all("lit")),
    ],
  },
  keyboard: doesntApply("never a stop for Tab; SUPER+SHIFT+Space switches the layout, the rest are Command menu Items."),
  measures: {
    samples: nonEmpty([
      { name: "Clock", for: "kanji, date, time, countdown", sample: () => clockPill({ countdown: CLOCK.countdown }) },
      { name: "Status", for: "symbols, battery's gap", sample: () => statusPill() },
      { name: "System monitor", for: "readings, their gaps", sample: SYSMON_SAMPLE },
      { name: "Bars", for: "a compact meter", sample: () => sysmonPill({ style: "bars", readings: READINGS.slice(0, 1) }) },
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
