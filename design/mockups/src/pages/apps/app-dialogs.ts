// Apps' dialogs (`zenity`): the program KubaOS installs as `zenity`, so that Chrome, Steam, Wine and
// scripts asking a question get the library's Dialog instead of GNOME's. It answers the modes
// callers use (`--question`, `--info`, `--warning`, `--error`, `--entry`) with their options and the
// text they send, in a window Hyprland floats and centres over the dimmed desktop, as polkit's.
//
// The page maps zenity's command line onto Dialog's options and draws nothing of its own: the kind's
// symbol, the caller line, the markup body, `--no-wrap`, `--default-cancel` and `--timeout` are all
// Dialog's (`lib/dialog.ts`).

import { backdrop } from "../../lib/backdrop.ts";
import type { Words } from "../../lib/content.ts";
import { type Action, type DialogKind, type DialogOptions, dialog } from "../../lib/dialog.ts";
import { field } from "../../lib/field.ts";
import { within } from "../../lib/layout.ts";
import { log } from "../../lib/log.ts";
import { markupText } from "../../lib/text.ts";
import { type Easing, doesntApply, type Move, type NonEmpty, nonEmpty, page, type Track } from "../../site/page.ts";
import { ALL } from "../../site/settings.ts";
import { barScreen } from "../shell/bar-parts.ts";
import { NARROW_WIDTH, SCREEN_WIDTH, TALL } from "../shell/screen-parts.ts";

/** zenity's modes this program answers. */
type Mode = "question" | "info" | "warning" | "error" | "entry";

/** One call of `zenity`, as its options give it. */
interface Call {
  readonly kind: Mode;
  /** `--title`; with none, the app's name stands in, else the mode's word. */
  readonly title?: string;
  /** `--text`, in Pango's markup. */
  readonly text: string;
  /** The app that asks, where the parent process names it. */
  readonly app?: string;
  /** `--icon-name`: a symbol of the caller's own. */
  readonly icon?: string;
  readonly ok?: string;
  readonly cancel?: string;
  readonly extra?: string;
  readonly defaultCancel?: boolean;
  readonly noWrap?: boolean;
  /** `--hide-text` on `--entry`. */
  readonly hidden?: boolean;
  /** `--entry-text`: what the field starts with. */
  readonly value?: string;
  /** `--timeout`, and the seconds left, drawn still. */
  readonly timeout?: { readonly of: number; readonly left: number };
}

/** Each mode's word, where no title or app stands in, and the Dialog kind it is drawn as. */
const MODES: Record<Mode, { readonly word: string; readonly kind: DialogKind }> = {
  question: { word: "Question", kind: "Info" },
  info: { word: "Information", kind: "Info" },
  warning: { word: "Warning", kind: "Warning" },
  error: { word: "Error", kind: "Error" },
  entry: { word: "Entry", kind: "Info" },
};

/** Chrome's own words when another Chrome holds the profile (from memory, not checked). */
const CHROME: Call = {
  kind: "question",
  app: "Google Chrome",
  title: "Google Chrome",
  text: "The profile appears to be in use by another Google Chrome process (4127) on another computer (kuba-desktop). Chrome has locked the profile so that it doesn't get corrupted. If you are sure no other processes are using this profile, you can unlock the profile and relaunch Chrome.",
  ok: "Unlock profile and relaunch",
  cancel: "Quit",
};

const STEAM_QUESTION: Call = {
  kind: "question",
  app: "Steam",
  title: "Steam",
  text: "Steam needs to install 32-bit libraries to run Windows games. Install them now?",
  ok: "Install",
  cancel: "Not now",
};

const SCRIPT_DELETE: Call = { kind: "question", app: "clean-downloads.sh", text: "Delete 3 files?", ok: "Delete", defaultCancel: true };

const STEAM_ERROR: Call = {
  kind: "error",
  app: "Steam",
  title: "Steam - Fatal Error",
  text: "Steam requires the 32-bit graphics driver, which is missing or broken. On a 64-bit system, install the 32-bit versions of libGL, libvulkan and your GPU's driver (mesa or nvidia), then run Steam again. Without them, games fail to start, the store and library windows stay blank, and Proton can't create a Vulkan device for the game it is asked to run. If you installed Steam from a Flatpak, check that the GL extensions for your driver's version are installed too.",
};

const INFO: Call = { kind: "info", app: "backup.sh", title: "Backup", text: "Backup finished: 12,480 files, 38.2 GB, in 4 min 12 s." };
const WARNING: Call = { kind: "warning", app: "Lutris", title: "Lutris", text: "Wine 9.0 is not installed. The game may not start." };

const ENTRY: Call = { kind: "entry", app: "rename.sh", title: "Rename", text: "New name for the folder:", value: "Holiday 2026" };
const PASSWORD: Call = { kind: "entry", app: "mount-nas.sh", title: "NAS", text: "Password for kuba@nas.local:", hidden: true };
const TIMED: Call = { kind: "question", app: "update.sh", title: "Restart", text: "Updates are installed. Restart now?", ok: "Restart", timeout: { of: 30, left: 18 } };
const WINE: Call = { kind: "warning", app: "Wine", text: "Wine could not find a Gecko package, which is needed for applications embedding HTML. Install it?" };
const NOBODY: Call = { kind: "warning", text: WINE.text };

/** `call` as Dialog's options: zenity's command line, mapped. */
function drawn(call: Call, landed: boolean) {
  const mode = MODES[call.kind];
  const entry = call.kind === "entry";
  const actions: Action[] = [...(call.extra ? [{ label: call.extra, kind: "Tonal" as const }] : []), { label: call.ok ?? "OK" }];
  const marked = markupText(call.text) !== call.text;
  const options: DialogOptions = {
    kind: mode.kind,
    title: call.title ?? call.app ?? mode.word,
    body: call.text,
    ...(marked ? { markup: true } : {}),
    ...(call.noWrap ? { noWrap: true } : {}),
    ...(call.app ? { caller: { name: call.app } } : {}),
    ...(call.hidden ? { icon: "lock" } : call.icon ? { icon: call.icon } : entry ? { icon: "draw" } : {}),
    ...(entry ? { asks: field({ ...(call.hidden ? { secret: true, icon: "key" } : {}), ...(call.value ? { value: call.value } : {}) }) } : {}),
    ...(call.timeout ? { timeout: call.timeout } : {}),
    ...(call.kind === "question" || entry ? { dismiss: call.cancel ?? "Cancel" } : {}),
    ...(call.defaultCancel ? { defaultCancel: true } : {}),
    actions,
    landed,
  };
  return dialog(options);
}

interface Placed {
  readonly landed?: boolean;
  readonly width?: number;
}

/** `call`'s dialog floated in the middle of the dimmed desktop, as Hyprland centres the window. */
function onDesktop(call: Call, placed: Placed = {}): HTMLElement {
  const width = placed.width ?? SCREEN_WIDTH;
  const under = barScreen(width, TALL).element;
  const shown = drawn(call, placed.landed ?? true).element;
  return backdrop({ width: `calc(${width} * var(--px))`, height: TALL, scrim: "dim", content: shown, under });
}

/** The callers the live stage asks in turn. */
const TURNS: readonly Call[] = [CHROME, STEAM_ERROR, SCRIPT_DELETE, PASSWORD, TIMED];

/** The stage: answer the dialog, then click the desktop and the next caller asks. */
function live(): HTMLElement {
  const holder = document.createElement("span");
  holder.style.display = "contents";
  let turn = 0;
  const ask = () => {
    const call = TURNS[turn % TURNS.length]!;
    const under = barScreen(SCREEN_WIDTH, TALL).element;
    const shown = drawn(call, false);
    shown.onAnswer = () => {
      turn += 1;
      const desk = barScreen(SCREEN_WIDTH, TALL).element;
      desk.addEventListener("click", ask, { once: true });
      holder.replaceChildren(desk);
    };
    holder.replaceChildren(backdrop({ width: `calc(${SCREEN_WIDTH} * var(--px))`, height: TALL, scrim: "dim", content: shown.element, under }));
    if (ALL) return;
    requestAnimationFrame(() => shown.land());
  };
  ask();
  return holder;
}

/** What an unsupported mode does: no window, a line on stderr, exit 255. */
function unsupported(): HTMLElement {
  const shown = log(
    [
      { level: "info", text: "$ zenity --list --column=Game --column=Size Hades 15GB" },
      { level: "error", text: "zenity: --list isn't supported by KubaOS's zenity" },
      { level: "info", text: "$ echo $?" },
      { level: "info", text: "255" },
    ],
    // Its four lines whole: the log is `space.s12` in above and below them.
    { height: "calc(4 * var(--type-mono-line) + 2 * var(--space-s12))", unwrapped: true },
  ).box;
  return within("var(--size-dialog)", shown);
}

const DIM = "the dim";
const FADE = "the dialog's fade";
const RISE = "the dialog's rise";

/** The window arriving as polkit's dialog does: the dim and the dialog fade in, the dialog rising. */
function arriving(reduced: boolean): NonEmpty<Track<Easing>> {
  const duration = reduced ? "short" : "long";
  const tracks: Track<Easing>[] = [
    { part: DIM, on: "a question", target: ".k-backdrop-layers", property: "opacity", from: 0, to: 1, duration, curve: "emphasized_decel" },
    { part: FADE, on: "a question", target: ".k-dialog", property: "opacity", from: 0, to: 1, duration, curve: "emphasized_decel" },
  ];
  if (!reduced) tracks.push({ part: RISE, on: "a question", target: ".k-dialog", property: "--dialog-rise", from: 1, to: 0, duration, curve: "emphasized_decel" });
  return nonEmpty(tracks);
}

function leaving(reduced: boolean): NonEmpty<Track<Easing>> {
  const duration = reduced ? "short" : "medium";
  return [
    { part: DIM, on: "a question", target: ".k-backdrop-layers", property: "opacity", from: 1, to: 0, duration, curve: "emphasized_accel" },
    { part: FADE, on: "a question", target: ".k-dialog", property: "opacity", from: 1, to: 0, duration, curve: "emphasized_accel" },
  ];
}

const ARRIVES: Move = {
  name: "Asking",
  what: "An app calls zenity: the desktop dims and the dialog rises in, the keyboard on its default.",
  tracks: arriving(false),
  reduced: arriving(true),
  back: "cuts",
  sample: () => onDesktop(STEAM_QUESTION_CALLER),
};

const LEAVES: Move = {
  name: "Answered",
  what: "Answered, closed or timed out: the dialog and the dim fade off; zenity exits after.",
  tracks: leaving(false),
  reduced: leaving(true),
  back: "cuts",
  sample: () => onDesktop(STEAM_QUESTION_CALLER),
};

const LONG_TITLE: Words = { short: "Steam", long: "Steam - Unable to connect to the Steam network, check your connection" };
const LONG_TEXT: Words = { short: "Done.", long: STEAM_ERROR.text };

const MARKUP: Call = {
  kind: "info",
  app: "notify-update.sh",
  title: "Updates",
  text: 'Installed <b>14 updates</b>, <i>one needs a restart</i>. Notes at <a href="https://kubaos.dev/notes">kubaos.dev/notes</a> &amp; in the log.',
};
const PATH: Call = {
  kind: "error",
  app: "Wine",
  title: "Wine",
  text: "Can't open /home/kuba/.local/share/Steam/steamapps/compatdata/1145360/pfx/drive_c/users/steamuser/AppData/Roaming/Supergiant/Hades/Profile1.sav",
};
const NO_WRAP: Call = {
  kind: "info",
  app: "disk-report.sh",
  title: "Disks",
  text: "nvme0n1   2.0 TB   41% used\nsda       4.0 TB   87% used\nsdb       500 GB    3% used",
  noWrap: true,
};
const LONG_LABELS: Call = { ...CHROME, extra: "Open a new profile instead" };
/** Steam's question under a title that adds to its name, so the caller line shows. */
const STEAM_QUESTION_CALLER: Call = { ...STEAM_QUESTION, title: "Steam - Windows games" };

export default page({
  level: "apps",
  slug: "app-dialogs",
  title: "App dialogs",
  lede: "What KubaOS's `zenity` draws when an app or a script asks a question: the library's Dialog, floated over the desktop.",
  code: { unbuilt: "zenity, a kubaos-ui app" },
  from: ["dialog", "field", "button-bar", "progress-bar", "icon-tile", "log"],
  stage: { live, hint: "answer it; click the desktop and the next app asks", daylight: true },
  variants: [
    { name: "Question", for: "`--question`: OK and Cancel, the caller's labels", sample: () => onDesktop(STEAM_QUESTION) },
    { name: "Chrome's profile in use", for: "the caller's words and labels, as Chrome sends them", sample: () => onDesktop(CHROME) },
    { name: "An extra button", for: "`--extra-button`: Tonal, before OK", sample: () => onDesktop({ ...STEAM_QUESTION, extra: "Always install" }) },
    { name: "Cancel is the default", for: "`--default-cancel`: the keyboard and Enter on Cancel", sample: () => onDesktop(SCRIPT_DELETE) },
    { name: "Information", for: "`--info`: OK alone; `info` in `primary`", sample: () => onDesktop(INFO) },
    { name: "Warning", for: "`--warning`: OK alone; the sign in `warning`", sample: () => onDesktop(WARNING) },
    { name: "Error", for: "`--error`: OK alone; `error` in its role", sample: () => onDesktop(STEAM_ERROR) },
    { name: "Entry", for: "`--entry`: the field, Enter answers OK", sample: () => onDesktop(ENTRY) },
    { name: "Hidden entry", for: "`--hide-text`: a password's field", sample: () => onDesktop(PASSWORD) },
    { name: "Timeout", for: "`--timeout`: the timed bar empties; exit 5", sample: () => onDesktop(TIMED) },
    { name: "Unsupported mode", for: "no window; stderr says why, exit 255", sample: unsupported },
  ],
  sizes: [
    { name: "The shots' screen", for: "1600 by 1000", sample: () => onDesktop(STEAM_QUESTION) },
    { name: "A narrow screen", for: "the dialog keeps `size.dialog`", sample: () => onDesktop(STEAM_QUESTION, { width: NARROW_WIDTH }) },
  ],
  states: {
    shown: ["rest", "lit", "error"],
    absent: {
      pressed: "its buttons press on their own page",
      disabled: "open or gone; a timeout answers by itself",
      selected: "nothing to pick",
      loading: "answers at once; the caller does the work",
      empty: "`--text` may be empty: the title stays",
    },
    rows: [
      { name: "Question", cell: (state) => onDesktop(state === "error" ? STEAM_ERROR : STEAM_QUESTION, { landed: state !== "rest" }) },
      { name: "Entry", cell: (state) => onDesktop(state === "error" ? STEAM_ERROR : ENTRY, { landed: state !== "rest" }) },
    ],
    looks: {
      rest: "the dialog over the dimmed desktop",
      lit: "the keyboard on the default button, or in the field",
      error: "`--error`: the kind's symbol in `error`",
    },
  },
  focus: {
    tubes: [
      { name: "The default button", tone: "accent", filled: true, content: false },
      { name: "The field", tone: "accent", filled: false, content: false },
    ],
    looks: { lit: "as Button's and Field's pages", signsOff: "the tube as glass, no glow" },
    signsOff: (index) => onDesktop(index === 0 ? STEAM_QUESTION : ENTRY),
  },
  content: {
    text: [
      { name: "The title", words: LONG_TITLE, sample: (words) => onDesktop({ ...STEAM_ERROR, title: words, text: "Could not connect." }) },
      { name: "The message", words: LONG_TEXT, sample: (words) => onDesktop({ ...STEAM_ERROR, text: words }) },
    ],
    missingPicture: [
      { name: "A title, the app known", for: "caller line over the title", sample: () => onDesktop(STEAM_QUESTION_CALLER) },
      { name: "No title, the app known", for: "the app's name stands in", sample: () => onDesktop(WINE) },
      { name: "No title, no app", for: "the kind's word stands in", sample: () => onDesktop(NOBODY) },
      { name: "The caller's icon", for: "`--icon-name`: its symbol in the kind's place", sample: () => onDesktop({ ...INFO, icon: "deployed_code" }) },
      { name: "The title names the app", for: "the caller line goes; the title says it", sample: () => onDesktop(STEAM_QUESTION) },
    ],
    counts: [
      { name: "One word", for: "a message of one word", sample: () => onDesktop({ ...INFO, text: "Done." }) },
      { name: "Scrolls inside", for: "a long message: the middle scrolls", sample: () => onDesktop({ ...STEAM_ERROR, text: Array.from({ length: 7 }, () => STEAM_ERROR.text).join("\n\n") }) },
      { name: "A long path", for: "breaks where it must", sample: () => onDesktop(PATH) },
      { name: "Markup", for: "`<b>`, `<i>`, `<a>`, `&amp;`", sample: () => onDesktop(MARKUP) },
      { name: "Long button labels", for: "they stack, OK on top", sample: () => onDesktop(LONG_LABELS) },
      { name: "No wrap", for: "`--no-wrap`: the caller's lines kept", sample: () => onDesktop(NO_WRAP) },
    ],
  },
  motion: {
    arrives: [ARRIVES],
    leaves: [LEAVES],
    between: doesntApply("it doesn't change while open, but for the timed bar."),
  },
  keyboard: {
    keys: [
      { keys: ["Enter"], does: "the default button: OK, or Cancel with `--default-cancel`" },
      { keys: ["Esc"], does: "Cancel, or closes: exit 1" },
      { keys: ["Tab", "Shift+Tab"], does: "field, Cancel, extra, OK, round" },
    ],
    order: "The field, Cancel, the extra button, OK; never out of the dialog.",
    lands: "In the field; else on Cancel with `--default-cancel`; else on OK.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Question", for: "the caller line, the title, the buttons", sample: () => onDesktop(STEAM_QUESTION_CALLER) },
      { name: "Timeout", for: "the timed bar in the asking slot", sample: () => onDesktop(TIMED) },
    ]),
    measures: [
      { part: "Dialog's width", token: "size-dialog" },
      { part: "Dialog's padding", token: "space-s24" },
      { part: "Caller line to title", token: "space-s12" },
      { part: "Caller's tile", token: "size-control_sm" },
      { part: "Title to body", token: "space-s12" },
      { part: "Body to field or bar", token: "space-s16" },
      { part: "Corners", token: "radius-content" },
    ],
    reflow: { note: "The dialog keeps its width, centred; `--width` and `--height` are ignored.", sample: () => onDesktop(STEAM_QUESTION, { width: NARROW_WIDTH }) },
  },
});
