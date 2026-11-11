// Polkit's prompt (`config/quickshell/modules/polkit/Polkit.qml`): a dialog asking for a password,
// or whatever PAM asks for, over the whole screen dimmed, on the overlay layer above the bar. The
// keyboard starts in the field; Enter answers, Esc or Cancel cancels.

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

export type Moment = "asking" | "checking" | "failed";

export interface PromptOptions {
  readonly moment?: Moment;
  readonly message?: string;
  /** What PAM asks for, without its colon: Password, or anything else, said as PAM wrote it. */
  readonly asked?: string;
  /** Drawn with the keyboard in the field, as it opens. */
  readonly landed?: boolean;
  readonly width?: number;
}

/** The dialog: the shield, the title, PAM's message, the field and its note, Cancel and Authenticate. */
function prompt(options: PromptOptions) {
  const moment = options.moment ?? "asking";
  const asked = options.asked ?? "Password";
  // "Password for Jakub" reads; any other prompt shows as PAM wrote it, and whom it is for goes under the body.
  const forWho = asked.toLowerCase() === "password";
  const failed = moment === "failed";
  const password = field({ icon: "key", secret: true, ...(failed ? { state: "Invalid" as const } : {}) });
  return dialog({
    icon: "admin_panel_settings",
    title: "Authentication required",
    body: forWho ? (options.message ?? MESSAGE) : `${options.message ?? MESSAGE}\nAs ${WHO}`,
    asks: labelled(forWho ? `${asked} for ${WHO}` : asked, password, failed ? { note: FAILED, invalid: true } : {}),
    dismiss: "Cancel",
    actions: [{ label: "Authenticate", ...(moment === "checking" ? { disabled: true } : {}) }],
    landed: options.landed ?? true,
  });
}

/** The prompt over the whole screen, the desktop and its bar dimmed under it. */
export function polkit(options: PromptOptions = {}): HTMLElement {
  const width = `calc(${options.width ?? SCREEN_WIDTH} * var(--px))`;
  const under = barScreen(options.width ?? SCREEN_WIDTH, TALL).element;
  return backdrop({ width, height: TALL, scrim: "dim", content: prompt(options).element, under });
}

/** The stage: type and Enter; the first try fails, the second lets it through; Cancel or Esc closes. */
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
  stage: { live, hint: "type, Enter: the first try fails; Esc cancels", daylight: true },
  variants: [
    { name: "A password", for: "the label names whom it is for", sample: () => polkit() },
    { name: "Anything else", for: "PAM's own words; whom under the body", sample: () => polkit({ asked: "Verification code" }) },
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
        name: "Prompt",
        cell: (state: Shown) => polkit({ moment: state === "loading" ? "checking" : state === "error" ? "failed" : "asking", landed: state !== "rest" }),
      },
    ],
    looks: {
      rest: "the field empty, Authenticate ready",
      lit: "the keyboard in the field: its tube",
      loading: "PAM checking: Authenticate disabled",
      error: "the field invalid, the note in `error`",
    },
  },
  focus: {
    tubes: [
      { name: "The field", tone: "accent", filled: false, content: false },
      { name: "The field, invalid", tone: "error", filled: false, content: false },
    ],
    looks: { lit: "the tube inside the field, glow outside", signsOff: "the tube as glass, no glow" },
    signsOff: () => polkit({ landed: true }),
  },
  content: {
    text: [{ name: "PAM's message", words: LONG, sample: (words: string) => polkit({ message: words }) }],
    missingPicture: doesntApply("its one symbol is the font's."),
    counts: doesntApply("one field, one prompt at a time."),
  },
  motion: { arrives: [ARRIVES], leaves: [LEAVES], between: doesntApply("its field and buttons move on their own pages.") },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "in the field: answers, as Authenticate" },
      { keys: ["Tab", "Shift+Tab"], does: "field, Cancel, Authenticate, round" },
      { keys: ["Esc"], does: "cancels the request" },
    ],
    order: "The field, Cancel, Authenticate; it never leaves the dialog.",
    lands: "Opening, in the field; after a failed try, in it again, emptied.",
  },
  measures: {
    samples: nonEmpty([{ name: "Failed", for: "the dialog, the field, the note", sample: () => polkit({ moment: "failed" }) }]),
    measures: [
      { part: "Dialog's width", token: "size-dialog" },
      { part: "Dialog's padding", token: "space-s24" },
      { part: "Title to body", token: "space-s12" },
      { part: "Body to field", token: "space-s16" },
      { part: "Field to buttons", token: "space-s24" },
      { part: "Corners", token: "radius-content" },
    ],
    reflow: { note: "The dialog keeps its width, centred; the dim fills the screen.", sample: () => polkit({ width: NARROW_WIDTH }) },
  },
});
