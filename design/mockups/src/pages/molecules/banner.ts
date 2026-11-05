// The Banner page: `docs/design/components/banner.md`. A message that matters on the page itself,
// with what to do about it inside: Info, Warning and Danger, each a rail of its tone down the same
// flat box, Danger holding the button that destroys and the bar its hold fills.

import { type BannerOptions, type BannerTone, BANNER_TONES, banner } from "../../lib/banner.ts";
import { buttonBar } from "../../lib/button-bar.ts";
import { shown, type Words } from "../../lib/content.ts";
import { stack, within } from "../../lib/layout.ts";
import { progressBar, setProgress } from "../../lib/progress.ts";
import { text } from "../../lib/text.ts";
import { number } from "../../lib/tokens.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty } from "../../site/page.ts";
import { page } from "../../site/legacy.ts";
import { settings } from "../../site/settings.ts";

/** A column of text at its widest: a banner of words alone. */
const ROOM = "var(--size-column)";
/** An app's window at its least: a banner holding its button bar, as Kōbō's does. */
const WIDE = "var(--size-app_min_width)";

const TITLE: Words = { short: "Last chance.", long: "Last chance to not fuck this up. Everything on Samsung 990 Pro (nvme0n1, 2 TB) gets erased." };
const BODY: Words = { short: "There's no undo.", long: "All 3 partitions, Windows 11 included. There's no undo, and nothing on it comes back." };

const ERASE: Omit<BannerOptions, "actions" | "foot"> = {
  tone: "Danger",
  icon: "warning",
  title: "Last chance to not fuck this up. Everything on Samsung 990 Pro (nvme0n1, 2 TB) gets erased.",
  body: "All 3 partitions, Windows 11 included. There's no undo.",
};
const BESIDE: Omit<BannerOptions, "actions" | "foot"> = {
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

/** How each tone reads, and when it is used. */
const FOR: Record<BannerTone, string> = {
  Info: "Worth knowing: where an install goes, what a step will do. Its rail and symbol `primary`.",
  Warning: "Worth acting on soon: a battery that may not last, a step that goes wrong unless done right. Its rail and symbol `warning`.",
  Danger: "About to destroy something: a disk about to be erased. Its rail and symbol `error`, its action a held Danger button.",
};

/**
 * The banner Kōbō's Last chance shows: its button bar, Back and the held Install, and the hold's
 * timed bar along its foot. Holding the button, with the pointer or Enter, fills the bar over
 * `duration.hold`; letting go before it is full empties it at once.
 */
function confirm(base: Omit<BannerOptions, "actions" | "foot">, live = false, fraction = 0): HTMLElement {
  const danger = base.tone === "Danger";
  const cta = danger ? "Hold to erase and install" : "Hold to install";
  const bar = buttonBar({ dismiss: "Back", actions: [{ label: cta, icon: "download" }], tone: danger ? "Danger" : "Normal" });
  const timed = progressBar({ fraction, timed: true, danger });
  const shownBanner = banner({ ...base, actions: bar.element, foot: timed });
  const held = bar.actions.at(-1);
  if (live && held) wireHold(held, timed, cta);
  return within(WIDE, shownBanner);
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
    const fraction = Math.min(1, (now - start) / (number("duration-hold") / settings().speed));
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

const DESCRIBED =
  "A status message lying flat in the page as a card does: `surface_container` on a window's `surface`, the sunken soft hairline inside its edge, `radius.content` and `space.s16` inside. Its tone is a rail of neon down its start side, from the top padding to the bottom one with round ends: the tabs' and the collapsible's rail made thicker, `banner.rail` at night and `banner.rail_day` by day, filling a lane `space.s4` wide `space.s12` in, so the words start `space.s12` past the lane, 28 in. In it the symbol in the tone at `subheading`'s 24 on the title's first line, `space.s16` from a `subheading` title in `on_surface` over a `body` line and any paragraphs more in `on_surface_variant`, `space.s4` apart, all wrapping; its button bar `space.s16` under the words, across their width; an optional foot `space.s16` under the rest.";

export default page<"rest">({
  level: "molecules",
  slug: "banner",
  title: "Banner",
  summary: "A message that matters on the page itself, with what to do about it inside: Info, Warning and Danger, a rail of its tone down a flat box.",
  live: () => confirm(ERASE, true),
  description: [
    "Rust `banner::banner(tone, glyph, title, body)` with `paragraphs`, `actions` and `foot`, drawn by `view`; QML `Banner` (`tone`, `icon`, `title`, `body`, `paragraphs`, its actions as children, `foot`). Kōbō's Last chance is a Danger banner holding Back and the held Install. Hold the button, with the pointer or Enter: the bar along the foot fills over `duration.hold`; let go before it is full and it empties.",
    DESCRIBED,
    "Every tone is the same flat box; only the rail and the symbol take the tone: Info `primary`, Warning `warning`, Danger `error`. At night the rail is the tone's core with a lit control's glow round it, near and far, while the signs are on, and glass while they are off; by day it is the tone, with no glow. The rail never outshouts the action: on a Danger banner the loudest thing is the Danger button that does the damage, held, with its timed bar along the foot, the danger bar, so it can't be pressed by accident and nothing asks again.",
    "Its actions are a button bar (`ButtonBar`) with its rules: Back, the dismiss button, at its start, and the action it is about at its end, Filled, or Danger on a Danger banner; where they don't fit on one line they stack.",
    "It sits in the page, read before going on, so it isn't raised and casts no shadow; it never goes by itself and has no close: it says something true about the page for as long as it is true. An empty body leaves the title alone, a line tall. It isn't a stop for Tab; its actions are, in reading order.",
  ],
  variants: nonEmpty([
    ...BANNER_TONES.map((tone) => ({
      name: tone,
      for: FOR[tone],
      sample: () => plain({ tone, icon: tone === "Danger" ? "warning" : tone === "Warning" ? "battery_1_bar" : "info", title: tone === "Danger" ? ERASE.title : tone === "Warning" ? "12% left. Plug the damn thing in." : BESIDE.title, body: tone === "Danger" ? ERASE.body : tone === "Warning" ? "The install takes about ten minutes." : BESIDE.body }),
    })),
    { name: "Title alone", for: "A warning that says it all in its title: a line tall.", sample: () => plain(BATTERY) },
    { name: "With paragraphs", for: "The steps of what to do, a paragraph each, `space.s4` apart.", sample: () => plain(SECURE_BOOT) },
    { name: "With actions and a foot", for: "What to do inside it, the held action last, its timed bar along the foot.", sample: () => confirm(BESIDE) },
  ]),
  sizes: doesntApply("one size: its padding and its words' styles; its width is its room's."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "it takes no pointer and no focus; its actions light as buttons.",
      pressed: "its actions take the press; a held one fills the bar along its foot.",
      disabled: "a banner is shown or not.",
      selected: "it holds no choice.",
      loading: "a held action's bar is its foot, not a state of the banner.",
      empty: "a banner always says something: its title at least.",
      error: "what broke is an ErrorBlock; a banner warns before.",
    },
    rows: nonEmpty(BANNER_TONES.map((tone) => ({ name: tone, cell: () => (tone === "Warning" ? plain(SECURE_BOOT) : confirm(tone === "Danger" ? ERASE : BESIDE, false, 0.4)) }))),
    described: {
      rest: `${DESCRIBED} By day the same roles in daylight colours, the rail its tone with no glow.`,
    },
  },
  focus: doesntApply("it takes no focus. Its rail is a fixed signal, always lit: it burns at night while the signs are on, flickers with them in a power change, and is its plain tone by day."),
  content: {
    text: [
      { name: "The title", words: TITLE, sample: (words: string) => confirm({ ...ERASE, title: words }) },
      { name: "The body", words: BODY, sample: (words: string) => confirm({ ...ERASE, body: words }) },
    ],
    missingPicture: doesntApply("its symbol is one of KubaOS Symbols, named by what it says."),
    counts: [
      { name: "No actions", for: "Something to know with nothing to do here: the words alone.", sample: () => plain(BATTERY) },
      { name: "One", for: "The one thing to do about it, alone at the bar's end.", sample: () => plain({ ...BATTERY, actions: buttonBar({ actions: [{ label: "Install anyway", kind: "Tonal" }] }).element }) },
      { name: "Two", for: "Back, and the action it is about, last.", sample: () => confirm(ERASE) },
    ],
  },
  motion: {
    arrives: doesntApply("it shows with its page, already in place."),
    leaves: doesntApply("it goes with its page, or when what it says stops being true, at once."),
    between: powerMoves(() => confirm(ERASE, false, 0.4)),
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "Through its actions in reading order." },
      { keys: ["Enter"], does: "Presses a focused action; on a held one, held down, fills the bar, and let go before it is full, empties it." },
      { keys: ["Esc"], does: "The page's: back one step, as Back." },
    ],
    order: "Its actions in reading order, Back before the held action; the banner itself isn't a stop.",
    lands: "Where the page puts the keyboard: on Last chance, the held Install, so Enter held is the one way on.",
  },
  measures: {
    samples: nonEmpty([
      { name: "With actions and a foot", for: "Its padding, the rail, the symbol, the words, the button bar and the foot.", sample: () => confirm(ERASE, false, 0.4) },
      { name: "With paragraphs", for: "The gaps between the title, the body and its paragraphs.", sample: () => plain(SECURE_BOOT) },
    ]),
    measures: [
      { what: "Inside it at the top, the end and the foot, the rail's ends in from its top and foot, between the symbol and the words, between the words and the button bar, and above the foot.", token: "space-s16" },
      { what: "The rail's lane in from its start edge, and the words past the lane.", token: "space-s12" },
      { what: "The rail's lane, the rail centred in it.", token: "space-s4" },
      { what: "The rail, at night, before it is rounded to whole device pixels.", token: "banner-rail" },
      { what: "The rail by day.", token: "banner-rail_day" },
      { what: "Between the button bar's buttons.", token: "space-s8" },
      { what: "Between the title, the body and each paragraph.", token: "space-s4" },
      { what: "The symbol, `subheading`'s.", token: "type-subheading-icon" },
      { what: "The title's line, `subheading`'s.", token: "type-subheading-line" },
      { what: "The body's, `body`'s.", token: "type-body-line" },
      { what: "Its corners.", token: "radius-content" },
      { what: "The foot's bar.", token: "size-progress" },
    ],
    reflow: {
      described: "It fills its room at every window size; its words wrap, and its button bar stacks where its buttons don't fit on one line.",
      sample: () => within("100%", stack("s8", "start", banner({ ...ERASE, title: shown(TITLE), body: shown(BODY), actions: buttonBar({ dismiss: "Back", actions: [{ label: "Hold to erase and install", icon: "download" }], tone: "Danger" }).element }), text("A banner as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
