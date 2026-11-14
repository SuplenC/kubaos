// The StepRailPage page: `docs/design/components/step-rail-page.md`. A step rail beside a page with
// a footer band, and how it moves between its pages: in an app's window (the updater), and framed
// in Kōbō's glass panel over its wallpaper.

import "./step-rail-page.css";
import wallpaper from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import blurred from "../../../../../crates/kubaos-kobo/assets/wallpaper-blur.jpg?url";
import mark from "../../../../../crates/kubaos-kobo/assets/logo/kubaos-mark.svg?raw";
import wordmark from "../../../../../crates/kubaos-kobo/assets/logo/kubaos-wordmark.svg?raw";
import { banner } from "../../lib/banner.ts";
import { shown, type Words } from "../../lib/content.ts";
import { field, labelled } from "../../lib/field.ts";
import type { KeyHint } from "../../lib/keycap.ts";
import { filling, header, stack, within } from "../../lib/layout.ts";
import { list } from "../../lib/list.ts";
import { step, type StepOptions } from "../../lib/step.ts";
import { footer, koboBrand, type RailPage, stepRailPage, type StepRailOptions, windowBrand } from "../../lib/step-rail.ts";
import { text } from "../../lib/text.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, type Track, page } from "../../site/page.ts";
import { windowSize } from "../../site/settings.ts";

/** A step in the rail, inside its `space.s12` of padding. */
const RAIL = "calc(var(--size-rail) - 2 * var(--space-s12))";

const PAGE_TITLE: Words = {
  short: "Network",
  long: "Network, so the install can fetch what changed since this stick was made",
};
const STEP_TITLE: Words = { short: "You", long: "Your account and the machine's name" };
const BLOCKER: Words = {
  short: "Pick a username first.",
  long: "Pick a username first: lowercase letters, digits and dashes, starting with a letter.",
};

/** A page's body: its header, then what it asks. */
function body(accent: string, title: string, subtitle: string, ...content: Element[]): () => HTMLElement {
  return () => stack("s24", "start", header({ accent, title, subtitle }), ...content);
}

/** Kōbō's flow, as its panel shows it: Language to Install. */
const KOBO: readonly RailPage[] = [
  {
    step: { title: "Language", icon: "language" },
    body: () =>
      body("言語と入力", "Language", "Pick what you read and what you type with.", list([
        { title: "English (United Kingdom)", subtitle: "en_GB.UTF-8", checked: true },
        { title: "Polski", subtitle: "pl_PL.UTF-8", checked: false },
        { title: "日本語", subtitle: "ja_JP.UTF-8", checked: false },
      ]))(),
  },
  { step: { title: "Time zone", icon: "schedule" }, body: body("時間帯", "Time zone", "Where your clock lives.", field({ placeholder: "Search for a city", icon: "search" })) },
  {
    step: { title: "Network", icon: "wifi" },
    body: () =>
      body("接続", shown(PAGE_TITLE), "Only for the install's updates. Skip it and it installs from the stick.", list([
        { title: "Hoshi 5G", subtitle: "Secured · WPA3", subtitleIcon: "lock", icon: "network_wifi_3_bar", checked: true },
        { title: "Kissa", subtitle: "Secured · WPA2", subtitleIcon: "lock", icon: "network_wifi_2_bar", checked: false },
      ]))(),
  },
  {
    step: { title: shown(STEP_TITLE), icon: "person" },
    body: body("アカウント", "You", "Who logs in. The password unlocks the disk too.", labelled("Username", field({ placeholder: "kuba", icon: "person" })), labelled("Password", field({ placeholder: "At least eight characters", icon: "lock", secret: true }))),
    blocker: shown(BLOCKER),
  },
  {
    step: { title: "Disk", icon: "hard_drive" },
    body: () =>
      body("ディスク", "Disk", "Where it goes. Nothing is written until the last screen.", list([
        { title: "Samsung 990 Pro", subtitle: "2 TB · /dev/nvme0n1", icon: "hard_drive", checked: true },
        { title: "USB stick", subtitle: "32 GB · /dev/sda · this installer", icon: "hard_drive", disabled: true },
      ]))(),
  },
  { step: { title: "Git", icon: "code" }, body: body("署名", "Git", "Who your commits say they are.", labelled("Email", field({ placeholder: "you@example.com", icon: "mail" }))), next: "Skip" },
  { step: { title: "Wallpapers", icon: "wallpaper" }, body: body("壁紙", "Wallpapers", "A new one every hour, from the sets you pick."), note: "12 of 40 picked." },
  {
    step: { title: "Last chance", icon: "list_alt" },
    body: body("確認", "Last chance", "Read it once more. Then hold the button.", banner({ tone: "Danger", icon: "warning", title: "Samsung 990 Pro gets erased", body: "Everything on it goes: 2 TB, three partitions, Windows among them." })),
    footless: true,
  },
  { step: { title: "Install", icon: "download" }, body: body("インストール中", "Installing", "Twelve minutes, give or take. The kettle's faster."), footless: true, fixed: true },
];

/** The updater's flow, in its own window. */
const UPDATER: readonly RailPage[] = [
  { step: { title: "What's new", icon: "info" }, body: body("新機能", "What's new", "0.5 Shibuya: the Command menu learnt sums, and the lock screen a clock."), next: "Continue" },
  { step: { title: "Snapshot", icon: "schedule" }, body: body("スナップショット", "Snapshot", "A snapshot first, so a bad update rolls back from the boot menu."), next: "Take it" },
  { step: { title: "Download", icon: "download" }, body: body("ダウンロード", "Download", "1.2 GB from the mirror nearest you."), note: "About four minutes." },
  { step: { title: "Install", icon: "system_update_alt" }, body: body("インストール", "Install", "It installs while you work. Nothing restarts on its own."), footless: true, fixed: true },
  { step: { title: "Reboot", icon: "restart_alt" }, body: body("再起動", "Reboot", "Reboot when it suits you. The new version starts then."), next: "Reboot" },
];

/**
 * The keys Kōbō's rail lists: the ones nothing on the page shows. Enter and Esc go, since Continue
 * and Back show them; the running install, with no buttons, says what Enter does there.
 */
function koboHints(index: number): readonly KeyHint[] {
  return index === 8 ? [["Enter", "show the log"]] : [["↑ ↓", "pick"], ["Tab", "next field"]];
}

/** The updater's: only the running install's Enter, which no button shows. */
function updaterHints(index: number): readonly KeyHint[] {
  return index === 3 ? [["Enter", "show the log"]] : [];
}

/** The template in Kōbō: its lockup, the flow, the power button and the battery. */
function kobo(extra: Partial<StepRailOptions> = {}): HTMLElement {
  return stepRailPage({
    host: "kobo",
    brand: koboBrand(mark, wordmark, "Kōbō · Shinjuku 0.4.0"),
    pages: KOBO,
    current: 2,
    reached: 3,
    hints: koboHints,
    foot: { power: true, battery: 82 },
    backdrop: { wallpaper, blurred },
    ...extra,
  });
}

/** The template in an app's window: the updater's title over its flow. */
function updater(extra: Partial<StepRailOptions> = {}): HTMLElement {
  return stepRailPage({ host: "window", brand: windowBrand("Update", "KubaOS 0.4 Shinjuku to 0.5 Shibuya"), pages: UPDATER, current: 1, reached: 2, hints: updaterHints, ...extra });
}

/** `template` in a box `width` × `height` (lengths' CSS): a window, or Kōbō's screen. */
function framed(template: HTMLElement, width = "var(--size-app_min_width)", height = "var(--size-app_min_height)"): HTMLElement {
  const box = within(width, template);
  box.classList.add("t-frame");
  box.style.height = height;
  return box;
}

/** A shorter window, for a move's sample. */
const SAMPLE_HEIGHT = "calc(var(--size-app_min_height) * 3 / 4)";

/** A rail step, alone at the rail's width. */
function railStep(options: StepOptions): HTMLElement {
  return within(RAIL, step({ box: "rail", mark: true, ...options }));
}

/** The footer band alone, at a page's width beside the rail in the smallest window. */
function band(spec: Partial<RailPage>, pins: Parameters<typeof footer>[3] = {}, nudged?: number): HTMLElement {
  const made = footer({ step: { title: "" }, body: () => stack(0, "start"), ...spec }, undefined, undefined, pins);
  if (nudged !== undefined) made.nudge?.style.setProperty("--nudge", String(nudged));
  return within("calc(var(--size-app_min_width) - var(--size-rail) - 2 * var(--space-s16))", filling(made.band));
}

const ROOM = ".k-step-room";

/** A page arriving: it slides `space.s32` in from where it comes from and fades in, over `long`. */
function arriving(on: string): readonly [Track<"emphasized_decel">, Track<"emphasized_decel">] {
  return [
    { part: "slide", on, target: ROOM, property: "--step-slide", from: 1, to: 0, duration: "long", curve: "emphasized_decel" },
    { part: "fade", on, target: ROOM, property: "opacity", from: 0, to: 1, duration: "long", curve: "emphasized_decel" },
  ];
}

/** The wire's light moving from step `from` to `to`, then the symbol it reaches lighting as it arrives. */
function travelling(from: number, to: number): readonly [Track<"emphasized_decel">, Track<"standard">] {
  return [
    { part: "wire", on: "the rail", target: ".k-rail-wire-lit", property: "--wire-lit", from, to, duration: "long", curve: "emphasized_decel" },
    { part: "symbol", on: "the step arrived at", target: `.k-rail-steps > .k-step:nth-child(${to + 1}) .k-step-bead`, property: "--bead-glow", from: 0, to: 1, duration: "short", cue: "medium", curve: "standard" },
  ];
}

const FADE_ONLY = (on: string): Track<"emphasized_decel"> => ({ part: "fade", on, target: ROOM, property: "opacity", from: 0, to: 1, duration: "short", curve: "emphasized_decel" });

const ON: Move = {
  name: "Continue: the next page arrives",
  what: "Continue or Enter: the new page slides in from the right, the wire's light growing to the next symbol.",
  tracks: [...arriving("the page"), ...travelling(1, 2)],
  reduced: [FADE_ONLY("the page")],
  back: "cuts",
  sample: () => framed(updater({ current: 2, arriving: { forward: true, shown: 1 } }), undefined, SAMPLE_HEIGHT),
};

const BACK: Move = {
  name: "Back: the page before arrives",
  what: "Back, Esc or a done step: the same from the left, the wire retracting.",
  tracks: [...arriving("the page"), ...travelling(2, 1)],
  reduced: [FADE_ONLY("the page")],
  back: "cuts",
  sample: () => framed(updater({ current: 1, arriving: { forward: false, shown: 1 } }), undefined, SAMPLE_HEIGHT),
};

const NUDGE: Move = {
  name: "Enter refused: the blocker flashes",
  what: "Enter while blocked: the reason flashes `error` and settles; nothing moves.",
  tracks: [{ part: "reason", on: "the blocker", target: ".k-step-blocker", property: "--nudge", from: 1, to: 0, duration: "slow", curve: "standard_accel" }],
  reduced: "same",
  back: "cuts",
  sample: () => band({ blocker: shown(BLOCKER) }),
};

const LIGHT: Move = {
  name: "A done step lighting up",
  what: "Pointer or keyboard on it: the tube fades in, never the flicker.",
  tracks: lights(".k-step", "a done step", 0, 1),
  reduced: "same",
  back: "plays",
  sample: () => railStep({ title: "Network", icon: "wifi", state: { is: "done" }, pressable: true }),
};

const PRESS: Move = {
  name: "A done step pressed",
  what: "Pressed while lit: the tube runs hot; letting go returns to its page.",
  tracks: press(".k-step", "a done step"),
  reduced: "same",
  back: "plays",
  sample: () => railStep({ title: "Network", icon: "wifi", state: { is: "done" }, pressable: true, pinned: "lit" }),
};

export default page<"rest" | "lit" | "pressed" | "disabled" | "selected" | "loading" | "error">({
  level: "templates",
  slug: "step-rail-page",
  title: "StepRailPage",
  lede: "A step rail beside a page with a footer band, in an app's window or Kōbō's glass panel.",
  code: { unbuilt: "not built yet" },
  stage: {
    live: () => framed(updater()),
    hint: "press Continue, Back or a done step",
    daylight: true,
  },
  variants: [
    { name: "In an app's window", for: "the updater, at an app's minimum", sample: () => framed(updater()) },
    { name: "Framed in Kōbō", for: "lockup, power and battery; the panel over the wallpaper", sample: () => framed(kobo(), "var(--size-app_width)", "var(--size-app_height)") },
  ],
  sizes: [
    { name: "The rail", for: "`size.rail` at every window size; the page takes the rest", sample: () => framed(updater(), "var(--size-app_min_width)", SAMPLE_HEIGHT) },
    { name: "Kōbō at the toolbar's size", for: "the panel's inset and width follow the screen", sample: () => framed(kobo(), windowSize().width, windowSize().height) },
  ],
  states: {
    shown: ["rest", "lit", "pressed", "disabled", "selected", "loading", "error"],
    absent: {
      empty: "a flow always has steps and a page",
    },
    rows: [
      {
        name: "A rail step",
        cell: (state) => {
          if (state === "selected") return railStep({ title: "Disk", icon: "hard_drive", state: { is: "current" } });
          if (state === "loading") return railStep({ title: "Install", icon: "download", state: { is: "current" }, running: true });
          if (state === "error") return railStep({ title: "Install", icon: "download", state: { is: "current" }, failed: true });
          if (state === "disabled") return railStep({ title: "Network", icon: "wifi", state: { is: "done" } });
          if (state === "rest") return railStep({ title: "Account", icon: "person", state: { is: "todo" } });
          return railStep({ title: "Network", icon: "wifi", state: { is: "done" }, pressable: true, pinned: state });
        },
      },
      {
        name: "The footer band",
        cell: (state) => {
          if (state === "lit" || state === "pressed") return band({}, { next: state });
          if (state === "disabled") return band({ blocker: shown(BLOCKER) });
          if (state === "error") return band({ blocker: shown(BLOCKER) }, {}, 1);
          if (state === "selected") return band({ note: "12 of 40 picked." });
          if (state === "loading") return text("No band: a running page has none.", "small", "on_surface_variant");
          return band({});
        },
      },
    ],
    looks: {
      rest: "a step to do, no fill; Back and Continue",
      lit: "a done step's tube, glow at night; Continue's core",
      pressed: "the tube white-hot, by day the tone's tint; Continue brighter",
      disabled: "behind a locked step, no press; Continue with its reason",
      selected: "current: `on_surface` at `opacity.hover`, symbol `primary`; the band's note",
      loading: "the symbol hums to `opacity.hum`; no press, no band",
      error: "the symbol in `error`, no glow; the blocker flashes `error`",
    },
  },
  focus: {
    tubes: [
      { name: "A done step", tone: "accent", filled: false, content: false },
      { name: "Back", tone: "accent", filled: false, content: true },
      { name: "Continue", tone: "accent", filled: true, content: true },
    ],
    looks: {
      lit: "a step's tube inside its edge; Continue's core; Back's burning label",
      signsOff: "each line as unlit glass, no glow, the labels at rest",
    },
    signsOff: (index: number) =>
      index === 0 ? railStep({ title: "Network", icon: "wifi", state: { is: "done" }, pressable: true, pinned: "signs-off" }) : band({}, { next: "signs-off" }),
  },
  content: {
    text: [
      { name: "A page's title", words: PAGE_TITLE, sample: (words: string) => framed(kobo({ pages: KOBO.map((each, index) => (index === 2 ? { ...each, body: body("接続", words, "Only for the install's updates.") } : each)) }), "var(--size-app_width)", "var(--size-app_height)") },
      { name: "A step's title", words: STEP_TITLE, sample: (words: string) => railStep({ title: words, icon: "person", state: { is: "todo" } }) },
      { name: "The blocker's reason", words: BLOCKER, sample: (words: string) => band({ blocker: words }) },
    ],
    missingPicture: doesntApply("its symbols and Kōbō's art ship inside it; a window's brand is its title."),
    counts: [
      { name: "Three steps", for: "a short flow: room under them, down to the hints", sample: () => framed(updater({ pages: UPDATER.slice(0, 3), current: 0, reached: 0 }), undefined, SAMPLE_HEIGHT) },
      { name: "Kōbō's nine", for: "Language to Install", sample: () => framed(kobo({ current: 7, reached: 7 }), "var(--size-app_width)", "var(--size-app_height)") },
      { name: "The first page", for: "Back goes nowhere; in Kōbō, to Welcome", sample: () => framed(updater({ current: 0, reached: 0 }), undefined, SAMPLE_HEIGHT) },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its window, or Kōbō's panel with its Backdrop."),
    leaves: doesntApply("it goes with its window or its Backdrop."),
    between: [ON, BACK, NUDGE, LIGHT, PRESS, ...powerMoves(() => railStep({ title: "Network", icon: "wifi", state: { is: "done" }, pressable: true, pinned: "lit" }))],
  },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "the `Filled` action; while blocked, its reason flashes" },
      { keys: ["Esc"], does: "back a page, never past a locked step" },
      { keys: ["Tab", "Shift+Tab"], does: "done steps, the page's controls, Back, Continue" },
      { keys: ["↑", "↓", "k", "j"], does: "in the rail, the next step that takes a press" },
      { keys: ["Space"], does: "presses the focused step or button" },
    ],
    order: "Pressable steps top down, the page's controls, Back, Continue; in Kōbō, power last.",
    lands: "A page arriving lands on its first thing to do: a field, its list or Continue.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The rail", for: "padding, brand, steps, hints and foot", sample: () => framed(kobo(), "var(--size-app_width)", "var(--size-app_height)") },
      { name: "The footer band", for: "Back, the note and Continue", sample: () => band({ note: "12 of 40 picked." }) },
    ]),
    measures: [
      { part: "Rail's width", token: "size-rail" },
      { part: "Rail's top, foot; band's gaps", token: "space-s16" },
      { part: "Rail's sides, brand's, hints' inset", token: "space-s12" },
      { part: "Brand → steps", token: "space-s20" },
      { part: "Key → its words", token: "space-s8" },
      { part: "Between hints; under the lockup", token: "space-s4" },
      { part: "Wire's thickness", token: "step_rail-wire" },
      { part: "Bead past the symbol", token: "step_rail-bead" },
      { part: "Page's top, its slide; Kōbō's inset", token: "space-s32" },
      { part: "Page's foot", token: "space-s24" },
      { part: "Band's buttons", token: "size-control_md" },
      { part: "Panel's corners", token: "radius-surface" },
      { part: "Beside Kōbō's widest columns", token: "space-s40" },
      { part: "A column of text", token: "size-column" },
    ],
    reflow: {
      note: "The rail keeps `size.rail`; the page takes the rest, its body scrolling above the band.",
      sample: () => updater(),
    },
  },
});
