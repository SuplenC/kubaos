// The Command menu's parts as the build draws them (`config/quickshell/modules/menu/`), on the
// ListDetail template's panel (`lib/list-detail.ts`) and the library alone, with the shots'
// stand-in Results (`test/shots/fakes/Kotodama/KotodamaSearch.qml`): the search's head, the jobs
// under it, an app's intro card, the Results and the pane, the footer's keys and words, the Action
// panel, the Toast, and the Views an Extension pushes. The menu's pages build their samples from
// these, each at its build's sizes, by token.

import "./menu.css";
import disc from "../../assets/icon-tile/disc.svg?url";
import plate from "../../assets/icon-tile/plate.svg?url";
import star from "../../assets/icon-tile/star.svg?url";
import wide from "../../assets/icon-tile/wide.svg?url";
import { button } from "../../lib/button.ts";
import { card } from "../../lib/card.ts";
import { checkbox, radio } from "../../lib/check.ts";
import { chip } from "../../lib/chip.ts";
import type { Pinned } from "../../lib/control.ts";
import { descriptionList, type Fact } from "../../lib/description.ts";
import { field, labelled } from "../../lib/field.ts";
import { icon } from "../../lib/icon.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { type KeyHint, keyHints } from "../../lib/keycap.ts";
import { fill, filling, fillingHeight, grid, row, splitColumns, stack } from "../../lib/layout.ts";
import { commandMenu, type MenuMeasures, menuMeasures, type MenuOptions, type Result } from "../../lib/list-detail.ts";
import type { ListRowSpec } from "../../lib/list.ts";
import { menu, type MenuEntry } from "../../lib/menu.ts";
import { type Job, progressPanel } from "../../lib/progress-panel.ts";
import { progressBar } from "../../lib/progress.ts";
import { scroll } from "../../lib/scroll.ts";
import { spinner } from "../../lib/spinner.ts";
import { clamped, text, wrapped } from "../../lib/text.ts";
import { toast } from "../../lib/toast.ts";
import { length } from "../../lib/tokens.ts";
import { popover } from "../../lib/tooltip.ts";

// ---------------------------------------------------------------- the Results --

/** The KubaOS Symbol standing for an Extension's Items with no icon of their own (`words.js`). */
const GLYPHS: Record<string, string> = {
  apps: "desktop_windows",
  calculator: "calculate",
  clipboard: "content_paste",
  emoji: "mood",
  folders: "folder",
  hyprland: "keyboard_command_key",
  system: "settings_power",
  "web-search": "travel_explore",
};

export const glyph = (extension: string) => GLYPHS[extension] ?? "bolt";

/** An Extension's name as a label: `web-search` reads "Web search". */
export function kindLabel(extension: string): string {
  const words = extension.replace(/[-_.]/g, " ");
  return words.charAt(0).toUpperCase() + words.slice(1);
}

/** One Item as the Daemon gives it. */
export interface Item {
  readonly title: string;
  readonly subtitle: string;
  readonly extension: string;
  /** An app's own icon, a stand-in from the library's pictures. */
  readonly picture?: string;
  readonly alias?: string;
  /** Why it can't run, shown in its subtitle's place, the row dimmed. */
  readonly unavailable?: string;
}

/** The shots' Items, in the Daemon's order. */
export const ITEMS: readonly Item[] = [
  { title: "Files", subtitle: "Access and organize files", extension: "apps", picture: plate },
  { title: "Firefox", subtitle: "Web browser", extension: "apps", picture: disc },
  { title: "Document Viewer", subtitle: "View, search or annotate documents in many different formats", extension: "apps", picture: wide },
  { title: "Image Viewer", subtitle: "View images", extension: "apps", picture: star },
  { title: "Settings: Desktop", subtitle: "Shell, wallpaper and the bar", extension: "kubaos" },
  { title: "Timer", subtitle: "Count down and ring", extension: "system" },
  { title: "kubaos", subtitle: "~/Projects/kubaos", extension: "folders" },
  { title: "Downloads", subtitle: "~/Downloads", extension: "folders" },
  { title: "Emoji", subtitle: "Pick an emoji and paste it", extension: "emoji" },
  { title: "Clipboard history", subtitle: "What you copied, newest first", extension: "clipboard" },
  { title: "Power off", subtitle: "Shut the machine down", extension: "system" },
  { title: "Suspend", subtitle: "Sleep until a key is pressed", extension: "system", unavailable: "Inhibited by Firefox: a video is playing" },
  { title: "Workspace 3", subtitle: "Move to workspace 3", extension: "hyprland" },
  { title: "Design specimen", subtitle: "Every token and component, live", extension: "kubaos" },
];

/** What the Action panel lists for every shot Result: the Item's own, then two Contributed. */
export const ACTIONS: Result["actions"] = [
  { title: "Open", keys: "↵" },
  { title: "Open in new window", keys: "⌃↵" },
  { title: "Copy path", keys: "⌃⇧C" },
  { title: "Forget", keys: "⌃⇧⌫" },
  { title: "Pin to the top", keys: "⌃P", group: "Kubaos" },
  { title: "Search the web for it", keys: "", group: "Web search" },
];

/** Twenty Actions, more than the panel has room for, as the long shot lists them. */
export const MANY_ACTIONS: Result["actions"] = Array.from({ length: 20 }, (_, n) => ({
  title: n === 0 ? "Open" : `Action ${n + 1}`,
  keys: n === 0 ? "↵" : n === 1 ? "⌃↵" : "",
  ...(n >= 12 ? { group: "Kubaos" } : {}),
}));

/** The keys a Result takes, in the shots' turn: an app's, then a command's. */
const hintsOf = (index: number): readonly KeyHint[] =>
  index % 2 === 1
    ? [
        ["↵", "Run"],
        ["⌃K", "Actions"],
      ]
    : [
        ["↵", "Open"],
        ["Tab", "Enter app"],
        ["⌃K", "Actions"],
      ];

/** `item` as a Result, its title's match of `typed` lit. */
export function result(item: Item, index: number, typed = ""): Result {
  const at = typed === "" ? -1 : item.title.toLowerCase().indexOf(typed.toLowerCase());
  const row: ListRowSpec = {
    title: item.title,
    ...(at >= 0 ? { matched: [{ start: at, end: at + typed.length }] } : {}),
    subtitle: item.unavailable ?? item.subtitle,
    icon: glyph(item.extension),
    ...(item.picture ? { picture: item.picture } : {}),
    ...(item.alias ? { detail: item.alias } : {}),
    ...(item.unavailable ? { disabled: true } : {}),
    tile: true,
    tall: true,
    clear: true,
    tight: true,
  };
  const kind = [kindLabel(item.extension), item.alias ?? ""].filter((each) => each !== "").join(" · ");
  return { row, kind, actions: ACTIONS, hints: hintsOf(index) };
}

/** The Results for `typed`: the Items whose title holds it, as the stand-in matches. */
export function results(typed = "", items: readonly Item[] = ITEMS): Result[] {
  return items.filter((item) => typed === "" || item.title.toLowerCase().includes(typed.toLowerCase())).map((item, index) => result(item, index, typed));
}

// ---------------------------------------------------------------- the screens --

/** The screens the menu is shown on: the shots' Full HD, a 1080p laptop at 1.25, and the e2e harness's. */
export const SCREENS = {
  full: { width: 1920, height: 1080, name: "1920 × 1080" },
  laptop: { width: 1536, height: 864, name: "1536 × 864" },
  narrow: { width: 1280, height: 800, name: "1280 × 800" },
} as const;
export type ScreenName = keyof typeof SCREENS;

export const measuresOf = (screen: ScreenName = "full"): MenuMeasures => menuMeasures(SCREENS[screen].width, SCREENS[screen].height);

// ---------------------------------------------------------------- the head --

/** The search's field, the keyboard in it unless `pinned` says otherwise. */
function searchField(query: string, placeholder: string, pinned: Pinned | null = "lit"): HTMLElement {
  return field({ placeholder, icon: "search", ...(query ? { value: query } : {}), ...(pinned ? { pinned } : {}) });
}

const PLACEHOLDER = "Search apps, projects and windows. Sums work too";

/** The search's head: the field, with the app it is in as a Selected chip before it. */
export function searchHead(query = "", scope = "", pinned: Pinned | null = "lit"): HTMLElement {
  const parts: Element[] = [];
  if (scope) parts.push(chip({ label: scope, kind: "Selected" }));
  parts.push(filling(searchField(query, scope ? `Search ${scope}` : PLACEHOLDER, pinned)));
  return headRow(...parts);
}

/** Back, Text, out of a pushed View. */
export function back(pinned?: Pinned): HTMLButtonElement {
  return iconButton({ icon: "arrow_back", label: "Back", kind: "Text", ...(pinned ? { pinned } : {}) });
}

/** A list View's head: Back, then its filter, the View's title its placeholder. */
export function filterHead(title: string, typed = "", backPinned?: Pinned): HTMLElement {
  return headRow(back(backPinned), filling(searchField(typed, title)));
}

/** Another View's head: Back, then its title in `subheading`, ending in an ellipsis. */
export function titleHead(title: string, backPinned?: Pinned): HTMLElement {
  const words = clamped(title, "subheading");
  return headRow(back(backPinned), filling(words));
}

/** The head's row: `size.control_md` tall, its parts `space.s8` apart, centred down it. */
function headRow(...parts: Element[]): HTMLElement {
  const head = row("s8", "centre", ...parts);
  head.classList.add("m-head");
  return head;
}

// ---------------------------------------------------------------- the jobs --

/**
 * The jobs still running, under the head, `space.s8` apart: each its state's symbol (a spinner
 * while it can't tell how far it is), its title and what it last said in `small`, and its bar,
 * three quarters and a quarter `space.s12` apart.
 */
export function jobs(list: readonly Job[]): HTMLElement {
  const lines = list.map((job) => {
    const failed = job.state === "failed";
    const known = job.state === "succeeded" || job.progress >= 0;
    const symbol = !failed && !known ? spinner("s16") : icon(failed ? "error" : job.state === "succeeded" ? "check_circle" : "hourglass_top", "var(--icon-s16)", failed ? "var(--color-error)" : "var(--color-primary)");
    const words = clamped(job.message ? `${job.title} · ${job.message}` : job.title, "small", failed ? "error" : "on_surface_variant");
    const said = row("s8", "centre", symbol, filling(words));
    const bar = !failed && job.progress >= 0 ? progressBar({ fraction: job.progress }) : document.createElement("span");
    return splitColumns([3, 1], "s12", [said, bar], { centred: true });
  });
  return stack("s8", "start", ...lines.map(filling));
}

export const JOBS: readonly Job[] = [
  { title: "Update", state: "running", progress: 0.42, message: "Fetching 12 of 28" },
  { title: "Index the disk", state: "running", progress: -1, message: "~/Projects" },
];

// ---------------------------------------------------------------- an app's intro --

export interface Intro {
  readonly note?: string;
  readonly about: string;
  readonly examples: readonly { readonly query: string; readonly result: string }[];
}

/** An app's intro card, before anything is typed in it: its note, what it does, and examples. */
export function introCard(intro: Intro): HTMLElement {
  const examples = intro.examples.map((each) => row("s8", "centre", text(each.query, "mono"), icon("arrow_forward", "var(--icon-s16)", "var(--color-on_surface_variant)"), text(each.result, "small", "on_surface_variant")));
  const words = stack("s8", "start", ...(intro.note ? [wrapped(intro.note, "body", "primary")] : []), wrapped(intro.about, "body"), ...examples);
  return card({ body: [words] });
}

export const LAYOUT_INTRO: Intro = {
  about: "Adds a keyboard layout to the ones you switch between.",
  examples: [
    { query: "german", result: "German, and its variants" },
    { query: "pl", result: "Polish" },
  ],
};

/** The layouts the "layout" app lists once entered, as the stand-in has them. */
export const LAYOUTS: readonly Item[] = [
  { title: "German", subtitle: "de", extension: "kubaos" },
  { title: "German (Switzerland)", subtitle: "ch", extension: "kubaos" },
  { title: "German (Austria)", subtitle: "at", extension: "kubaos" },
  { title: "German (no dead keys)", subtitle: "de(nodeadkeys)", extension: "kubaos" },
  { title: "French", subtitle: "fr", extension: "kubaos" },
  { title: "Polish", subtitle: "pl", extension: "kubaos" },
];

// ---------------------------------------------------------------- the footer --

/** The footer's keys. */
export const footerKeys = (hints: readonly KeyHint[]) => keyHints(hints);

/** A line of the footer's words in `small`: why an Action failed, in `error`, or what is down. */
export function footerWords(words: string, failed: boolean): HTMLElement {
  return filling(clamped(words, "small", failed ? "error" : "on_surface_variant"));
}

/** An Action that asks first: its question in `label` and `error`, then its keys, `space.s12` after. */
export function confirmLine(question: string): HTMLElement {
  return row(
    "s12",
    "centre",
    text(question, "label", "error"),
    keyHints(
      [
        ["↵", "Confirm"],
        ["Esc", "Cancel"],
      ],
      { color: "error" },
    ),
  );
}

/** `left` with the problems the Daemon knows of at the line's end, Text sm, `space.s12` clear. */
export function withProblems(left: HTMLElement, count: number, pinned?: Pinned): HTMLElement {
  const problems = button({ label: count === 1 ? "1 problem" : `${count} problems`, kind: "Text", size: "Sm", ...(pinned ? { pinned } : {}) });
  const line = row("s12", "centre", filling(left), fill(), problems);
  return filling(line);
}

// ---------------------------------------------------------------- the Action panel --

/**
 * The Action panel, Ctrl+K: the selected Result's Actions as a Menu on a popover half the panel
 * wide, the keyboard on `current`; a Contributed group starts after a separator. No taller than the
 * body: past it, it scrolls.
 */
export function actionPanel(actions: Result["actions"], measures: MenuMeasures, current = 0, pinned?: "lit" | "pressed"): HTMLElement {
  const entries: MenuEntry[] = [];
  actions.forEach((action, index) => {
    if (index > 0 && action.group !== actions[index - 1]?.group) entries.push("separator");
    entries.push({ label: action.title, ...(action.keys ? { keys: action.keys } : {}) });
  });
  const at = entries.findIndex((entry, index) => entry !== "separator" && entries.slice(0, index + 1).filter((each) => each !== "separator").length === current + 1);
  const most = Math.floor((measures.body - length("space-s4") * 2) / length("size-control_sm"));
  const list = menu(entries, { current: at, ...(pinned ? { pinned } : {}), most });
  const panel = popover(2, list);
  panel.classList.add("m-actions");
  panel.style.setProperty("--popover-width", `calc(${Math.floor(measures.width / 2 / length("space-s4")) * length("space-s4")} * var(--px))`);
  return panel;
}

// ---------------------------------------------------------------- the Toast --

/** What Forget just did, and the key that undoes it, over the bottom of the list. */
export const forgotToast = (title = "Files") => toast({ words: `Forgot ${title}`, undo: ["⌃Z", "Undo"] });

// ---------------------------------------------------------------- the Views --

export type FieldSpec =
  | { readonly kind: "text"; readonly label: string; readonly placeholder?: string; readonly value?: string; readonly secret?: boolean }
  | { readonly kind: "checkbox"; readonly label: string; readonly value: boolean }
  | { readonly kind: "select"; readonly label: string; readonly options: readonly string[]; readonly value: number };

/** A form's fields as a View sends them: a name, where it goes, two options and a choice. */
export const FIELDS: readonly FieldSpec[] = [
  { kind: "text", label: "Name", placeholder: "Type here" },
  { kind: "text", label: "Folder", placeholder: "~/Projects", value: "~/Projects/kubaos" },
  { kind: "select", label: "Kind", options: ["Rust", "QML", "Nix", "Shell", "Lua"], value: 0 },
  { kind: "checkbox", label: "Open it when done", value: true },
  { kind: "text", label: "Token", placeholder: "Pasted from the site", secret: true, value: "hunter22" },
];

/** Twelve fields, more than the body has room for, as the long shot sends them. */
export const MANY_FIELDS: readonly FieldSpec[] = Array.from({ length: 12 }, (_, n) =>
  n % 4 === 3 ? { kind: "checkbox", label: `Option ${n + 1}`, value: n % 8 === 3 } : { kind: "text", label: `Field ${n + 1}`, placeholder: "Type here" },
);

/** One field: a text field or a select's radios under its label, or a checkbox alone. */
function formField(spec: FieldSpec, lit: boolean): HTMLElement {
  if (spec.kind === "checkbox") return checkbox({ label: spec.label, tick: spec.value ? "Full" : "Empty", ...(lit ? { pinned: "lit" as const } : {}) });
  if (spec.kind === "select") {
    const radios = spec.options.map((label, index) => radio({ label, checked: index === spec.value, ...(lit && index === spec.value ? { pinned: "lit" as const } : {}) }));
    return labelled(spec.label, grid(3, "s12", radios));
  }
  const typed = field({ placeholder: spec.placeholder ?? "", ...(spec.value ? { value: spec.value } : {}), ...(spec.secret ? { secret: true } : {}), ...(lit ? { pinned: "lit" as const } : {}) });
  return labelled(spec.label, typed);
}

/**
 * A form View: its fields in a column `size.column` wide at most, centred, `space.s16` apart, and
 * the submit button, Filled, at the end `space.s24` under them. Past the body it scrolls, reaching
 * into the gaps round it, under the header fade.
 */
export function formView(fields: readonly FieldSpec[], submit: string, lit = 0, down?: number): HTMLElement {
  const column = stack("s24", "start", filling(stack("s16", "start", ...fields.map((each, index) => filling(formField(each, index === lit))))), filling(row(0, "centre", fill(), button({ label: submit, kind: "Filled" }))));
  column.classList.add("m-form");
  const { box } = scroll(column, { height: "100%", above: "s12", below: "s12", fade: "s12", ...(down !== undefined ? { down, fadePinned: down > 0 } : {}) });
  return fillingHeight(box);
}

export interface Detail {
  /** The Markdown, as paragraphs: a `# ` line is a heading. */
  readonly markdown: readonly string[];
  readonly metadata: readonly Fact[];
}

export const DETAIL: Detail = {
  markdown: ["# Firefox 148.0", "A fast, private browser from Mozilla.", "Installed from the Ubuntu archive. Updates come with the system's."],
  metadata: [
    { label: "Version", value: "148.0" },
    { label: "Size", value: "312 MB" },
    { label: "Path", value: "Desktop entry", detail: "/usr/share/applications/firefox.desktop" },
  ],
};

/** A detail View: the Extension's Markdown in `body`, and its metadata `space.s24` under it. */
export function detailView(detail: Detail, down?: number): HTMLElement {
  const words = detail.markdown.map((each) => (each.startsWith("# ") ? wrapped(each.slice(2), "heading") : wrapped(each, "body")));
  const column = stack("s24", "start", filling(stack("s12", "start", ...words.map(filling))), ...(detail.metadata.length > 0 ? [filling(descriptionList(detail.metadata))] : []));
  const { box } = scroll(column, { height: "100%", above: "s12", below: "s12", ...(down !== undefined ? { down } : {}) });
  return fillingHeight(box);
}

/** A progress View: the job followed to its end, centred in the body. */
export function progressView(job: Job): HTMLElement {
  const room = stack(0, "centre", progressPanel(job));
  room.classList.add("m-progress");
  return fillingHeight(room);
}

// ---------------------------------------------------------------- the panel --

export interface Panel extends Partial<Omit<MenuOptions, "screen">> {
  readonly screen?: ScreenName;
}

/** The menu's panel on `screen`, the Results for "fi" unless given. */
export function panel(spec: Panel = {}): HTMLElement {
  const measures = measuresOf(spec.screen);
  const query = spec.query ?? (spec.results ? "" : "fi");
  const { screen: _screen, ...options } = spec;
  return commandMenu({ results: spec.results ?? results(query), ...options, query }, measures);
}

/** The menu with its Action panel open over the Results. */
export function withActions(spec: Panel & { readonly actions?: Result["actions"]; readonly at?: number; readonly pinned?: "lit" | "pressed" } = {}): HTMLElement {
  const measures = measuresOf(spec.screen);
  const over = actionPanel(spec.actions ?? ACTIONS, measures, spec.at ?? 0, spec.pinned);
  // The Action panel holds the keyboard: the field rests.
  const { actions: _actions, at: _at, pinned: _pinned, ...rest } = spec;
  return panel({ head: searchHead(spec.query ?? "fi", "", null), ...rest, over: [over] });
}
