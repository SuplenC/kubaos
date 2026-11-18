// What KubaOS shows while something is on air: the picker an app's request to share opens (the
// system's share prompt), the bar's sharing and camera pills and the panels they open, and the
// marks on what is shared, which only the person sees, never the stream.
//
// The picker and the sharing panel hold one choice of what to share, Screen · Window · Region
// (`shareChoice`): the picker asks with it, the panel changes the running share with it. Both are
// assembled from the library: Dialog, Tabs, PictureChoice, ScrollingList, EmptyState, Checkbox,
// PanelPage, Card, Media, Button. The desktop's windows, the marks and the region's drawing are
// draft parts, marked as proposed, waiting for Kuba before they join the library. The data is
// stand-ins: the thumbnails and the camera's picture are the site's photos.

import "./sharing.css";
import kobo from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import chromium from "../../assets/apps/chromium.svg?url";
import discord from "../../assets/apps/discord.svg?url";
import firefox from "../../assets/apps/firefox.svg?url";
import spotify from "../../assets/apps/spotify.svg?url";
import steam from "../../assets/apps/steam.svg?url";
import vscode from "../../assets/apps/vscode-insiders.svg?url";
import rainy from "../../assets/lock/shibuya-rainy-corner.jpg?url";
import purple from "../../assets/lock/shinjuku-purple-hour.jpg?url";
import tower from "../../assets/lock/tokyo-tower-blue-hour.jpg?url";
import { backdrop } from "../../lib/backdrop.ts";
import { button } from "../../lib/button.ts";
import { card } from "../../lib/card.ts";
import { checkbox } from "../../lib/check.ts";
import { chip } from "../../lib/chip.ts";
import { type Dialog, dialog } from "../../lib/dialog.ts";
import { emptyState } from "../../lib/empty-state.ts";
import { focusRing, windowRing } from "../../lib/focus.ts";
import { iconTile } from "../../lib/icon-tile.ts";
import { filling, grid, row, stack } from "../../lib/layout.ts";
import { list, listLabel, type ListItem, scrollingList } from "../../lib/list.ts";
import { media } from "../../lib/media.ts";
import { panelPage } from "../../lib/panel-page.ts";
import { pictureChoice } from "../../lib/picture-choice.ts";
import { nested, role } from "../../lib/radius.ts";
import { hang, screen, sheet } from "../../lib/sheet.ts";
import { tabs } from "../../lib/tabs.ts";
import { clamped, text } from "../../lib/text.ts";
import { mutePill } from "./bar-audio-parts.ts";
import {
  barScreen,
  bellPill,
  type CameraUse,
  cameraPill,
  EVERYDAY,
  type Share,
  SHARE_ICONS,
  type ShareKind,
  sharingPill,
  statusPill,
  wallpaper,
} from "./bar-parts.ts";
import { CORNER, fadeIn, TALL } from "./control-center-parts.ts";
import { SCREEN_WIDTH } from "./screen-parts.ts";

export { kobo };

// The stand-ins.

/** The app asking to share, and the site in it where the app names one. */
export interface Asker {
  readonly app: string;
  readonly picture?: string;
  readonly site?: string;
}

export const MEET: Asker = { app: "Chromium", picture: chromium, site: "meet.google.com" };
export const DISCORD: Asker = { app: "Discord", picture: discord };
/** An app with no icon of its own: its initial stands in. */
export const NO_ICON: Asker = { app: "Jitsi Meet" };
/** The longest name a picker's title holds. */
export const LONG_ASKER: Asker = { app: "Chromium", picture: chromium, site: "weekly-design-review.whereby.com" };

/** A screen a share can take: its connector, its resolution, whether it is the main one, and its picture. */
export interface Display {
  readonly connector: string;
  readonly resolution: string;
  readonly main?: boolean;
  readonly picture: string;
}

export const DISPLAYS: readonly Display[] = [
  { connector: "DP-10", resolution: "2560 × 1440", main: true, picture: tower },
  { connector: "eDP-1", resolution: "2880 × 1800", picture: purple },
];
export const THREE_DISPLAYS: readonly Display[] = [...DISPLAYS, { connector: "HDMI-A-1", resolution: "1920 × 1080", picture: kobo }];

/** An app's open windows, by their titles. */
export interface AppWindows {
  readonly app: string;
  readonly picture?: string;
  readonly titles: readonly string[];
}

export const WINDOWS: readonly AppWindows[] = [
  { app: "Visual Studio Code - Insiders", picture: vscode, titles: ["sharing-parts.ts — kubaos"] },
  { app: "Firefox", picture: firefox, titles: ["MDN Web Docs — Mozilla Firefox", "The Rust Programming Language — Mozilla Firefox"] },
  { app: "Discord", picture: discord, titles: ["#design | KubaOS"] },
  { app: "Spotify", picture: spotify, titles: ["Spotify Premium"] },
  { app: "Steam", picture: steam, titles: ["Steam"] },
  { app: "Ghostty", titles: ["~/Projects/kubaos"] },
];

/** Each open window in the list's order, with its app. */
export function windowsOf(apps: readonly AppWindows[]): { readonly app: AppWindows; readonly title: string }[] {
  return apps.flatMap((app) => app.titles.map((title) => ({ app, title })));
}

/** The camera's picture: a stand-in for what it sees. */
export const CAMERA_PICTURE = rainy;
export const CAMERA = "Integrated Camera";

/** The modes a camera offers, as it lists them: its resolutions and its frame rates. */
export const RESOLUTIONS = ["1920 × 1080", "1280 × 720", "640 × 480"] as const;
export const FRAME_RATES = ["30 fps", "60 fps"] as const;

/** When each thing went on air, as the clock says it: the shots' 21:47 is now. */
export const SINCE = { sharing: "21:32", camera: "21:30" } as const;

// What a share says, one message each, the names as placeholders.

/** The words for what a share takes, by its kind. */
export const SHARE_WHAT = {
  screen: (connector: string) => connector,
  window: (app: string) => `${app}'s window`,
  region: (connector: string) => `A region of ${connector}`,
} as const;

/** The name a screen's tile shows: its connector and resolution. */
export const screenName = (display: Display) => `${display.connector} · ${display.resolution}`;
/** A region's size, as its chip and the panel show it. */
export const regionSize = (width: number, height: number) => `${width} × ${height}`;

export const WORDS = {
  title: (name: string) => `Share with ${name}`,
  body: (app: string) => `${app} sees what you pick until you stop sharing from the bar.`,
  remember: (app: string) => `Let ${app} share again without asking`,
  regionTitle: "Draw on the screen",
  regionBody: "Share hides this; then drag across what to share.",
  regionKept: (size: string, connector: string) => `${size} on ${connector}, drawn last time`,
  withWhom: (who: string, since: string) => `With ${who} since ${since}`,
  held: "Notifications held until you stop",
  instead: "Share instead",
  stop: "Stop sharing",
  cameraWho: (device: string, since: string) => `${device} since ${since}`,
  close: (app: string) => `Close ${app}`,
  cameraNote: "Only closing it turns the camera off",
} as const;

// The choice of what to share.

export const KINDS: readonly { readonly kind: ShareKind; readonly label: string }[] = [
  { kind: "screen", label: "Screen" },
  { kind: "window", label: "Window" },
  { kind: "region", label: "Region" },
];

/** What is picked: the kind shown, and the screen and the window picked in each. */
export interface Pick {
  readonly kind: ShareKind;
  readonly screen: number;
  readonly window: number;
}

export const FIRST: Pick = { kind: "screen", screen: 0, window: 0 };

export interface ChoiceOptions {
  readonly pick?: Partial<Pick>;
  readonly displays?: readonly Display[];
  readonly windows?: readonly AppWindows[];
  /** Drawn with the keyboard in the pane, its pick lit: a picture of it as it opens. */
  readonly keyboard?: boolean;
  /** The region drawn last time, if one was, as the region's pane says it. */
  readonly region?: string;
  /** Each pick, as it is made: a panel switches the share with it at once. */
  readonly onPick?: (pick: Pick) => void;
}

export interface Choice {
  readonly element: HTMLElement;
  /** The pane the keyboard is in: ← → pick in it. */
  readonly pane: HTMLElement;
  readonly picked: () => Pick;
  /** Shows the next kind, or the one before: Tab and Shift+Tab in the picker. */
  readonly switchKind: (by: -1 | 1) => void;
  /** Picks the screen or the window `by` along, round from either end. */
  readonly move: (by: -1 | 1) => void;
}

/**
 * Screen · Window · Region as stretched Tabs over a pane as tall as five list rows, the same height
 * for every kind, so nothing under it moves as the kind changes: the screens as PictureChoices,
 * their live thumbnails, named by connector and resolution, the main one badged; the windows in a
 * ScrollingList, grouped under their app's name, each with its app's icon; a region as an
 * EmptyState saying it is drawn on the screen. The arrows pick in the pane; a pick lands at once.
 */
export function shareChoice(options: ChoiceOptions = {}): Choice {
  const displays = options.displays ?? DISPLAYS;
  const apps = options.windows ?? WINDOWS;
  const windows = windowsOf(apps);
  let pick: Pick = { ...FIRST, ...options.pick };
  let keyboard = options.keyboard ?? false;
  const pane = document.createElement("span");
  pane.className = "share-pane";
  pane.tabIndex = 0;
  const kindIndex = () => KINDS.findIndex((each) => each.kind === pick.kind);
  const well = tabs({
    choices: KINDS.map((each) => ({ label: each.label, icon: SHARE_ICONS[each.kind] })),
    picked: kindIndex(),
    onPick: (index) => {
      pick = { ...pick, kind: KINDS[index]!.kind };
      draw(true);
      options.onPick?.(pick);
    },
  });

  function screens(): HTMLElement {
    const tiles = displays.map((display, index) => {
      const current = index === pick.screen;
      const made = pictureChoice({
        source: display.picture,
        name: screenName(display),
        picked: current,
        ...(display.main ? { badge: { icon: "star", label: "Main" } } : {}),
        ...(current && keyboard ? { pinned: "lit" as const } : {}),
      });
      made.addEventListener("click", () => set({ screen: index }));
      return made;
    });
    const columns = Math.min(Math.max(displays.length, 2), 3);
    const laid = grid(columns, "s16", tiles);
    laid.classList.add("share-screens");
    return laid;
  }

  function windowList(): HTMLElement {
    const items: ListItem[] = [];
    let at = 0;
    for (const [index, app] of apps.entries()) {
      items.push(listLabel(app.app, index > 0));
      for (const title of app.titles) {
        items.push({
          title,
          named: true,
          tile: true,
          icon: "app_window",
          ...(app.picture ? { picture: app.picture } : { initial: app.app }),
          checked: at === pick.window,
        });
        at += 1;
      }
    }
    return scrollingList(items, {
      height: "var(--share-pane)",
      ...(keyboard ? { current: pick.window } : {}),
      onTake: (index) => set({ window: index }),
    });
  }

  function region(): HTMLElement {
    const said = emptyState({ icon: SHARE_ICONS.region, title: WORDS.regionTitle, body: options.region ?? WORDS.regionBody });
    said.classList.add("share-region-pane");
    return said;
  }

  function draw(changed = false): void {
    pane.dataset.kind = pick.kind;
    pane.replaceChildren(pick.kind === "screen" ? screens() : pick.kind === "window" ? windowList() : region());
    if (changed) fadeIn(pane.firstElementChild as HTMLElement);
  }

  function set(change: Partial<Pick>): void {
    pick = { ...pick, ...change };
    draw();
    options.onPick?.(pick);
  }

  function switchKind(by: -1 | 1): void {
    // The Tabs' own arrows send the pick along, its stretch travelling as a pick by hand does.
    well.querySelector<HTMLElement>('.k-tab[aria-selected="true"]')?.dispatchEvent(new KeyboardEvent("keydown", { key: by > 0 ? "ArrowRight" : "ArrowLeft", bubbles: true }));
    pane.focus({ preventScroll: true });
  }

  function move(by: -1 | 1): void {
    if (pick.kind === "screen") set({ screen: (pick.screen + by + displays.length) % displays.length });
    else if (pick.kind === "window") set({ window: (pick.window + by + windows.length) % windows.length });
  }

  pane.addEventListener("focus", () => {
    keyboard = true;
    draw();
  });
  pane.addEventListener("blur", () => {
    keyboard = false;
    draw();
  });
  pane.addEventListener("keydown", (event) => {
    const by = event.key === "ArrowLeft" || event.key === "ArrowUp" ? -1 : event.key === "ArrowRight" || event.key === "ArrowDown" ? 1 : 0;
    if (by === 0) return;
    event.preventDefault();
    event.stopPropagation();
    move(by);
  });
  draw();
  const element = stack("s12", "start", filling(well), filling(pane));
  element.classList.add("share-choice");
  return { element, pane, picked: () => pick, switchKind, move };
}

// The picker.

export interface PickerOptions extends ChoiceOptions {
  readonly asker?: Asker;
  /** The box to share again without asking, ticked. */
  readonly remember?: boolean;
}

export interface Picker {
  readonly dialog: Dialog;
  readonly choice: Choice;
  /** Puts the keyboard in the pane, as the picker opens. */
  readonly land: () => void;
}

/**
 * The share prompt: a Dialog with the asking app on its caller line and the site, or the app, in
 * its title; the choice, then the box to let the app share again without asking; Cancel and Share.
 * The keyboard lands in the pane: ← → pick, Tab and Shift+Tab switch the kind, Enter shares and Esc
 * cancels, from anywhere in it.
 */
export function picker(options: PickerOptions = {}): Picker {
  const asker = options.asker ?? MEET;
  const choice = shareChoice(options);
  const remember = checkbox({ label: WORDS.remember(asker.app), tick: options.remember ? "Full" : "Empty" });
  const asks = stack("s16", "start", filling(choice.element), remember);
  const made = dialog({
    icon: "desktop_windows",
    caller: { name: asker.app, ...(asker.picture ? { picture: asker.picture } : {}) },
    title: WORDS.title(asker.site ?? asker.app),
    body: WORDS.body(asker.app),
    asks,
    dismiss: "Cancel",
    actions: [{ label: "Share" }],
  });
  made.element.classList.add("share-picker");
  // Tab switches the kind rather than walking the buttons: the picker is answered by Enter and Esc.
  // Caught on the way down, before the Dialog's own Tab walks its stops.
  made.element.addEventListener(
    "keydown",
    (event) => {
      if (event.key !== "Tab") return;
      event.preventDefault();
      event.stopPropagation();
      choice.switchKind(event.shiftKey ? -1 : 1);
    },
    { capture: true },
  );
  return { dialog: made, choice, land: () => choice.pane.focus({ focusVisible: true, preventScroll: true } as FocusOptions) };
}

/** The picker over the whole screen, the desktop and its bar dimmed under it. */
export function pickerScreen(options: PickerOptions = {}, width = SCREEN_WIDTH): HTMLElement {
  const under = barScreen(width, TALL).element;
  return backdrop({ width: `calc(${width} * var(--px))`, height: TALL, scrim: "dim", content: picker({ keyboard: true, ...options }).dialog.element, under });
}

/** What a pick shares, as the pill and the panel name it. */
export function shareOf(pick: Pick, who: Asker, options: { displays?: readonly Display[]; windows?: readonly AppWindows[] } = {}): Share {
  const displays = options.displays ?? DISPLAYS;
  const windows = windowsOf(options.windows ?? WINDOWS);
  const display = displays[pick.screen] ?? displays[0]!;
  const what =
    pick.kind === "screen" ? SHARE_WHAT.screen(display.connector) : pick.kind === "window" ? SHARE_WHAT.window((windows[pick.window] ?? windows[0]!).app.app) : SHARE_WHAT.region(display.connector);
  return { kind: pick.kind, what, who: who.app };
}

// The panels the pills open.

/** The bar over a panel on the screen's corner: the on-air pills, the mute pill, the bell, status. */
function cornerBar(sharing?: Share, camera?: CameraUse, open?: "sharing" | "camera"): Element[] {
  return [
    ...(sharing ? [sharingPill(sharing, open === "sharing" ? "lit" : "rest")] : []),
    ...(camera ? [cameraPill(camera, open === "camera" ? "lit" : "rest")] : []),
    mutePill(),
    bellPill(),
    statusPill(),
  ];
}

/** A panel open on the screen's corner under the bar, drawn still unless `reveal` says how far. */
export function onCorner(panel: HTMLElement, bar: Element[], reveal?: number): HTMLElement {
  const room = screen({ width: CORNER, height: TALL, wallpaper, bar });
  room.element.dataset.open = "";
  hang(room, sheet({ content: panel, edge: "right", ...(reveal !== undefined ? { reveal } : {}) }));
  return room.element;
}

export interface SharingPanelOptions {
  readonly pick?: Partial<Pick>;
  readonly asker?: Asker;
  readonly keyboard?: boolean;
  /** Each pick switches the running share at once. */
  readonly onPick?: (pick: Pick) => void;
  readonly onStop?: () => void;
}

/**
 * The sharing panel, a PanelPage: who it is shared with and since when under the title; a Card
 * saying what is shared and that notifications are held; the same choice the picker asks with,
 * which switches the share as it is picked; and Stop sharing in its footer.
 */
export function sharingPanel(options: SharingPanelOptions = {}): HTMLElement {
  const asker = options.asker ?? MEET;
  let pick: Pick = { ...FIRST, ...options.pick };
  const said = document.createElement("span");
  said.className = "share-now";
  const tell = () => {
    const share = shareOf(pick, asker);
    said.replaceChildren(card({ header: { glyph: SHARE_ICONS[share.kind], title: share.what, subtitle: WORDS.held } }));
  };
  tell();
  const choice = shareChoice({
    pick,
    ...(options.keyboard ? { keyboard: true } : {}),
    onPick: (next) => {
      pick = next;
      tell();
      options.onPick?.(next);
    },
  });
  const instead = stack("s8", "start", clamped(WORDS.instead, "label", "on_surface_variant"), filling(choice.element));
  const body = stack("s16", "start", filling(said), filling(instead));
  const stop = button({ label: WORDS.stop, kind: "Tonal", icon: "stop", size: "Sm" });
  if (options.onStop) stop.addEventListener("click", options.onStop);
  const page = panelPage({ title: "Sharing", subtitle: WORDS.withWhom(asker.site ?? asker.app, SINCE.sharing), body, footer: [stop] });
  page.element.classList.add("share-panel");
  return page.element;
}

export interface CameraPanelOptions {
  readonly who?: Asker;
  readonly resolution?: number;
  readonly rate?: number;
  /** The camera's picture, or none: the Media's placeholder holds its place while it starts. */
  readonly picture?: string | null;
  readonly onClose?: () => void;
}

/**
 * The camera panel, a PanelPage: what the camera sees, live, as a Media; the app that has it, with
 * its icon, in a one-row List; the quality it runs at, its resolution and its frame rate as
 * segmented Tabs of the modes the camera offers; and Close the app in its footer, the only way the
 * camera goes off when an app opened it directly.
 */
export function cameraPanel(options: CameraPanelOptions = {}): HTMLElement {
  const who = options.who ?? DISCORD;
  const picture = options.picture === undefined ? CAMERA_PICTURE : options.picture;
  const preview = media({ corner: role("control"), aspect: "wallpaper", ...(picture ? { source: picture } : { arrival: "loading" as const }) });
  const app = list([{ title: who.app, subtitle: WORDS.cameraWho(CAMERA, SINCE.camera), tile: true, icon: "video", named: true, ...(who.picture ? { picture: who.picture } : { initial: who.app }) }]);
  const quality = (name: string, choices: readonly string[], picked: number) =>
    stack("s8", "start", clamped(name, "label", "on_surface_variant"), filling(tabs({ choices: choices.map((label) => ({ label })), picked })));
  const settings = card({ body: [stack("s16", "start", filling(quality("Resolution", RESOLUTIONS, options.resolution ?? 0)), filling(quality("Frame rate", FRAME_RATES, options.rate ?? 0)))] });
  const body = stack("s16", "start", filling(preview), filling(app), filling(settings), text(WORDS.cameraNote, "small", "on_surface_variant"));
  const close = button({ label: WORDS.close(who.app), kind: "Tonal", icon: "close", size: "Sm" });
  if (options.onClose) close.addEventListener("click", options.onClose);
  const page = panelPage({ title: "Camera", subtitle: WORDS.withWhom(who.app, SINCE.camera), body, footer: [close] });
  page.element.classList.add("camera-panel");
  return page.element;
}

/** The sharing panel open under its pill, the camera's pill beside it. */
export function sharingShot(options: SharingPanelOptions = {}, camera = true): HTMLElement {
  const pick = { ...FIRST, ...options.pick };
  return onCorner(sharingPanel(options), cornerBar(shareOf(pick, options.asker ?? MEET), camera ? { who: DISCORD.app } : undefined, "sharing"));
}

/** The camera panel open under its pill, the sharing pill beside it. */
export function cameraShot(options: CameraPanelOptions = {}, sharing = true): HTMLElement {
  return onCorner(cameraPanel(options), cornerBar(sharing ? shareOf(FIRST, MEET) : undefined, { who: (options.who ?? DISCORD).app }, "camera"));
}

// Draft parts, proposed for the library: a window on the desktop, the on-air marks, the region's drawing.

/** A box on the screen, in logical pixels from its top left. */
export interface Rect {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

/** A box's place and size as the CSS draws it at the scale picked. */
function place(element: HTMLElement, rect: Rect): void {
  element.style.left = `calc(${rect.x} * var(--px))`;
  element.style.top = `calc(${rect.y} * var(--px))`;
  element.style.width = `calc(${rect.width} * var(--px))`;
  element.style.height = `calc(${rect.height} * var(--px))`;
}

export interface DeskWindowOptions {
  readonly app: AppWindows;
  readonly title: string;
  readonly rect: Rect;
  /** It has the keyboard: its tube lit. */
  readonly focused?: boolean;
  /** It is shared: its border violet, its tube violet while it is focused. */
  readonly shared?: boolean;
}

/**
 * Draft part, proposed: an app's window on the desktop, as Hyprland draws it: `surface`, its corner
 * the frame's less its gap, its app's tile and title, its border and the tube of the window that
 * has the keyboard (`windowRing`), the window border the thin stroke wide. The Focus page draws
 * one of its own; this would be the library's, for every desktop scene.
 */
export function deskWindow(options: DeskWindowOptions): HTMLElement {
  const corner = nested(role("surface"), "s8");
  const drawn = document.createElement("span");
  drawn.className = "share-window";
  drawn.style.setProperty("--corner", corner.css);
  place(drawn, options.rect);
  const tile = iconTile({ name: options.app.app, size: "Sm", ...(options.app.picture ? { picture: options.app.picture } : {}) });
  drawn.append(row("s8", "centre", tile, clamped(options.title, "body_strong", "on_surface")));
  const ring = focusRing(drawn, "surface", options.shared ? "sharing" : "accent", false);
  windowRing(ring, "stroke-thin", corner.css);
  if (options.shared) {
    // The violet border stays whether or not the window has the keyboard: it is the on-air mark.
    const border = document.createElement("span");
    border.className = "share-border";
    ring.append(border);
    drawn.dataset.shared = "";
  }
  if (options.focused) drawn.dataset.pinned = "lit";
  return drawn;
}

/** Draft part, proposed: a shared region's mark, a line of `sharing` the thin stroke wide just outside it. */
export function regionMark(rect: Rect): HTMLElement {
  const mark = document.createElement("span");
  mark.className = "share-region";
  place(mark, rect);
  return mark;
}

/**
 * Draft part, proposed: a shared screen's marks, four squares of `sharing` a privacy dot's side in
 * the screen's own outer corners, square, over the frame's rounded hole.
 */
export function screenMarks(): HTMLElement {
  const marks = document.createElement("span");
  marks.className = "share-corners";
  for (const at of ["top-left", "top-right", "bottom-left", "bottom-right"]) {
    const square = document.createElement("span");
    square.className = "share-corner";
    square.dataset.at = at;
    marks.append(square);
  }
  return marks;
}

/** The desktop's windows: VS Code on the left, Firefox and Discord stacked on the right. */
export const DESK: readonly { readonly app: AppWindows; readonly title: string; readonly rect: Rect }[] = [
  { app: WINDOWS[0]!, title: WINDOWS[0]!.titles[0]!, rect: { x: 16, y: 48, width: 780, height: 936 } },
  { app: WINDOWS[1]!, title: WINDOWS[1]!.titles[0]!, rect: { x: 804, y: 48, width: 780, height: 464 } },
  { app: WINDOWS[2]!, title: WINDOWS[2]!.titles[0]!, rect: { x: 804, y: 520, width: 780, height: 464 } },
];

/** A region of the left window, as a share would draw it. */
export const REGION: Rect = { x: 120, y: 200, width: 560, height: 360 };

export interface DesktopOptions {
  /** What is shared: a screen, a window of `DESK` by index, or a region. */
  readonly shared?: { readonly kind: "screen" } | { readonly kind: "window"; readonly index: number } | { readonly kind: "region"; readonly rect?: Rect };
  /** The window of `DESK` with the keyboard. */
  readonly focused?: number;
  readonly camera?: boolean;
}

/** The desktop with its windows, the bar's on-air pills, and the marks on what is shared. */
export function desktop(options: DesktopOptions = {}): HTMLElement {
  const shared = options.shared;
  const share: Share | undefined = shared
    ? shared.kind === "screen"
      ? { kind: "screen", what: SHARE_WHAT.screen("DP-10"), who: MEET.app }
      : shared.kind === "window"
        ? { kind: "window", what: SHARE_WHAT.window(DESK[shared.index]!.app.app), who: MEET.app }
        : { kind: "region", what: SHARE_WHAT.region("DP-10"), who: MEET.app }
    : undefined;
  const made = barScreen(SCREEN_WIDTH, TALL, { content: { ...EVERYDAY, ...(share ? { sharing: share } : {}), ...(options.camera ? { camera: { who: DISCORD.app } } : {}) } });
  const desk = document.createElement("span");
  desk.className = "share-desk";
  desk.append(
    ...DESK.map((each, index) =>
      deskWindow({ ...each, focused: index === (options.focused ?? 0), shared: shared?.kind === "window" && shared.index === index }),
    ),
  );
  made.element.querySelector(".k-screen-dim")!.before(desk);
  if (shared?.kind === "region") made.element.append(regionMark(shared.rect ?? REGION));
  if (shared?.kind === "screen") made.element.append(screenMarks());
  return made.element;
}

export interface DrawOptions {
  /** The box drawn so far, or none before the drag. */
  readonly rect?: Rect;
  /** The screen's real pixels to a logical one, for the size the chip says. */
  readonly density?: number;
  readonly onDone?: (rect: Rect) => void;
  readonly onBack?: () => void;
}

/**
 * Draft part, proposed: drawing a region on the screen. The screen dims as under a dialog, the
 * box being drawn clear of the dim with the accent's line round it and its size, in the screen's
 * pixels, in a Neutral chip under its start; a drag draws it, Enter shares it and Esc goes back.
 */
export function regionDraw(options: DrawOptions = {}): HTMLElement {
  const density = options.density ?? 1.6;
  const surface = document.createElement("span");
  surface.className = "share-draw";
  surface.tabIndex = 0;
  const box = document.createElement("span");
  box.className = "share-draw-box";
  const size = document.createElement("span");
  size.className = "share-draw-size";
  surface.append(box, size);
  let rect = options.rect;
  const show = () => {
    surface.toggleAttribute("data-drawn", rect !== undefined);
    if (!rect) return;
    place(box, rect);
    size.style.left = `calc(${rect.x} * var(--px))`;
    size.style.top = `calc(${rect.y + rect.height} * var(--px) + var(--space-s8))`;
    size.replaceChildren(chip({ label: regionSize(Math.round(rect.width * density), Math.round(rect.height * density)), icon: SHARE_ICONS.region }));
  };
  show();
  const logical = (event: PointerEvent) => {
    const bounds = surface.getBoundingClientRect();
    return { x: ((event.clientX - bounds.left) / bounds.width) * SCREEN_WIDTH, y: ((event.clientY - bounds.top) / bounds.height) * (SCREEN_WIDTH * (bounds.height / bounds.width)) };
  };
  surface.addEventListener("pointerdown", (event) => {
    const from = logical(event);
    surface.setPointerCapture(event.pointerId);
    const drag = (moved: PointerEvent) => {
      const to = logical(moved);
      rect = { x: Math.round(Math.min(from.x, to.x)), y: Math.round(Math.min(from.y, to.y)), width: Math.round(Math.abs(to.x - from.x)), height: Math.round(Math.abs(to.y - from.y)) };
      show();
    };
    surface.addEventListener("pointermove", drag);
    surface.addEventListener("pointerup", () => surface.removeEventListener("pointermove", drag), { once: true });
  });
  surface.addEventListener("keydown", (event) => {
    if (event.key === "Enter" && rect && rect.width > 0 && rect.height > 0) options.onDone?.(rect);
    else if (event.key === "Escape") options.onBack?.();
    else return;
    event.preventDefault();
    event.stopPropagation();
  });
  return surface;
}

/** The region's drawing over the desktop, the box drawn, or none before the drag. */
export function drawingScreen(rect: Rect | null = REGION): HTMLElement {
  const made = desktop();
  made.append(regionDraw(rect ? { rect } : {}));
  return made;
}
