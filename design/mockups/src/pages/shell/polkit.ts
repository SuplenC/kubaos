// Polkit's prompt (`config/quickshell/modules/polkit/Polkit.qml`): a dialog asking for a password,
// or whatever PAM asks for, over the whole screen dimmed, on the overlay layer above the bar. The
// keyboard starts in the field; Enter answers, Esc or Cancel cancels. A finger answers it too, on
// the security key's sensor, so with a reader the field ends in the lock's `fingerprint` mark.

import { backdrop } from "../../lib/backdrop.ts";
import type { Words } from "../../lib/content.ts";
import { dialog } from "../../lib/dialog.ts";
import { field, labelled } from "../../lib/field.ts";
import { type Easing, doesntApply, type Move, type NonEmpty, nonEmpty, page, type Track } from "../../site/page.ts";
import { ALL } from "../../site/settings.ts";
import { barScreen } from "./bar-parts.ts";
import { NARROW_WIDTH, SCREEN_WIDTH, TALL } from "./screen-parts.ts";

/** What the shots' agent asks, and for whom. */
export const MESSAGE = "Authentication is needed to run `/usr/bin/apt-get install btop' as the super user";
export const WHO = "Jakub Wasilczyk";
const FAILED = "That didn't work. Fat fingers? Try again.";
/** PAM's cues, as `system/70-bitwarden.sh` sets them on the security key's two lines. */
const TOUCH = "Touch the fingerprint sensor";
const CONFIRM = "Touch the key to confirm the PIN";

/**
 * What the prompt shows: asking, PAM checking, a wrong password, PAM's cue while the key waits for
 * a finger or for a touch after its PIN, or a finger the key doesn't know.
 */
export type Moment = "asking" | "checking" | "failed" | "touch" | "confirm" | "unknown";

export interface PromptOptions {
  readonly moment?: Moment;
  readonly message?: string;
  /** What PAM asks for, without its colon: Password, or anything else, said as PAM wrote it. */
  readonly asked?: string;
  /** Drawn with the keyboard in the field, as it opens. */
  readonly landed?: boolean;
  readonly width?: number;
  /** Whether a security key with a fingerprint sensor is there: its mark ends the field. */
  readonly reader?: boolean;
}

/** The dialog: the shield, the title, PAM's message, the field and its note, Cancel and Authenticate. */
function prompt(options: PromptOptions) {
  const moment = options.moment ?? "asking";
  const asked = options.asked ?? "Password";
  // "Password for Jakub" reads; any other prompt shows as PAM wrote it, and whom it is for goes under the body.
  const forWho = asked.toLowerCase() === "password";
  const failed = moment === "failed";
  const unknown = moment === "unknown";
  const cue = moment === "touch" ? TOUCH : moment === "confirm" ? CONFIRM : "";
  // The lock's reader mark: `primary`, `error` once a finger is unknown, the placeholder saying so.
  const password = field({
    icon: "key",
    secret: true,
    ...(unknown ? { placeholder: "Finger not recognised" } : {}),
    ...((options.reader ?? true) ? { trailing: { icon: "fingerprint", color: unknown ? "error" : "primary" } } : {}),
    ...(failed ? { state: "Invalid" as const } : {}),
  });
  const note = failed ? { note: FAILED, invalid: true } : cue ? { note: cue } : {};
  return dialog({
    icon: "admin_panel_settings",
    title: "Authentication required",
    body: forWho ? (options.message ?? MESSAGE) : `${options.message ?? MESSAGE}\nAs ${WHO}`,
    asks: labelled(forWho ? `${asked} for ${WHO}` : asked, password, note),
    dismiss: "Cancel",
    // PAM asks for nothing while it checks or while the key waits for a finger or a touch.
    actions: [{ label: "Authenticate", ...(moment === "checking" || cue ? { disabled: true } : {}) }],
    landed: options.landed ?? true,
  });
}

/** The prompt over the whole screen, the desktop and its bar dimmed under it. */
export function polkit(options: PromptOptions = {}): HTMLElement {
  const width = `calc(${options.width ?? SCREEN_WIDTH} * var(--px))`;
  const under = barScreen(options.width ?? SCREEN_WIDTH, TALL).element;
  return backdrop({ width, height: TALL, scrim: "dim", content: prompt(options).element, under });
}

/**
 * The stage: type and Enter, or click the reader's mark for a finger; the first try of either
 * fails, the second lets it through; Cancel or Esc closes.
 */
function live(): HTMLElement {
  const under = barScreen(SCREEN_WIDTH, TALL).element;
  const holder = document.createElement("span");
  holder.style.display = "contents";
  let tries = 0;
  const show = (moment: Moment) => {
    const asked = prompt({ moment, landed: false });
    asked.onAnswer = (answer) => {
      if (answer === "Authenticate" && tries === 0) {
        tries = 1;
        show("failed");
      } else {
        tries = 0;
        holder.replaceChildren(closed());
      }
    };
    // A finger on the key: unknown the first time, which changes nothing but the mark and the
    // placeholder; known the next, which authenticates.
    asked.element.querySelector(".k-field-trailing")?.addEventListener("click", (event) => {
      event.stopPropagation();
      if (tries === 0) {
        tries = 1;
        show("unknown");
      } else asked.onAnswer("Finger");
    });
    holder.replaceChildren(backdrop({ width: `calc(${SCREEN_WIDTH} * var(--px))`, height: TALL, scrim: "dim", content: asked.element, under }));
    if (!ALL) requestAnimationFrame(() => asked.land());
  };
  const closed = () => {
    const desk = barScreen(SCREEN_WIDTH, TALL).element;
    desk.addEventListener("click", () => show("asking"), { once: true });
    return desk;
  };
  show("asking");
  return holder;
}

const DIM = "the dim";
const FADE = "the dialog's fade";
const RISE = "the dialog's rise";

function arriving(reduced: boolean): NonEmpty<Track<Easing>> {
  const duration = reduced ? "short" : "long";
  const tracks: Track<Easing>[] = [
    { part: DIM, on: "polkit", target: ".k-backdrop-layers", property: "opacity", from: 0, to: 1, duration, curve: "emphasized_decel" },
    { part: FADE, on: "polkit", target: ".k-dialog", property: "opacity", from: 0, to: 1, duration, curve: "emphasized_decel" },
  ];
  if (!reduced) tracks.push({ part: RISE, on: "polkit", target: ".k-dialog", property: "--dialog-rise", from: 1, to: 0, duration, curve: "emphasized_decel" });
  return nonEmpty(tracks);
}

function leaving(reduced: boolean): NonEmpty<Track<Easing>> {
  const duration = reduced ? "short" : "medium";
  return [
    { part: DIM, on: "polkit", target: ".k-backdrop-layers", property: "opacity", from: 1, to: 0, duration, curve: "emphasized_accel" },
    { part: FADE, on: "polkit", target: ".k-dialog", property: "opacity", from: 1, to: 0, duration, curve: "emphasized_accel" },
  ];
}

const ARRIVES: Move = {
  name: "Asking",
  what: "An app asks polkit: the screen dims and the dialog rises in, the keyboard in the field.",
  tracks: arriving(false),
  reduced: arriving(true),
  back: "cuts",
  sample: () => polkit(),
};

const LEAVES: Move = {
  name: "Answered",
  what: "Authenticated or cancelled: the dialog and the dim fade off the desktop.",
  tracks: leaving(false),
  reduced: leaving(true),
  back: "cuts",
  sample: () => polkit(),
};

type Shown = "rest" | "lit" | "loading" | "error";

const LONG: Words = {
  short: "Authentication is needed to mount the disk.",
  long: "Authentication is needed to run `/usr/bin/apt-get install btop' as the super user, so the package manager can write to the system's folders and leave a record of what it changed.",
};

export default page<Shown>({
  level: "shell",
  slug: "polkit",
  title: "Polkit",
  lede: "The authentication prompt: a dialog over the dimmed screen, the keyboard in its field.",
  code: { qml: "Polkit" },
  from: ["dialog", "field", "button-bar", "backdrop"],
  stage: { live, hint: "type and Enter, or click the print: the first try fails", daylight: true },
  variants: [
    { name: "A password", for: "the label names whom it is for", sample: () => polkit() },
    { name: "Anything else", for: "PAM's own words; whom under the body", sample: () => polkit({ asked: "Verification code" }) },
    { name: "With a reader", for: "`fingerprint` ends the field, in `primary`", sample: () => polkit() },
    { name: "Without a reader", for: "no mark: no key, or none with a finger", sample: () => polkit({ reader: false }) },
    { name: "A finger not recognised", for: "the mark in `error`; the placeholder says so; no shake", sample: () => polkit({ moment: "unknown" }) },
    { name: "The key waiting", for: "PAM's cue under the field; Authenticate waits", sample: () => polkit({ moment: "touch" }) },
    { name: "After the key's PIN", for: "PAM's cue to touch the key", sample: () => polkit({ moment: "confirm", asked: "PIN" }) },
  ],
  sizes: [
    { name: "The shots' screen", for: "1600 by 1000", sample: () => polkit() },
    { name: "A narrow screen", for: "the dialog keeps `size.dialog`", sample: () => polkit({ width: NARROW_WIDTH }) },
  ],
  states: {
    shown: ["rest", "lit", "loading", "error"],
    absent: {
      pressed: "its buttons press on their own page",
      disabled: "open or gone",
      selected: "nothing to pick",
      empty: "always a field to fill",
    },
    rows: [
      {
        name: "With a reader",
        cell: (state: Shown) => polkit({ moment: state === "loading" ? "checking" : state === "error" ? "unknown" : "asking", landed: state !== "rest" }),
      },
      {
        name: "Without a reader",
        cell: (state: Shown) => polkit({ reader: false, moment: state === "loading" ? "checking" : state === "error" ? "failed" : "asking", landed: state !== "rest" }),
      },
    ],
    looks: {
      rest: "the field empty, Authenticate ready",
      lit: "the keyboard in the field: its tube",
      loading: "PAM checking: Authenticate disabled; the mark stays",
      error: "a wrong password: invalid; an unknown finger: the mark in `error`",
    },
  },
  focus: {
    tubes: [
      { name: "The field", tone: "accent", filled: false, content: false },
      { name: "The field, invalid", tone: "error", filled: false, content: false },
    ],
    looks: { lit: "the tube inside the field, glow outside", signsOff: "the tube as glass, no glow" },
    signsOff: (index: number) => polkit({ landed: true, reader: index === 0 }),
  },
  content: {
    text: [{ name: "PAM's message", words: LONG, sample: (words: string) => polkit({ message: words }) }],
    missingPicture: doesntApply("its symbols are the font's."),
    counts: doesntApply("one field, one prompt at a time."),
  },
  motion: { arrives: [ARRIVES], leaves: [LEAVES], between: doesntApply("its field and buttons move on their own pages.") },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "in the field: answers, as Authenticate" },
      { keys: ["Tab", "Shift+Tab"], does: "field, Cancel, Authenticate, round" },
      { keys: ["Esc"], does: "cancels the request" },
      { keys: ["A finger"], does: "on the key's sensor: authenticates, or the mark turns `error`" },
    ],
    order: "The field, Cancel, Authenticate; it never leaves the dialog.",
    lands: "Opening, in the field; after a failed try, in it again, emptied.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Failed", for: "the dialog, the field, the note", sample: () => polkit({ moment: "failed" }) },
      { name: "With a reader", for: "the field's marks at both ends", sample: () => polkit() },
    ]),
    measures: [
      { part: "Dialog's width", token: "size-dialog" },
      { part: "Dialog's padding", token: "space-s24" },
      { part: "Title to body", token: "space-s12" },
      { part: "Body to field", token: "space-s16" },
      { part: "Field to buttons", token: "space-s24" },
      { part: "Reader's mark", token: "type-body_large-icon" },
      { part: "Mark from the field's end", token: "space-s12" },
      { part: "Corners", token: "radius-content" },
    ],
    reflow: { note: "The dialog keeps its width, centred; the dim fills the screen.", sample: () => polkit({ width: NARROW_WIDTH }) },
  },
});
