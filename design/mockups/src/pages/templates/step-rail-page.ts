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
import { doesntApply, type Move, nonEmpty, type Track } from "../../site/page.ts";
import { page } from "../../site/legacy.ts";
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
  what: "Continue or Enter: the rail's rows move on at once (the step left is done, the next one current, its quiet fill with it). The wire's lit length grows from the step left down to the next symbol over `long` on `emphasized_decel`, its end on whole device pixels; that symbol lights (`primary`, and its glow at night) over `short` from `medium` in, as the light arrives at its bead, and the symbol left loses its glow as the light sets off. The new page slides `space.s32` in from the right, the direction of travel, and fades in, both over `long` on `emphasized_decel`; its offset lands on whole device pixels as it goes. The page left goes at once: nothing slides out. The footer band moves with its page. With reduced motion the wire and the symbols change at once and the page only fades in, over `short`.",
  tracks: [...arriving("the page"), ...travelling(1, 2)],
  reduced: [FADE_ONLY("the page")],
  back: "cuts",
  sample: () => framed(updater({ current: 2, arriving: { forward: true, shown: 1 } }), undefined, SAMPLE_HEIGHT),
};

const BACK: Move = {
  name: "Back: the page before arrives",
  what: "Back, Esc or a done step pressed: the same move from the left, the page coming back the way it went. The rail's rows move at once: the step gone back to is current, the steps after it show to do again, and those already reached still take a press, so Continue's way forward can be retraced from the rail. The wire retracts the same way, over `long` on `emphasized_decel`: each symbol it leaves goes out as the light leaves its bead, and the one gone back to glows as the light arrives. Gone back several steps, the light passes each on the way.",
  tracks: [...arriving("the page"), ...travelling(2, 1)],
  reduced: [FADE_ONLY("the page")],
  back: "cuts",
  sample: () => framed(updater({ current: 1, arriving: { forward: false, shown: 1 } }), undefined, SAMPLE_HEIGHT),
};

const NUDGE: Move = {
  name: "Enter refused: the blocker flashes",
  what: "Enter while something blocks Continue: its reason and its symbol turn `error` at once and settle back to `on_surface_variant` over `slow`. Nothing moves; Continue stays disabled.",
  tracks: [{ part: "reason", on: "the blocker", target: ".k-step-blocker", property: "--nudge", from: 1, to: 0, duration: "slow", curve: "standard_accel" }],
  reduced: "same",
  back: "cuts",
  sample: () => band({ blocker: shown(BLOCKER) }),
};

const LIGHT: Move = {
  name: "A done step lighting up",
  what: "The pointer onto a done step that can be gone back to, or the keyboard on it: the accent's tube fades in inside its edge over `short` on `standard`, the glow outside at night. A fade, never the flicker.",
  tracks: lights(".k-step", "a done step", 0, 1),
  reduced: "same",
  back: "plays",
  sample: () => railStep({ title: "Network", icon: "wifi", state: { is: "done" }, pressable: true }),
};

const PRESS: Move = {
  name: "A done step pressed",
  what: "Pressed while lit: the tube runs white-hot at night over `micro`, the tone tints its fill by day. Letting go goes back to its page, which arrives from the left.",
  tracks: press(".k-step", "a done step"),
  reduced: "same",
  back: "plays",
  sample: () => railStep({ title: "Network", icon: "wifi", state: { is: "done" }, pressable: true, pinned: "lit" }),
};

const DESCRIBED =
  "The rail is `size.rail` (236) wide, `space.s16` in above and below and `space.s12` at its sides, on the well's fill (`surface_container_low` at `opacity.well`). Down it: the brand (Kōbō's lockup over its version line, `space.s4` under it) or the window's title over its line, `space.s12` in so it lines up with the steps' symbols; `space.s20`; the steps back to back, each a `Step` in its rail box across the rail's whole inner width, a mark at its end, its corners `radius.surface` less `space.s12`, concentric with the panel's; then room, `space.s16` at least, and at its foot the keys nothing on the page shows, each a line of `KeyHints` (its words `space.s8` after its keys), `space.s4` apart; in Kōbō `space.s16` under them the power button (a sm `Text` button, its symbol on the steps' symbols' edge) and the battery at the far end. Where the steps don't fit they scroll between the brand and the hints, cut at their own box.";

const WIRE =
  "A wire runs down the steps' symbols: one tube `step_rail.wire` (3) thick, centred on them on whole device pixels, from the first symbol's centre to the last's. Unlit it is `outline_variant`; lit, from the first symbol down to the current one's, it is `primary`, with a lit control's glow round it at night (`focus.near` and `focus.far`, while the signs are on) and none by day. Each symbol is a bead on it: the wire and its glow are cut away in a disc `step_rail.bead` (3) wider than the symbol on every side, so the wire passes behind the symbol and whatever is under it (the rail's fill, the current row's) shows round it. A done step's symbol and the current one's are `primary`, the current one glowing at night as the wire does; a step not reached keeps its ink. While the current step runs (Install) its symbol hums, down to `opacity.hum` and back over `hum` each way.";

export default page<"rest" | "lit" | "pressed" | "disabled" | "selected" | "loading" | "error">({
  level: "templates",
  slug: "step-rail-page",
  title: "StepRailPage",
  summary: "A step rail beside a page with a footer band, and how it moves between its pages: an app's flow (the updater) and Kōbō's install.",
  live: () => framed(updater()),
  description: [
    "One template with two hosts. In an app's window (the updater) the rail and the page fill the window, on `surface`. In Kōbō they are its glass panel over the wallpaper and the scrim: `surface` at `opacity.glass` over the blurred wallpaper, in the soft hairline at `radius.surface`, casting `shadow.panel`. Kōbō keeps its look and stops drawing its own fills (`crates/kubaos-kobo/src/view.rs`). Press a done step, Back, Continue, or Tab in and use Enter and Esc; on You, Enter flashes why Continue waits.",
    DESCRIBED,
    WIRE,
    "A done step shows the `check` mark and takes a press to go back to its page, as far back as the first step after one that can't be undone (the install), and none while a step runs. The current step is a quiet fill, `on_surface` at `opacity.hover`, its symbol lit; Continue stays the only loud thing. Steps not reached take no press and are passed over. A step that failed shows `error` in its symbol's place, with no glow (`Step`).",
    "The page beside the rail is a clear `Page`: the container padding at its sides by its own width, `space.s32` above, `space.s24` below. Its body takes the height left; then `space.s16` and the footer band, which has **no hairline**: the page runs into it. The band is Back (`Text`, `arrow_back`) on the left, its symbol on the page's text edge, its own padding reaching past it, and the one `Filled` button on the right (Continue, Skip, Take it, with `arrow_forward` after its label), `space.s16` apart, and a note in `small` just before it: 12 of 40 picked; or, while something blocks Continue, its reason after the `error` symbol, Continue disabled, the reason flashing in `error` when Enter is refused. A page whose action is in its body (Last chance's held button) or that runs (Install) has no band.",
    "Kōbō's panel sits `space.s32` in from the screen's edges where the screen holds two columns of text side by side (2 × `size.column`), `space.s16` on a smaller one, centred, and never wider than the rail beside two columns with `space.s40` either side, so lines stay short on a big screen. In a window the template is the window: it grows with it, the rail keeping its width.",
  ],
  variants: [
    { name: "In an app's window", for: "The updater: its title at the rail's top, the rail and the page filling the window. Shown at an app's minimum, 720 × 480.", sample: () => framed(updater()) },
    { name: "Framed in Kōbō", for: "Kōbō's install: its lockup, the power button and the battery in the rail, the panel over the wallpaper. Shown on a screen an app's default size, 1120 × 720.", sample: () => framed(kobo(), "var(--size-app_width)", "var(--size-app_height)") },
  ],
  sizes: [
    { name: "The rail", for: "One width, `size.rail`, at every window size: a rail of short step names needs no more. The page takes the rest.", sample: () => framed(updater(), "var(--size-app_min_width)", SAMPLE_HEIGHT) },
    { name: "Kōbō on a screen of the toolbar's window size", for: "The panel's inset, width and centring follow the screen: pick a window size in the toolbar.", sample: () => framed(kobo(), windowSize().width, windowSize().height) },
  ],
  states: {
    shown: ["rest", "lit", "pressed", "disabled", "selected", "loading", "error"],
    absent: {
      empty: "a flow always has its steps and its page; a page with nothing to show holds its own `EmptyState`.",
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
    described: {
      rest: "A step to do: no fill, its words `on_surface`, its mark's room kept. The band: Back and Continue at rest. By day the same roles in daylight colours.",
      lit: "A done step under the pointer or the keyboard: the accent's tube inside its edge, the glow outside at night. A lit Continue: the core inside the fill's edge and the glow; by day the line alone.",
      pressed: "Overdriven: the step's tube white-hot at night, the tone's tint by day; Continue's fill a touch brighter and its glow stronger.",
      disabled: "A done step behind one that can't be undone, or any step while one runs, takes no press: it keeps its check and is passed over. Continue while something blocks it: disabled, its reason in `small` with the `error` symbol before it.",
      selected: "The current step: a quiet fill, `on_surface` at `opacity.hover`, its words as they rest and its symbol `primary`, glowing at night. The band with a note before Continue.",
      loading: "The current step running (Install): its symbol hums, down to `opacity.hum` and back over `hum` each way (still with reduce motion), and no step takes a press until it ends; its page has no band.",
      error: "The current step failed: its symbol is `error`, in `error`, with no glow. The band's blocker as Enter is refused: its reason and symbol in `error`, settling back over `slow`.",
    },
  },
  focus: {
    tubes: [
      { name: "A done step", tone: "accent", filled: false, content: false },
      { name: "Back", tone: "accent", filled: false, content: true },
      { name: "Continue", tone: "accent", filled: true, content: true },
    ],
    looks: {
      lit: "At night: a step's tube inside its edge with the glow outside; Back's tube and its burning label; Continue's core inside its fill's edge, the glow outside and its label brighter.",
      signsOff: "By day, and at night while the signs are off: each line as unlit glass with no glow, the labels at rest.",
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
    missingPicture: doesntApply("the rail's symbols are KubaOS Symbols, and Kōbō's lockup and wallpaper ship inside it; a window's brand is its title."),
    counts: [
      { name: "Three steps", for: "A short flow: the rail's room under them is empty down to the hints.", sample: () => framed(updater({ pages: UPDATER.slice(0, 3), current: 0, reached: 0 }), undefined, SAMPLE_HEIGHT) },
      { name: "Kōbō's nine", for: "Language to Install, with Network shown only offline and Wallpapers only where the payload has sets.", sample: () => framed(kobo({ current: 7, reached: 7 }), "var(--size-app_width)", "var(--size-app_height)") },
      { name: "The first page", for: "Back on the first page goes nowhere: Kōbō's Language goes back to Welcome, the full-bleed screen before the panel.", sample: () => framed(updater({ current: 0, reached: 0 }), undefined, SAMPLE_HEIGHT) },
    ],
  },
  motion: {
    arrives: doesntApply("the template comes with its window, which Hyprland opens, or with Kōbō's panel, which arrives with its Backdrop (the Backdrop page)."),
    leaves: doesntApply("it goes with its window or its Backdrop."),
    between: [ON, BACK, NUDGE, LIGHT, PRESS, ...powerMoves(() => railStep({ title: "Network", icon: "wifi", state: { is: "done" }, pressable: true, pinned: "lit" }))],
  },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "The page's `Filled` action: Continue, once per press; while something blocks it, its reason flashes. On a focused step or button, presses that." },
      { keys: ["Esc"], does: "Back one page; never past a step that can't be undone." },
      { keys: ["Tab", "Shift+Tab"], does: "Through the done steps that take a press, then the page's controls in reading order, then Back and Continue." },
      { keys: ["↑", "↓", "k", "j"], does: "In the rail: to the step above or below that takes a press." },
      { keys: ["Space"], does: "Presses the focused step or button." },
    ],
    order: "The rail's pressable steps top to bottom, the page's controls, Back, Continue; the power button last, in Kōbō.",
    lands: "A page arriving puts the keyboard on its first thing to do: its first field, its list, or Continue.",
  },
  measures: {
    samples: nonEmpty([
      { name: "The rail", for: "Its padding, the brand, `space.s20`, the steps, the hints and the foot.", sample: () => framed(kobo(), "var(--size-app_width)", "var(--size-app_height)") },
      { name: "The footer band", for: "Back, the note and Continue, `space.s16` apart.", sample: () => band({ note: "12 of 40 picked." }) },
    ]),
    measures: [
      { what: "The rail's width.", token: "size-rail" },
      { what: "The rail's padding above and below; the least room above the hints; between the body and the footer band; between the band's parts.", token: "space-s16" },
      { what: "The rail's padding at its sides, and the brand's and hints' inset, so they line up with the steps' symbols.", token: "space-s12" },
      { what: "Between the brand and the steps.", token: "space-s20" },
      { what: "Between a key and what it does.", token: "space-s8" },
      { what: "Between hints; under the lockup.", token: "space-s4" },
      { what: "The wire's thickness, before it is rounded to whole device pixels.", token: "step_rail-wire" },
      { what: "How far a symbol's bead reaches past the symbol: the wire stops that far short of it.", token: "step_rail-bead" },
      { what: "The page's room above; how far a page slides in; Kōbō's panel's inset on a screen two columns wide.", token: "space-s32" },
      { what: "The page's room below, where the band sits.", token: "space-s24" },
      { what: "The band's buttons.", token: "size-control_md" },
      { what: "Kōbō's panel's corners, and the rail's outer ones.", token: "radius-surface" },
      { what: "Either side of the two columns, in Kōbō's widest panel.", token: "space-s40" },
      { what: "A column of text: two of them decide Kōbō's panel's inset and its widest.", token: "size-column" },
    ],
    reflow: {
      described:
        "The updater in a window of the toolbar's size. The rail keeps `size.rail` and runs the window's height, its hints at its foot; the page takes the rest, its padding the container's step for its own width: 484 at the 720 minimum, padded 16; 884 at the 1120 default, 24; 1684 on a 1080p screen, 32. Its body takes the height left above the band; a page whose body is taller scrolls it there, in its own `Scroll`. Kōbō's panel is under Sizes, on a screen of the same size.",
      sample: () => updater(),
    },
  },
});
