// The Banner page: `docs/design/components/banner.md`. A message that matters on the page itself,
// with what to do about it inside: Info, Warning and Danger, really red, holding the button that
// destroys and the bar its hold fills.

import { type BannerOptions, type BannerTone, BANNER_TONES, banner, onRed } from "../../lib/banner.ts";
import { button } from "../../lib/button.ts";
import { shown, type Words } from "../../lib/content.ts";
import { keycap } from "../../lib/keycap.ts";
import { row, stack, within } from "../../lib/layout.ts";
import { progressBar, setProgress } from "../../lib/progress.ts";
import { text } from "../../lib/text.ts";
import { number } from "../../lib/tokens.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { settings } from "../../site/settings.ts";

/** A column of text at its widest: a banner of words alone. */
const ROOM = "var(--size-column)";
/** An app's window at its least: a banner holding its actions beside its words, as Kōbō's does. */
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
  Info: "Worth knowing: where an install goes, what a step will do. `surface_container_high`, its edge and symbol `primary`.",
  Warning: "Worth acting on soon: a battery that may not last, a step that goes wrong unless done right. Its edge and symbol `warning`.",
  Danger: "About to destroy something: a disk about to be erased. Really red, `error_container`, its edge and symbol `error`.",
};

/** The held button and its hint under it: or hold, and Enter's keycap. */
function held(tone: BannerTone, label: string): { readonly part: HTMLElement; readonly button: HTMLButtonElement } {
  const danger = tone === "Danger";
  const pressed = button({ label, kind: danger ? "Danger" : "Filled", size: "Lg", icon: "download" });
  const ink = danger ? "rgb(from var(--color-on_error_container) r g b / var(--opacity-subdued_text))" : "var(--color-on_surface_variant)";
  const hint = text("or hold", "caption");
  hint.style.color = ink;
  return { part: stack("s8", "centre", pressed, row("s4", "centre", hint, keycap("Enter"))), button: pressed };
}

/**
 * The banner Kōbō's Last chance shows: Back on its red, the held Install, and the hold's timed bar
 * along its foot. Holding the button, with the pointer or Enter, fills the bar over `duration.hold`;
 * letting go before it is full empties it at once.
 */
function confirm(base: Omit<BannerOptions, "actions" | "foot">, live = false, fraction = 0): HTMLElement {
  const danger = base.tone === "Danger";
  const cta = danger ? "Hold to erase and install" : "Hold to install";
  const back = button({ label: "Back", kind: "Text", icon: "arrow_back" });
  if (danger) onRed(back);
  const hold = held(base.tone, cta);
  const bar = progressBar({ fraction, timed: true, danger });
  const shownBanner = banner({ ...base, actions: [back, hold.part], foot: bar });
  if (live) wireHold(hold.button, bar, cta);
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
  "A status message lying flat in the page: its tone's fill, the sunken soft hairline inside its edge, `radius.content` and `space.s16` inside, and its left edge lit in its tone (the thin stroke, tapering into both corners: the tone's core with its near glow at night while the signs are on, the tone by day). In it a symbol at `subheading`'s 24, `space.s16` from a `subheading` title over a `body` line and any paragraphs more, `space.s4` apart, all wrapping; its actions `space.s12` apart at its end beside the words, the one it is about last; an optional foot `space.s16` under the rest.";

export default page<"rest">({
  level: "molecules",
  slug: "banner",
  title: "Banner",
  summary: "A message that matters on the page itself, with what to do about it inside: Info, Warning, and Danger, really red.",
  live: () => confirm(ERASE, true),
  description: [
    "Rust `banner::banner(tone, glyph, title, body)` with `paragraphs`, `actions` and `foot`, drawn by `view`, and `banner::inside` for a Text button on its red; QML `Banner` (`tone`, `icon`, `title`, `body`, `paragraphs`, its actions as children, `foot`). Kōbō's Last chance is a Danger banner holding Back and the held Install. Hold the button, with the pointer or Enter: the bar along the foot fills over `duration.hold`; let go before it is full and it empties.",
    DESCRIBED,
    "Info and Warning lie on `surface_container_high`, their edge and symbol `primary` and `warning`, their title `on_surface` and body `on_surface_variant`. Danger is really red, so the step that destroys something can't be skimmed: `error_container`, its edge and symbol `error`, its title `on_error_container` and its body that at `opacity.subdued_text`. On its red a Text button takes `on_error_container`, its label, line and glow, and the foot's bar is the danger bar, the banner's own colours rather than the accent's. A Danger action is held, with its timed bar along the foot: it can't be pressed by accident, so nothing asks again.",
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
      rest: `${DESCRIBED} By day the same roles in daylight colours, the edge its tone with no glow.`,
    },
  },
  focus: doesntApply("it takes no focus. Its lit edge is a fixed signal, always lit: it burns at night while the signs are on, flickers with them in a power change, and is its plain tone by day."),
  content: {
    text: [
      { name: "The title", words: TITLE, sample: (words: string) => confirm({ ...ERASE, title: words }) },
      { name: "The body", words: BODY, sample: (words: string) => confirm({ ...ERASE, body: words }) },
    ],
    missingPicture: doesntApply("its symbol is one of KubaOS Symbols, named by what it says."),
    counts: [
      { name: "No actions", for: "Something to know with nothing to do here: the words alone.", sample: () => plain(BATTERY) },
      { name: "One", for: "The one thing to do about it.", sample: () => plain({ ...BATTERY, actions: [button({ label: "Install anyway", kind: "Tonal" })] }) },
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
      { name: "With actions and a foot", for: "Its padding, the symbol, the words, the actions and the foot.", sample: () => confirm(ERASE, false, 0.4) },
      { name: "With paragraphs", for: "The gaps between the title, the body and its paragraphs.", sample: () => plain(SECURE_BOOT) },
    ]),
    measures: [
      { what: "Inside it all round, between the symbol and the words, between the words and the actions, and above the foot.", token: "space-s16" },
      { what: "Between its actions.", token: "space-s12" },
      { what: "Between the title, the body and each paragraph.", token: "space-s4" },
      { what: "The symbol, `subheading`'s.", token: "type-subheading-icon" },
      { what: "The title's line, `subheading`'s.", token: "type-subheading-line" },
      { what: "The body's, `body`'s.", token: "type-body-line" },
      { what: "Its corners.", token: "radius-content" },
      { what: "Its lit edge, before it is rounded to whole device pixels.", token: "stroke-thin" },
      { what: "The foot's bar.", token: "size-progress" },
    ],
    reflow: {
      described: "It fills its room at every window size; its words wrap in the room its actions leave, and the actions keep their size at its end.",
      sample: () => within("100%", stack("s8", "start", banner({ ...ERASE, title: shown(TITLE), body: shown(BODY), actions: [onRed(button({ label: "Back", kind: "Text", icon: "arrow_back" }))] }), text("A banner as wide as the window.", "small", "on_surface_variant"))),
    },
  },
});
