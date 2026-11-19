// The Banner page: `docs/design/components/banner.md`. A message that matters on the page itself,
// with what to do about it inside: Info, Warning, Danger and Tip, each a rail of its tone down the
// same flat box, Danger holding the button that destroys and the bar its hold fills; Small outside a
// card and a Line inside one, as a tip is.

import { type BannerOptions, type BannerTone, BANNER_TONES, banner, tip, tipLine } from "../../lib/banner.ts";
import { buttonBar } from "../../lib/button-bar.ts";
import { shown, type Words } from "../../lib/content.ts";
import { within } from "../../lib/layout.ts";
import { progressBar, setProgress } from "../../lib/progress.ts";
import { durationMs } from "../../lib/tokens.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { settings } from "../../site/settings.ts";

/** A column of text at its widest: a banner of words alone. */
const ROOM = "var(--size-column)";
/** An app's window at its least: a banner holding its button bar, as Kōbō's does. */
const WIDE = "var(--size-app_min_width)";

const TITLE: Words = { short: "Last chance.", long: "Last chance to not fuck this up. Everything on Samsung 990 Pro (nvme0n1, 2 TB) gets erased." };
const BODY: Words = { short: "There's no undo.", long: "All 3 partitions, Windows 11 included. There's no undo, and nothing on it comes back." };

const ERASE: Omit<BannerOptions, "actions" | "foot"> & { readonly title: string } = {
  tone: "Danger",
  icon: "warning",
  title: "Last chance to not fuck this up. Everything on Samsung 990 Pro (nvme0n1, 2 TB) gets erased.",
  body: "All 3 partitions, Windows 11 included. There's no undo.",
};
const BESIDE: Omit<BannerOptions, "actions" | "foot"> & { readonly title: string } = {
  tone: "Info",
  icon: "storage",
  title: "KubaOS goes into the free space on Samsung 990 Pro (nvme0n1, 2 TB).",
  body: "Windows 11 stays where it is.",
};
const BATTERY: BannerOptions = { tone: "Warning", icon: "battery_1_bar", title: "12% left. Plug the damn thing in.", body: "" };
const SECURE_BOOT: BannerOptions = {
  tone: "Warning",
  icon: "key",
  title: "Enroll the key before you turn Secure Boot on",
  body: "Turn it on first and the machine won't boot KubaOS.",
  paragraphs: ["Reboot into the firmware's setup, enroll the key from the USB stick, then turn Secure Boot on.", "Your password is asked for on the next boot."],
};

/** A text column's width: a Small banner's, or a Line's. */
const LIST = "var(--size-column)";

const SPACES = "[Super] [Tab] goes to the next workspace.";
const RESIZE = "Hold [Super] and drag with the right button to resize a window from anywhere on it; the left button moves it.";

/** How each tone reads, and when it is used. */
const FOR: Record<BannerTone, string> = {
  Info: "worth knowing; rail and symbol `primary`",
  Warning: "worth acting on soon; rail and symbol `warning`",
  Danger: "about to destroy; rail and symbol `error`, a held action",
  Tip: "how to do it faster; rail, symbol and title `cold`",
};

/** Each tone's sample. */
const SAMPLE: Record<BannerTone, BannerOptions> = {
  Info: { tone: "Info", icon: "info", title: BESIDE.title, body: BESIDE.body },
  Warning: { tone: "Warning", icon: "battery_1_bar", title: "12% left. Plug the damn thing in.", body: "The install takes about ten minutes." },
  Danger: { tone: "Danger", icon: "warning", title: ERASE.title, body: ERASE.body },
  Tip: { tone: "Tip", size: "Small", icon: "lightbulb", title: "Tip", body: RESIZE },
};

/**
 * The banner Kōbō's Last chance shows: its button bar, Back and the held Install, and the hold's
 * timed bar along its foot. Holding the button, with the pointer or Enter, fills the bar over
 * `duration.hold`; letting go before it is full empties it at once.
 */
function confirm(base: Omit<BannerOptions, "actions" | "foot">, live = false, fraction = 0, room = WIDE): HTMLElement {
  const danger = base.tone === "Danger";
  const cta = danger ? "Hold to erase and install" : "Hold to install";
  const bar = buttonBar({ dismiss: "Back", actions: [{ label: cta, icon: "download" }], tone: danger ? "Danger" : "Normal" });
  const timed = progressBar({ fraction, timed: true, danger });
  const shownBanner = banner({ ...base, actions: bar.element, foot: timed });
  const held = bar.actions.at(-1);
  if (live && held) wireHold(held, timed, cta);
  return within(room, shownBanner);
}

/** Fills `bar` while `pressed` is held, over `duration.hold` under the speed setting. */
function wireHold(pressed: HTMLButtonElement, bar: HTMLElement, cta: string): void {
  const words = pressed.querySelector<HTMLElement>(".k-text");
  let start = 0;
  let frame = 0;
  const say = (label: string) => {
    if (words) words.textContent = label;
  };
  const tickHold = (now: number) => {
    const fraction = Math.min(1, (now - start) / (durationMs("hold") / settings().speed));
    setProgress(bar, fraction);
    if (fraction < 1) frame = requestAnimationFrame(tickHold);
    else say("Installing");
  };
  const begin = () => {
    if (frame) return;
    start = performance.now();
    say("Keep holding…");
    frame = requestAnimationFrame(tickHold);
  };
  const end = () => {
    cancelAnimationFrame(frame);
    frame = 0;
    setProgress(bar, 0);
    say(cta);
  };
  pressed.addEventListener("pointerdown", begin);
  for (const event of ["pointerup", "pointerleave", "blur"]) pressed.addEventListener(event, end);
  pressed.addEventListener("keydown", (event) => {
    if (event.key !== "Enter" || event.repeat) return;
    event.preventDefault();
    begin();
  });
  pressed.addEventListener("keyup", (event) => {
    if (event.key === "Enter") end();
  });
}

function plain(options: BannerOptions): HTMLElement {
  return within(ROOM, banner(options));
}

export default page<"rest">({
  level: "molecules",
  slug: "banner",
  title: "Banner",
  lede: "A message that matters on the page itself, with what to do about it inside: a rail of its tone down a flat box.",
  code: { rust: "banner::banner", qml: "Banner" },
  stage: {
    live: () => confirm(ERASE, true, 0, "100%"),
    hint: "hold the button, or Tab to it and hold Enter",
    daylight: true,
  },
  variants: nonEmpty([
    ...BANNER_TONES.map((tone) => ({ name: tone, for: FOR[tone], sample: () => within(tone === "Tip" ? LIST : ROOM, banner(SAMPLE[tone])) })),
    { name: "Title alone", for: "says it all in a line", sample: () => plain(BATTERY) },
    { name: "With paragraphs", for: "the steps, `space.s4` apart", sample: () => plain(SECURE_BOOT) },
    { name: "With actions and a foot", for: "a button bar, the held action's timed bar along its foot", sample: () => confirm(BESIDE) },
    { name: "Tip outside a card", for: "titled, under a heading or alone", sample: () => within(LIST, tip(SPACES)) },
    { name: "Tip inside a card", for: "a line under its setting, the symbol first", sample: () => within(LIST, tipLine(SPACES, true)) },
    { name: "Tip inside, wrapping", for: "the rail down every line", sample: () => within("var(--size-find)", tipLine(RESIZE, true)) },
    { name: "Tip inside, no symbol", for: "the rail and the words alone", sample: () => within(LIST, tipLine(SPACES, false)) },
  ]),
  sizes: [
    { name: "Regular", for: "a page's: subheading over body, `space.s16` in", sample: () => plain(SAMPLE.Info) },
    { name: "Small", for: "outside a card: label over small", sample: () => within(LIST, banner(SAMPLE.Tip)) },
    { name: "Line", for: "inside a card: no box, no title", sample: () => within(LIST, tipLine(RESIZE)) },
  ],
  states: {
    shown: ["rest"],
    absent: {
      lit: "takes no pointer or focus; its actions light",
      pressed: "its actions take the press",
      disabled: "shown or not",
      selected: "holds no choice",
      loading: "a held action's bar is its foot",
      empty: "always has a title",
      error: "what broke is an `ErrorBlock`",
    },
    rows: nonEmpty(BANNER_TONES.map((tone) => ({ name: tone, cell: () => (tone === "Warning" ? plain(SECURE_BOOT) : confirm(tone === "Danger" ? ERASE : BESIDE, false, 0.4)) }))),
    looks: {
      rest: "`surface_container`, soft hairline, a rail of its tone; no glow by day",
    },
  },
  focus: doesntApply("it takes no focus; its rail is a fixed signal, burning at night and plain by day."),
  content: {
    text: [
      { name: "The title", words: TITLE, sample: (words: string) => confirm({ ...ERASE, title: words }) },
      { name: "The body", words: BODY, sample: (words: string) => confirm({ ...ERASE, body: words }) },
    ],
    missingPicture: doesntApply("its symbol is one of KubaOS Symbols, named by what it says."),
    counts: [
      { name: "No actions", for: "nothing to do here: the words alone", sample: () => plain(BATTERY) },
      { name: "One", for: "alone at the bar's end", sample: () => plain({ ...BATTERY, actions: buttonBar({ actions: [{ label: "Install anyway", kind: "Tonal" }] }).element }) },
      { name: "Two", for: "Back, and the action last", sample: () => confirm(ERASE) },
    ],
  },
  motion: {
    arrives: doesntApply("it shows with its page, already in place."),
    leaves: doesntApply("it goes with its page, or at once when it stops being true."),
    between: powerMoves(() => confirm(ERASE, false, 0.4)),
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "through its actions" },
      { keys: ["Enter"], does: "presses an action; held, fills a held one's bar" },
      { keys: ["Esc"], does: "the page's: back one step" },
    ],
    order: "Back, then the held action; the banner itself isn't a stop.",
    lands: "Where the page puts it: on Last chance, the held Install.",
  },
  measures: {
    samples: nonEmpty([
      { name: "With actions and a foot", for: "padding, rail, symbol, words, button bar, foot", sample: () => confirm(ERASE, false, 0.4) },
      { name: "With paragraphs", for: "gaps between title, body and paragraphs", sample: () => plain(SECURE_BOOT) },
    ]),
    measures: [
      { part: "Inset, rail ends, gaps to foot", token: "space-s16" },
      { part: "Rail lane in, words past it", token: "space-s12" },
      { part: "Rail lane", token: "space-s4" },
      { part: "Rail at night", token: "banner-rail" },
      { part: "Rail by day", token: "banner-rail_day" },
      { part: "Between buttons", token: "space-s8" },
      { part: "Between title, body, paragraphs", token: "space-s4" },
      { part: "Symbol", token: "type-subheading-icon" },
      { part: "Title line", token: "type-subheading-line" },
      { part: "Body line", token: "type-body-line" },
      { part: "Corner", token: "radius-content" },
      { part: "Foot's bar", token: "size-progress" },
      { part: "Small: inset", token: "space-s12" },
      { part: "Small: lane in, words past", token: "space-s8" },
      { part: "Small and Line: rail, night", token: "tabs-rail" },
      { part: "Small and Line: rail, day", token: "tabs-rail_day" },
      { part: "Small: symbol", token: "type-label-icon" },
      { part: "Line: symbol", token: "type-small-icon" },
      { part: "Line: words past the lane", token: "space-s8" },
      { part: "A line naming keys", token: "size-keycap" },
    ],
    reflow: {
      note: "It fills its room: its words wrap, and its button bar stacks where its buttons don't fit.",
      sample: () => within("100%", banner({ ...ERASE, title: shown(TITLE), body: shown(BODY), actions: buttonBar({ dismiss: "Back", actions: [{ label: "Hold to erase and install", icon: "download" }], tone: "Danger" }).element })),
    },
  },
});
