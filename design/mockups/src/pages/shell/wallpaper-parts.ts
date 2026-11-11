// The wallpaper picker as the build draws it (`config/quickshell/modules/wallpaper/`), from the
// library alone, with stand-in pictures for the shots' wallpaper sets: a bottom-edge sheet holding
// the folders' tabs and the key hints, then the thumbnails, or the settings as the last tab.

import shibuya from "../../assets/lock/shibuya-rainy-corner.jpg?url";
import purple from "../../assets/lock/shinjuku-purple-hour.jpg?url";
import tower from "../../assets/lock/tokyo-tower-blue-hour.jpg?url";
import hero from "../../../../../branding/png/kubaos-hero.jpg?url";
import front from "../../../../../branding/photo/neon-sign-front@2x.jpg?url";
import { choiceGroup } from "../../lib/card.ts";
import type { Pinned } from "../../lib/control.ts";
import { emptyState } from "../../lib/empty-state.ts";
import { keyHints, type KeyHint } from "../../lib/keycap.ts";
import { fill, filling, inset, row, stack } from "../../lib/layout.ts";
import { scaleOf } from "../../lib/list.ts";
import { pictureChoice, setPickedPicture, type Tag } from "../../lib/picture-choice.ts";
import { tabs } from "../../lib/tabs.ts";
import { text, wrapped } from "../../lib/text.ts";
import { raw } from "../../lib/tokens.ts";
import { settings } from "../../site/settings.ts";
import { wallpaper } from "./bar-parts.ts";
import { paddingAround } from "./screen-parts.ts";

export interface Picture {
  readonly name: string;
  readonly source: string;
  /** A moving wallpaper: its badge says so. */
  readonly video?: boolean;
  /** Held loading or unreadable. */
  readonly arrival?: "loading" | "broken";
}

export interface Folder {
  readonly name: string;
  readonly pictures: readonly Picture[];
}

/** The shots' three sets, their names the files' as the build labels them. */
export const FOLDERS: readonly Folder[] = [
  {
    name: "Shinjuku",
    pictures: [
      { name: "shinjuku purple hour", source: purple },
      { name: "kabukicho rain", source: wallpaper },
      { name: "neon sign front", source: front, video: true },
      { name: "shibuya rainy corner", source: shibuya },
      { name: "tokyo tower blue hour", source: tower },
    ],
  },
  {
    name: "Cyberpunk",
    pictures: [
      { name: "shibuya rainy corner", source: shibuya },
      { name: "kubaos hero", source: hero },
      { name: "kabukicho rain", source: wallpaper },
      { name: "shinjuku purple hour", source: purple },
    ],
  },
  {
    name: "Japanese",
    pictures: [
      { name: "shinjuku purple hour", source: purple },
      { name: "tokyo tower blue hour", source: tower },
    ],
  },
];

/** The one applied: the default wallpaper. */
export const APPLIED = wallpaper;

const CURRENT: Tag = { icon: "check", label: "Current" };
/** Up to four a row, at least two, so a folder of few makes a narrower panel. */
const MOST_COLUMNS = 4;
const MOTION = ["Always", "On AC", "Never"] as const;
export type MotionMode = (typeof MOTION)[number];
const HELP: Record<MotionMode, string> = {
  Always: "Always, battery be damned. Loops keep moving off the charger.",
  "On AC": "On AC. Plugged in it moves; on battery it holds still.",
  Never: "Never. Posters only.",
};

export interface PickerOptions {
  /** The tab picked: a folder's index, or the folders' count for the settings. */
  readonly tab?: number;
  readonly folders?: readonly Folder[];
  /** The thumbnail the keyboard is on, by its index; the applied one by default. */
  readonly at?: number;
  /** The keyboard's thumbnail pinned as the specimens pin a state. */
  readonly pinned?: Pinned;
  readonly motion?: MotionMode;
  /** The words of every tab and name, at the toolbar's length. */
  readonly folderName?: string;
  /** Live: the tabs switch, the arrows move, Enter applies, the desktop previews. */
  readonly live?: {
    readonly preview: (source: string) => void;
    readonly close: () => void;
  };
}

/** A folder's tab: its name and how many it holds. */
function tabLabel(folder: Folder): string {
  return `${folder.name} ${folder.pictures.length}`;
}

/** The key hints, as the grid or the settings take the keys. */
function hints(settingsOpen: boolean): HTMLElement {
  const said: readonly KeyHint[] = settingsOpen
    ? [
        ["← →", "change"],
        ["Esc", "back"],
      ]
    : [
        ["Tab", "folder"],
        ["S", "settings"],
        ["Enter", "apply"],
        ["Esc", "close"],
      ];
  return keyHints(said);
}

/** The thumbnails, `columns` across, the applied one marked Current. */
function thumbnails(folder: Folder, options: PickerOptions): HTMLElement {
  const count = folder.pictures.length;
  const columns = Math.max(2, Math.min(MOST_COLUMNS, count));
  const applied = folder.pictures.findIndex((each) => each.source === APPLIED);
  const at = options.at ?? Math.max(0, applied);
  const cells = folder.pictures.map((each, index) =>
    pictureChoice({
      source: each.source,
      name: each.name,
      picked: index === applied,
      mark: CURRENT,
      ...(each.video ? { badge: { icon: index === at ? "motion_photos_on" : "motion_photos_paused", label: index === at ? "Playing" : "Video" } } : {}),
      ...(each.arrival ? { arrival: each.arrival } : {}),
      ...(index === at && options.pinned ? { pinned: options.pinned } : {}),
    }),
  );
  const target = (cell: HTMLElement) => cell.querySelector<HTMLElement>(".k-picture-tile")!;
  const group = choiceGroup("one", columns, cells, { target, set: setPickedPicture, arrowsPick: false });
  group.style.width = `calc(${columns} * var(--size-thumbnail_width) + ${columns - 1} * var(--space-s16))`;
  // The keyboard starts on the applied one, as the picker opens.
  for (const [index, cell] of cells.entries()) target(cell).tabIndex = index === at ? 0 : -1;
  if (options.live) {
    const { preview } = options.live;
    let timer = 0;
    cells.forEach((cell, index) => {
      target(cell).addEventListener("focus", () => {
        window.clearTimeout(timer);
        // The desktop shows the one under the keyboard after a short pause.
        timer = window.setTimeout(() => preview(folder.pictures[index]!.source), Number.parseFloat(raw("duration-short")) / settings().speed);
      });
    });
  }
  return group;
}

/** The settings tab: one row, its name and help, the motion's three choices at its end. */
function motionSettings(mode: MotionMode, live: boolean): HTMLElement {
  let help = wrapped(HELP[mode], "small", "on_surface_variant");
  const words = filling(stack("s4", "start", text("Motion", "body_strong"), help));
  const holder = row("s16", "centre", words);
  const choose = (index: number) => {
    const fresh = wrapped(HELP[MOTION[index]!], "small", "on_surface_variant");
    help.replaceWith(fresh);
    help = fresh;
  };
  const picker = tabs({ choices: MOTION.map((label) => ({ label })), picked: MOTION.indexOf(mode), ...(live ? { onPick: choose } : {}) });
  holder.append(picker);
  return inset(holder, "s12", "s16");
}

/** The picker's content: the header over the thumbnails, the settings or the empty folder. */
export function picker(options: PickerOptions = {}): HTMLElement {
  const folders = options.folders ?? FOLDERS;
  const tab = options.tab ?? 0;
  const settingsOpen = tab === folders.length;
  const labels = [...folders.map((folder, index) => (index === 0 && options.folderName ? tabLabel({ ...folder, name: options.folderName }) : tabLabel(folder))), "Settings"];
  const box = stack("s16", "centre");
  const header = (picked: number) => {
    const bar = tabs({ choices: labels.map((label) => ({ label })), picked, ...(options.live ? { onPick: (index) => show(index) } : {}) });
    return filling(row("s16", "centre", bar, fill(), hints(picked === folders.length)));
  };
  let last = settingsOpen ? 0 : tab;
  const body = (picked: number): HTMLElement => {
    if (picked === folders.length) {
      const view = filling(motionSettings(options.motion ?? "On AC", options.live !== undefined));
      // The settings take the width of four thumbnails, whatever the folder.
      view.style.minWidth = `calc(${MOST_COLUMNS} * var(--size-thumbnail_width) + ${MOST_COLUMNS - 1} * var(--space-s16))`;
      return view;
    }
    const folder = folders[picked]!;
    if (folder.pictures.length === 0) {
      const empty = emptyState({ icon: "wallpaper", title: "Nothing here yet", body: `Drop some images or videos in ~/Pictures/Wallpapers/${folder.name.toLowerCase()}.` });
      empty.style.width = `calc(2 * var(--size-thumbnail_width) + var(--space-s16))`;
      return empty;
    }
    return thumbnails(folder, options);
  };
  function show(picked: number): void {
    if (picked !== folders.length) last = picked;
    box.replaceChildren(header(picked), body(picked));
    if (options.live) requestAnimationFrame(() => land());
  }
  function land(): void {
    const target = box.querySelector<HTMLElement>('.k-picture-tile[tabindex="0"]');
    (target ?? box).focus({ preventScroll: true });
  }
  box.replaceChildren(header(tab), body(tab));
  box.tabIndex = -1;
  // The sides take the container's padding round what it holds, a step for its width.
  const pad = () => {
    const inner = Math.max(...[...box.children].map((child) => (child as HTMLElement).offsetWidth)) / scaleOf(box);
    box.style.paddingInline = `var(--space-${paddingAround(inner)})`;
  };
  box.style.paddingBlock = "var(--space-s24)";
  new ResizeObserver(pad).observe(box);
  if (options.live) {
    const { close } = options.live;
    box.addEventListener("keydown", (event) => {
      const now = box.querySelector('.k-tab[aria-selected="true"]');
      const picked = now ? [...box.querySelectorAll(".k-tab")].indexOf(now) : 0;
      const inSettings = picked === folders.length;
      if (event.key === "Tab") {
        event.preventDefault();
        const n = folders.length + 1;
        show((picked + (event.shiftKey ? -1 : 1) + n) % n);
      } else if (event.key === "s" && !inSettings) {
        event.preventDefault();
        show(folders.length);
      } else if (event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        if (inSettings) show(last);
        else close();
      } else if (inSettings && (event.key === "ArrowLeft" || event.key === "ArrowRight" || event.key === "h" || event.key === "l" || event.key === " " || event.key === "Enter")) {
        event.preventDefault();
        const choices = [...box.querySelectorAll<HTMLElement>(".k-tabs")].at(-1)!.querySelectorAll<HTMLElement>(".k-tab");
        const on = [...choices].findIndex((each) => each.getAttribute("aria-selected") === "true");
        const back = event.key === "ArrowLeft" || event.key === "h";
        choices[(on + (back ? 2 : 1)) % 3]!.click();
      }
    });
  }
  return box;
}

/** Puts the keyboard where it lands as the picker opens: the applied thumbnail. */
export function landIn(content: HTMLElement): void {
  requestAnimationFrame(() => content.querySelector<HTMLElement>('.k-picture-tile[tabindex="0"]')?.focus({ preventScroll: true }));
}
