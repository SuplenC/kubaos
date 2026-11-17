// Apps' dialogs (`zenity`): the program KubaOS installs as `zenity`, so that Chrome, Steam, Wine and
// scripts asking a question get the library's Dialog instead of GNOME's. A draft: it shows the
// modes callers use (`--question`, `--info`, `--warning`, `--error`, `--entry`), their options and
// the text they send, floated and centred over the desktop as Hyprland places the window.
//
// What the library's Dialog lacks is drawn here as draft parts, each marked: the kind's symbol in
// its fixed role, the body's Pango markup, `--no-wrap`, and the line naming the app that asks.

import "./app-dialogs.css";
import { backdrop } from "../../lib/backdrop.ts";
import { iconInLine } from "../../lib/button.ts";
import type { Words } from "../../lib/content.ts";
import { type Action, type DialogOptions, dialog } from "../../lib/dialog.ts";
import { field } from "../../lib/field.ts";
import { iconTile } from "../../lib/icon-tile.ts";
import { gap, row } from "../../lib/layout.ts";
import { log } from "../../lib/log.ts";
import { captionedBar, progressBar } from "../../lib/progress.ts";
import { text } from "../../lib/text.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";
import { ALL } from "../../site/settings.ts";
import { barScreen } from "../shell/bar-parts.ts";
import { NARROW_WIDTH, SCREEN_WIDTH, TALL } from "../shell/screen-parts.ts";

/** zenity's modes this program answers. */
type Kind = "question" | "info" | "warning" | "error" | "entry";

/** How a caller is named on the dialog: the decision this draft puts to Kuba. */
type Identity = "line" | "symbol" | "none";

/** One call of `zenity`, as its options give it. */
interface Call {
  readonly kind: Kind;
  /** `--title`; none, and the kind's own word stands in. */
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

/** The kind's word, its symbol and that symbol's fixed role: no colour of the dialog's own. */
const KINDS: Record<Kind, { readonly word: string; readonly icon: string; readonly role: string }> = {
  question: { word: "Question", icon: "info", role: "primary" },
  info: { word: "Information", icon: "info", role: "primary" },
  warning: { word: "Warning", icon: "warning", role: "warning" },
  error: { word: "Error", icon: "error", role: "error" },
  entry: { word: "Entry", icon: "draw", role: "primary" },
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

/**
 * Draft part: a body in Pango's markup, the subset callers send: `<b>`, `<i>`, `<a href>` and the
 * entities. Anything else shows as written.
 */
function markup(words: string, into: HTMLElement): void {
  const parts: Node[] = [];
  const tags = /<(\/?)(b|i|a)(?:\s+href="([^"]*)")?>/g;
  const stack: HTMLElement[] = [];
  let at = 0;
  const add = (node: Node) => (stack.at(-1) ?? { append: (child: Node) => parts.push(child) }).append(node);
  for (const match of words.matchAll(tags)) {
    if (match.index > at) add(document.createTextNode(unescape(words.slice(at, match.index))));
    at = match.index + match[0].length;
    if (match[1]) stack.pop();
    else {
      const run = document.createElement(match[2] === "a" ? "a" : "span");
      run.className = `app-dialog-${match[2]}`;
      if (match[3]) (run as HTMLAnchorElement).href = match[3];
      add(run);
      stack.push(run);
    }
  }
  if (at < words.length) add(document.createTextNode(unescape(words.slice(at))));
  into.replaceChildren(...parts);
}

function unescape(words: string): string {
  return words.replaceAll("&lt;", "<").replaceAll("&gt;", ">").replaceAll("&quot;", '"').replaceAll("&apos;", "'").replaceAll("&amp;", "&");
}

/** The words `markup` draws, without the tags: what the body is measured by. */
function plain(words: string): string {
  return unescape(words.replaceAll(/<[^>]+>/g, ""));
}

/** Draft part: the line over the title naming the app that asks, its tile and its name. */
function callerLine(app: string): HTMLElement {
  const line = row("s8", "centre", iconTile({ name: app, size: "Sm" }), text(app, "label", "on_surface_variant"));
  line.classList.add("app-dialog-caller");
  return line;
}

/** The library's Dialog for `call`, its draft parts put in. */
function drawn(call: Call, identity: Identity, landed: boolean) {
  const kind = KINDS[call.kind];
  const title = call.title ?? (identity !== "none" && call.app ? call.app : kind.word);
  const asking = call.kind === "entry";
  const actions: Action[] = [
    ...(call.extra ? [{ label: call.extra, kind: "Tonal" as const }] : []),
    { label: call.ok ?? "OK" },
  ];
  const hasCancel = call.kind === "question" || asking;
  const asks = asking
    ? field({ ...(call.hidden ? { secret: true, icon: "key" } : {}), ...(call.value ? { value: call.value } : {}) })
    : call.timeout
      ? captionedBar(progressBar({ fraction: call.timeout.left / call.timeout.of, timed: true }), { words: `Answers by itself in ${call.timeout.left} s`, icon: "timer" })
      : undefined;
  const options: DialogOptions = {
    title,
    body: plain(call.text),
    ...(asks ? { asks } : {}),
    ...(hasCancel ? { dismiss: call.cancel ?? "Cancel" } : {}),
    actions,
    landed: landed && !call.defaultCancel,
  };
  const shown = dialog(options);
  const surface = shown.element;
  surface.classList.add("app-dialog");
  // Draft part: the kind's symbol in its fixed role, or the caller's own, or the app's tile.
  const head = surface.querySelector<HTMLElement>(".k-dialog-column > .k-row")!;
  const symbol = head.firstElementChild!;
  if (identity === "symbol" && call.app) symbol.replaceWith(iconTile({ name: call.app, size: "Sm" }));
  else if (call.icon) symbol.replaceWith(iconInLine(call.icon, "heading", "var(--color-primary)"));
  else symbol.replaceWith(iconInLine(kind.icon, "heading", `var(--color-${kind.role})`));
  if (identity === "line" && call.app) head.before(callerLine(call.app), gap("s12"));
  const body = surface.querySelector<HTMLElement>(".k-dialog-scroll > .k-text")!;
  if (call.text !== plain(call.text)) markup(call.text, body);
  body.toggleAttribute("data-no-wrap", call.noWrap ?? false);
  if (call.defaultCancel) {
    const cancel = surface.querySelector<HTMLElement>(".k-button-bar button")!;
    if (landed) cancel.dataset.pinned = "lit";
    surface.dataset.defaultCancel = "";
  }
  return shown;
}

interface Placed {
  readonly identity?: Identity;
  readonly landed?: boolean;
  readonly dimmed?: boolean;
  readonly width?: number;
}

/** `call`'s dialog floated in the middle of the desktop, as Hyprland centres the window. */
function onDesktop(call: Call, placed: Placed = {}): HTMLElement {
  const width = placed.width ?? SCREEN_WIDTH;
  const under = barScreen(width, TALL).element;
  const shown = drawn(call, placed.identity ?? "line", placed.landed ?? true).element;
  const screen = backdrop({ width: `calc(${width} * var(--px))`, height: TALL, scrim: "dim", content: shown, under });
  screen.toggleAttribute("data-undimmed", placed.dimmed === false);
  return screen;
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
    const shown = drawn(call, "line", false);
    shown.onAnswer = () => {
      turn += 1;
      const desk = barScreen(SCREEN_WIDTH, TALL).element;
      desk.addEventListener("click", ask, { once: true });
      holder.replaceChildren(desk);
    };
    holder.replaceChildren(backdrop({ width: `calc(${SCREEN_WIDTH} * var(--px))`, height: TALL, scrim: "dim", content: shown.element, under }));
    if (ALL) return;
    requestAnimationFrame(() => {
      if (call.defaultCancel) shown.element.querySelector<HTMLElement>(".k-button-bar button")?.focus({ focusVisible: true } as FocusOptions);
      else shown.land();
    });
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
    { height: "var(--size-control_lg)", unwrapped: true },
  ).box;
  shown.classList.add("app-dialog-log");
  return shown;
}

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
    { name: "Not dimmed", for: "decision: the desktop left undimmed", sample: () => onDesktop(STEAM_QUESTION, { dimmed: false }) },
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
      { name: "A title, the app known", for: "caller line over the title", sample: () => onDesktop(STEAM_QUESTION) },
      { name: "No title, the app known", for: "the app's name stands in", sample: () => onDesktop(WINE) },
      { name: "No title, no app", for: "the kind's word stands in", sample: () => onDesktop(NOBODY) },
      { name: "The caller's icon", for: "`--icon-name`: its symbol in the kind's place", sample: () => onDesktop({ ...INFO, icon: "deployed_code" }) },
      { name: "Option B: the app's tile", for: "decision: the tile in the symbol's place", sample: () => onDesktop(STEAM_QUESTION, { identity: "symbol" }) },
      { name: "Option C: the title alone", for: "decision: as zenity does, no app named", sample: () => onDesktop(STEAM_QUESTION, { identity: "none" }) },
    ],
    counts: [
      { name: "One word", for: "a message of one word", sample: () => onDesktop({ ...INFO, text: "Done." }) },
      { name: "Scrolls inside", for: "a long message: the middle scrolls", sample: () => onDesktop({ ...STEAM_ERROR, text: `${STEAM_ERROR.text} ${STEAM_ERROR.text} ${STEAM_ERROR.text}` }) },
      { name: "A long path", for: "breaks where it must", sample: () => onDesktop(PATH) },
      { name: "Markup", for: "`<b>`, `<i>`, `<a>`, `&amp;`", sample: () => onDesktop(MARKUP) },
      { name: "Long button labels", for: "they stack, OK on top", sample: () => onDesktop(LONG_LABELS) },
      { name: "No wrap", for: "`--no-wrap`: the caller's lines kept", sample: () => onDesktop(NO_WRAP) },
    ],
  },
  motion: {
    arrives: doesntApply("as Polkit's: the dim and the dialog fade in, the dialog rising `space.s16`."),
    leaves: doesntApply("as Polkit's: both fade off; zenity exits once they are gone."),
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
      { name: "Question", for: "the caller line, the title, the buttons", sample: () => onDesktop(STEAM_QUESTION) },
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
