// What KubaOS shows while something is on air: picking what to share, when an app asks (the
// system's share prompt), the bar's sharing and camera pills and the panels they open, and the
// marks on what is shared, which only the person sees, never the stream.
//
// Picking is on the desktop itself, no dialog: every screen dims a little and holds a bar at its
// bottom left, the asking site or app, Share this screen, Region and Cancel; a window is picked by
// pointing at it. Whatever is lit while picking wears the very mark it keeps on air. The bar and
// the window's label are Popovers, Buttons, an IconButton and Tooltips from the library. The
// desktop's windows, the on-air marks and the region's drawing are draft parts, proposed for the
// library. The data is stand-ins: the camera's picture is one of the site's photos.

import "./sharing.css";
import chromium from "../../assets/apps/chromium.svg?url";
import discord from "../../assets/apps/discord.svg?url";
import firefox from "../../assets/apps/firefox.svg?url";
import spotify from "../../assets/apps/spotify.svg?url";
import steam from "../../assets/apps/steam.svg?url";
import vscode from "../../assets/apps/vscode-insiders.svg?url";
import rainy from "../../assets/lock/shibuya-rainy-corner.jpg?url";
import tower from "../../assets/lock/tokyo-tower-blue-hour.jpg?url";
import { button } from "../../lib/button.ts";
import { card } from "../../lib/card.ts";
import { chip } from "../../lib/chip.ts";
import { focusRing, windowRing } from "../../lib/focus.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { iconTile } from "../../lib/icon-tile.ts";
import { filling, inset, row, stack } from "../../lib/layout.ts";
import { list } from "../../lib/list.ts";
import { media } from "../../lib/media.ts";
import { panelPage } from "../../lib/panel-page.ts";
import { nested, role } from "../../lib/radius.ts";
import { hang, screen, sheet } from "../../lib/sheet.ts";
import { tabs } from "../../lib/tabs.ts";
import { centreOnInk, clamped, text } from "../../lib/text.ts";
import { floatingTooltip, keyTooltip, tooltip } from "../../lib/tooltip.ts";
import { mutePill } from "./bar-audio-parts.ts";
import { barScreen, bellPill, type CameraUse, cameraPill, EVERYDAY, type Share, SHARE_ICONS, type ShareKind, sharingPill, statusPill, wallpaper } from "./bar-parts.ts";
import { CORNER, TALL } from "./control-center-parts.ts";
import { live as liven, SCREEN_WIDTH } from "./screen-parts.ts";
import type { Group } from "./workspaces.ts";

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
/** The longest name the bar holds. */
export const LONG_ASKER: Asker = { app: "Chromium", picture: chromium, site: "weekly-design-review.whereby.com" };

/** An app's open windows, by their titles. */
export interface AppWindows {
  readonly app: string;
  readonly picture?: string;
  readonly titles: readonly string[];
}

export const VSCODE: AppWindows = { app: "Visual Studio Code - Insiders", picture: vscode, titles: ["sharing-parts.ts — kubaos"] };
export const FIREFOX: AppWindows = { app: "Firefox", picture: firefox, titles: ["MDN Web Docs — Mozilla Firefox", "The Rust Programming Language — Mozilla Firefox"] };
export const DISCORD_APP: AppWindows = { app: "Discord", picture: discord, titles: ["#design | KubaOS"] };
export const SPOTIFY: AppWindows = { app: "Spotify", picture: spotify, titles: ["Spotify Premium"] };
export const STEAM: AppWindows = { app: "Steam", picture: steam, titles: ["Steam"] };
/** An app with no icon of its own: its initial stands in on the window's tile. */
export const GHOSTTY: AppWindows = { app: "Ghostty", titles: ["~/Projects/kubaos"] };

/** A box on a screen, in logical pixels from its top left. */
export interface Rect {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

/**
 * A window where it is tiled on its screen: its grid lines in the screen's tiling, as Hyprland
 * splits the room inside the frame, so it fills that room at any size of the screen with the same
 * gaps ("1", or "1 / 3" to span two).
 */
export interface Placed {
  readonly app: AppWindows;
  readonly title: string;
  readonly column: string;
  readonly row: string;
}

/**
 * A screen on the desk: its connector, its model, its size in logical pixels and its place among
 * the others, top left at `x`, `y`, and whether it is the laptop's own panel. The connector is
 * never shown: a person reads a screen by where it stands (`placeOf`).
 */
export interface DeskScreen {
  readonly connector: string;
  readonly model: string;
  readonly width: number;
  readonly height: number;
  readonly x: number;
  readonly y: number;
  readonly laptop?: boolean;
}

/** A screen with what picking draws on it: the screen's own pixels to one, and its windows. */
export interface Monitor extends DeskScreen {
  readonly density: number;
  readonly windows: readonly Placed[];
  /** The workspace it shows, where it isn't the first. */
  readonly workspace?: number;
}

/** DP-10's windows: VS Code on the left, Firefox and Discord stacked on the right. */
export const DESK: readonly Placed[] = [
  { app: VSCODE, title: VSCODE.titles[0]!, column: "1", row: "1 / 3" },
  { app: FIREFOX, title: FIREFOX.titles[0]!, column: "2", row: "1" },
  { app: DISCORD_APP, title: DISCORD_APP.titles[0]!, column: "2", row: "2" },
];

export const DP10: Monitor = { connector: "DP-10", model: "Dell U2723QE", width: SCREEN_WIDTH, height: 1000, x: 0, y: 0, density: 1.6, windows: DESK };
/** The laptop's screen beside it, smaller: Spotify and a terminal side by side. */
export const EDP1: Monitor = {
  connector: "eDP-1",
  model: "Built-in display",
  laptop: true,
  width: 1440,
  height: 900,
  x: SCREEN_WIDTH,
  y: 0,
  density: 2,
  windows: [
    { app: SPOTIFY, title: SPOTIFY.titles[0]!, column: "1", row: "1" },
    { app: GHOSTTY, title: GHOSTTY.titles[0]!, column: "2", row: "1" },
  ],
};
/** DP-10 on its third workspace: what the picker shows once the person switches to it. */
export const DP10_THIRD: Monitor = {
  ...DP10,
  workspace: 3,
  windows: [
    { app: FIREFOX, title: FIREFOX.titles[1]!, column: "1", row: "1" },
    { app: STEAM, title: STEAM.titles[0]!, column: "2", row: "1" },
  ],
};
/** The narrowest screen the bar is laid out for, an app's least window wide. */
export const NARROW: Monitor = {
  connector: "HDMI-A-1",
  model: "Dell P2423",
  width: 720,
  height: 480,
  x: 0,
  y: 0,
  density: 1,
  windows: [{ app: FIREFOX, title: FIREFOX.titles[0]!, column: "1", row: "1" }],
};

export const MONITORS: readonly Monitor[] = [DP10, EDP1];

/** The camera's picture: a stand-in for what it sees. */
export const CAMERA_PICTURE = rainy;
export const CAMERA = "Integrated Camera";

/** The modes a camera offers, as it lists them: its resolutions and its frame rates. */
export const RESOLUTIONS = ["1920 × 1080", "1280 × 720", "640 × 480"] as const;
export const FRAME_RATES = ["30 fps", "60 fps"] as const;

/** When each thing went on air, as the clock says it: the shots' 21:47 is now. */
export const SINCE = { sharing: "21:32", camera: "21:30" } as const;

// What a share says, one message each, the names as placeholders.

/** Where a screen stands on the desk, which names it: a person never reads its connector. */
export type Place = "left" | "middle" | "right" | "laptop" | "only";

/**
 * The place of the screen at `index` on `desk`: the only one, the laptop's own panel, or by where
 * it stands among the others from left to right, the first left, the last right, any between middle.
 */
export function placeOf(desk: readonly DeskScreen[], index: number): Place {
  if (desk.length === 1) return "only";
  const screen = desk[index]!;
  if (screen.laptop) return "laptop";
  const order = [...desk].sort((a, b) => a.x - b.x);
  const at = order.indexOf(screen);
  return at === 0 ? "left" : at === order.length - 1 ? "right" : "middle";
}

/**
 * A screen's words by its place, each a whole message: its name, as an object, a region of it as
 * the hint says it, and where a window or a region is on it.
 */
export const SCREEN_WORDS: Record<
  Place,
  { readonly name: string; readonly the: string; readonly regionOf: string; readonly workspace: (workspace: number) => string; readonly region: (size: string) => string }
> = {
  left: { name: "Left screen", the: "the left screen", regionOf: "a region of the left screen", workspace: (n) => `Left screen, workspace ${n}`, region: (size) => `Left screen, ${size}` },
  middle: { name: "Middle screen", the: "the middle screen", regionOf: "a region of the middle screen", workspace: (n) => `Middle screen, workspace ${n}`, region: (size) => `Middle screen, ${size}` },
  right: { name: "Right screen", the: "the right screen", regionOf: "a region of the right screen", workspace: (n) => `Right screen, workspace ${n}`, region: (size) => `Right screen, ${size}` },
  laptop: { name: "Laptop screen", the: "the laptop screen", regionOf: "a region of the laptop screen", workspace: (n) => `Laptop screen, workspace ${n}`, region: (size) => `Laptop screen, ${size}` },
  only: { name: "This screen", the: "this screen", regionOf: "a region of this screen", workspace: (n) => `This screen, workspace ${n}`, region: (size) => `This screen, ${size}` },
};

/** The words for what a share takes, by its kind, as the sharing pill's hint says it. */
export const SHARE_WHAT = {
  screen: (place: Place) => SCREEN_WORDS[place].the,
  window: (app: string) => `${app}'s window`,
  region: (place: Place) => SCREEN_WORDS[place].regionOf,
} as const;

/** A region's size, in the screen's own pixels, as its chip shows it. */
export const regionSize = (width: number, height: number) => `${width} × ${height}`;

// A count's word follows the locale's plural rule, never a letter added by hand.
const PLURAL = new Intl.PluralRules("en-GB");

export const WORDS = {
  title: (name: string) => `Share with ${name}`,
  screen: "Share this screen",
  region: "Region",
  cancel: "Cancel",
  click: "Click to share",
  asked: (app: string) => `${app} asked to share`,
  regionHint: "Or drag with the right button anywhere.",
  withWhom: (who: string, since: string) => `With ${who} since ${since}`,
  toWhom: (who: string, since: string) => `To ${who} since ${since}`,
  windowTitle: (app: string, title: string) => `${app} · ${title}`,
  /** A card's where line with the monitor's model after it, quiet and last: "1280 × 720 · Dell U2723QE". */
  withModel: (where: string, model: string) => `${where} · ${model}`,
  cameras: (count: number) => (PLURAL.select(count) === "one" ? `${count} camera in use` : `${count} cameras in use`),
  held: "Notifications held until you stop",
  change: "Change what's shared",
  stop: "Stop sharing",
  cameraWho: (device: string, since: string) => `${device} since ${since}`,
  close: (app: string) => `Close ${app}`,
  cameraNote: "Only closing it turns the camera off",
} as const;

/** The keys picking takes, besides Tab, Enter and Esc. */
export const PICK_KEYS = { screen: "S", region: "R" } as const;

/** A share as its card shows it: the desk, the screen it is on, and what it takes there. */
export interface ShareView {
  readonly kind: ShareKind;
  readonly desk: readonly DeskScreen[];
  /** The screen it is on, by its index on the desk. */
  readonly screen: number;
  /** A window's or a region's box, in its screen's own pixels. */
  readonly rect?: Rect;
  /** The workspace a shared window is on. */
  readonly workspace?: number;
  readonly app?: AppWindows;
  /** A shared window's title. */
  readonly title?: string;
  readonly who: string;
  readonly site?: string;
  readonly since: string;
}

/** The Share a view gives the bar's pill: what it takes, named from the desk, and with whom. */
export function shareFor(view: ShareView): Share {
  const place = placeOf(view.desk, view.screen);
  const what = view.kind === "screen" ? SHARE_WHAT.screen(place) : view.kind === "window" ? SHARE_WHAT.window(view.app?.app ?? "") : SHARE_WHAT.region(place);
  return { kind: view.kind, what, who: view.who, since: view.since, ...(view.site ? { site: view.site } : {}) };
}

const DELL = "Dell U2723QE";
const BUILT_IN = "Built-in display";

/** Two 16:9 monitors side by side and the laptop on their right, its panel's foot level with theirs. */
export const DESK_THREE: readonly DeskScreen[] = [
  { connector: "DP-3", model: DELL, width: 2560, height: 1440, x: 0, y: 0 },
  { connector: "DP-4", model: DELL, width: 2560, height: 1440, x: 2560, y: 0 },
  { connector: "eDP-1", model: BUILT_IN, laptop: true, width: 1536, height: 960, x: 5120, y: 480 },
];
/** One 21:9 ultrawide and the laptop on its right. */
export const DESK_WIDE: readonly DeskScreen[] = [
  { connector: "DP-1", model: "LG 34WN80C", width: 3440, height: 1440, x: 0, y: 0 },
  { connector: "eDP-1", model: BUILT_IN, laptop: true, width: 1536, height: 960, x: 3440, y: 480 },
];
/** One monitor alone. */
export const DESK_ONE: readonly DeskScreen[] = [{ connector: "DP-1", model: DELL, width: 2560, height: 1440, x: 0, y: 0 }];

/**
 * The three kinds of share on `desk`: the first screen whole to Meet; Firefox's window on workspace
 * 4 of the second screen, or the only one, to Discord; a region of the first screen to Meet.
 */
export function viewsOn(desk: readonly DeskScreen[]): { readonly screen: ShareView; readonly window: ShareView; readonly region: ShareView } {
  const second = Math.min(1, desk.length - 1);
  const on = desk[second]!;
  const first = desk[0]!;
  const meet = { who: MEET.app, site: MEET.site!, since: "21:47" };
  return {
    screen: { kind: "screen", desk, screen: 0, ...meet },
    window: {
      kind: "window",
      desk,
      screen: second,
      workspace: 4,
      app: FIREFOX,
      title: "MDN Web Docs",
      rect: { x: Math.round(on.width * 0.52), y: Math.round(on.height * 0.1), width: Math.round(on.width * 0.45), height: Math.round(on.height * 0.42) },
      who: DISCORD.app,
      since: "21:40",
    },
    region: { kind: "region", desk, screen: 0, rect: { x: Math.round(first.width * 0.14), y: Math.round(first.height * 0.26), width: 1280, height: 720 }, ...meet },
  };
}

export const VIEWS = { three: viewsOn(DESK_THREE), wide: viewsOn(DESK_WIDE), one: viewsOn(DESK_ONE) } as const;

/** A window whose title runs long: the card cuts it with an ellipsis rather than growing. */
export const LONG_VIEW: ShareView = { ...VIEWS.three.window, title: "The Rust Programming Language: Understanding Ownership, References and Borrowing — Mozilla Firefox" };

/** The shares the pills show: the three kinds on the first desk, and a second, Firefox's window to Discord. */
export const SHARES = {
  screen: shareFor(VIEWS.three.screen),
  window: shareFor({ ...VIEWS.three.window, who: MEET.app, site: MEET.site!, since: "21:47" }),
  region: shareFor(VIEWS.three.region),
  second: shareFor(VIEWS.three.window),
} as const satisfies Record<string, Share>;

/** A camera in use as its panel shows it: the app with its icon, the camera, its picture and its mode. */
export interface CameraView {
  readonly use: CameraUse;
  readonly asker: Asker;
  /** What it sees, or none: the Media's placeholder holds its place while it starts. */
  readonly picture: string | null;
  readonly resolution?: number;
  readonly rate?: number;
}

/** The cameras the pills and panels show: the laptop's to Discord, and a second, a webcam to Meet. */
export const CAMERAS = {
  first: { use: { who: DISCORD.app, device: CAMERA, since: SINCE.camera }, asker: DISCORD, picture: CAMERA_PICTURE },
  second: { use: { who: MEET.app, device: "Logitech C920", since: "21:44" }, asker: MEET, picture: tower },
} as const satisfies Record<string, CameraView>;

// Draft parts, proposed for the library: a window on the desktop, the on-air marks, the region's drawing.

/** The size a box's place is a share of: a screen's, in logical pixels. */
export interface Room {
  readonly width: number;
  readonly height: number;
}

/**
 * `rect`'s place and size as shares of `room`, so the box keeps its share of the screen however
 * large the screen is drawn: on the stage, in Full screen, as the window resizes.
 */
function place(element: HTMLElement, rect: Rect, room: Room): void {
  element.style.left = `${(rect.x / room.width) * 100}%`;
  element.style.top = `${(rect.y / room.height) * 100}%`;
  element.style.width = `${(rect.width / room.width) * 100}%`;
  element.style.height = `${(rect.height / room.height) * 100}%`;
}

/**
 * Makes `desk` the screen's tiling: a grid of `columns` by `rows` equal tracks in the room inside
 * the frame, `space.s8` apart and in from it, as Hyprland's gaps; each window names its lines.
 */
export function tiling(desk: HTMLElement, columns: number, rows: number): HTMLElement {
  desk.className = "share-desk";
  desk.style.setProperty("--desk-columns", String(columns));
  desk.style.setProperty("--desk-rows", String(rows));
  return desk;
}

/** How many columns and rows `windows`' lines need. */
function tracks(windows: readonly Placed[]): { readonly columns: number; readonly rows: number } {
  const last = (line: string) => {
    const [start, end] = line.split("/").map((each) => Number.parseInt(each.trim(), 10));
    return end !== undefined && !Number.isNaN(end) ? end - 1 : (start ?? 1);
  };
  return { columns: Math.max(1, ...windows.map((each) => last(each.column))), rows: Math.max(1, ...windows.map((each) => last(each.row))) };
}

export interface DeskWindowOptions extends Placed {
  /** It has the keyboard: its tube lit. */
  readonly focused?: boolean;
  /** It is shared, or lit while picking: its border violet, its tube violet while it is focused. */
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
  drawn.style.gridColumn = options.column;
  drawn.style.gridRow = options.row;
  const tile = iconTile({ name: options.app.app, size: "Sm", ...(options.app.picture ? { picture: options.app.picture } : {}) });
  drawn.append(row("s8", "centre", tile, clamped(options.title, "body_strong", "on_surface")));
  const ring = focusRing(drawn, "surface", options.shared ? "sharing" : "accent", false);
  windowRing(ring, "stroke-thin", corner.css);
  // The violet border, the on-air mark, is always there and shows while the window is shared
  // (`data-shared`), whether or not it has the keyboard: picking lights a window by that attribute.
  const border = document.createElement("span");
  border.className = "share-border";
  ring.append(border);
  drawn.toggleAttribute("data-shared", options.shared ?? false);
  if (options.focused) drawn.dataset.pinned = "lit";
  return drawn;
}

/** Draft part, proposed: a shared region's mark, a line of `sharing` the thin stroke wide just outside it. */
export function regionMark(rect: Rect, room: Room): HTMLElement {
  const mark = document.createElement("span");
  mark.className = "share-region";
  place(mark, rect, room);
  return mark;
}

/**
 * Draft part, proposed: a shared screen's mark, the same as a shared window's and a region's: a
 * line of `sharing_line` the thin stroke wide round the screen's whole edge, inside it, at the
 * screen's own corners. It takes no pointer, and it lies inside the bar's own room above and
 * below its pills, so it covers none of their content.
 */
export function screenMark(): HTMLElement {
  const mark = document.createElement("span");
  mark.className = "share-edge";
  return mark;
}

/** What a share map shows on the desk: the kind, the screen it is on, its box there, a window's workspace. */
export interface MapShare {
  readonly kind: ShareKind;
  readonly screen: number;
  readonly rect?: Rect;
  readonly workspace?: number;
}

/**
 * Draft part, proposed: a map of the desk for a share's card. Every screen a rectangle in its real
 * place and proportions, the whole as wide as the card and no taller than `size.row_three_line`,
 * centred where it is narrower. A screen shared wears `sharing_line` with a faint `sharing` fill;
 * a window or a region is a solid `sharing` box at its real place and size, its screen outlined
 * quietly, and a window's screen carries the window's workspace in its top left corner, where the
 * bar shows it. Nothing else: no picture, no other window.
 */
export function shareMap(desk: readonly DeskScreen[], shared: MapShare): HTMLElement {
  const left = Math.min(...desk.map((each) => each.x));
  const top = Math.min(...desk.map((each) => each.y));
  const wide = Math.max(...desk.map((each) => each.x + each.width)) - left;
  const tall = Math.max(...desk.map((each) => each.y + each.height)) - top;
  const map = document.createElement("span");
  map.className = "share-map";
  map.style.setProperty("--map-ratio", String(wide / tall));
  // Each box's place and size as a share of what holds it, which the CSS turns into lengths.
  const placed = (element: HTMLElement, x: number, y: number, width: number, height: number, ofWidth: number, ofHeight: number) => {
    element.style.setProperty("--x", String((x / ofWidth) * 100));
    element.style.setProperty("--y", String((y / ofHeight) * 100));
    element.style.setProperty("--w", String((width / ofWidth) * 100));
    element.style.setProperty("--h", String((height / ofHeight) * 100));
  };
  desk.forEach((screen, index) => {
    const box = document.createElement("span");
    box.className = "share-map-screen";
    placed(box, screen.x - left, screen.y - top, screen.width, screen.height, wide, tall);
    if (index === shared.screen) {
      box.dataset.shared = shared.kind === "screen" ? "whole" : "part";
      if (shared.rect && shared.kind !== "screen") {
        const taken = document.createElement("span");
        taken.className = "share-map-taken";
        placed(taken, shared.rect.x, shared.rect.y, shared.rect.width, shared.rect.height, screen.width, screen.height);
        box.append(taken);
      }
      if (shared.kind === "window" && shared.workspace !== undefined) {
        const number = text(String(shared.workspace), "label", "on_surface_variant");
        number.classList.add("share-map-workspace");
        box.append(number);
      }
    }
    map.append(box);
  });
  return map;
}

/** A region of the left window, as a share would draw it. */
export const REGION: Rect = { x: 120, y: 200, width: 560, height: 360 };

export interface DesktopOptions {
  /** What is shared: a screen, a window of `DESK` by index, or a region. */
  readonly shared?: { readonly kind: "screen" } | { readonly kind: "window"; readonly index: number } | { readonly kind: "region"; readonly rect?: Rect };
  /** The window of `DESK` with the keyboard. */
  readonly focused?: number;
  readonly camera?: boolean;
}

/** DP-10 on air: its windows, the bar's on-air pills, and the marks on what is shared. */
export function desktop(options: DesktopOptions = {}): HTMLElement {
  const shared = options.shared;
  const share: Share | undefined = shared
    ? shared.kind === "screen"
      ? shareOf({ kind: "screen", screen: 0 }, [DP10])
      : shared.kind === "window"
        ? { kind: "window", what: SHARE_WHAT.window(DESK[shared.index]?.app.app ?? FIREFOX.app), who: MEET.app }
        : shareOf({ kind: "region", screen: 0 }, [DP10])
    : undefined;
  const made = barScreen(SCREEN_WIDTH, TALL, { content: { ...EVERYDAY, ...(share ? { sharing: share } : {}), ...(options.camera ? { camera: { who: DISCORD.app } } : {}) } });
  const desk = tiling(document.createElement("span"), tracks(DESK).columns, tracks(DESK).rows);
  desk.append(...DESK.map((each, index) => deskWindow({ ...each, focused: index === (options.focused ?? 0), shared: shared?.kind === "window" && shared.index === index })));
  made.element.querySelector(".k-screen-dim")!.before(desk);
  if (shared?.kind === "region") made.element.append(regionMark(shared.rect ?? REGION, DP10));
  if (shared?.kind === "screen") made.element.append(screenMark());
  return made.element;
}

export interface DrawOptions {
  /** The box drawn so far, or none before the drag. */
  readonly rect?: Rect;
  /** The screen's size in logical pixels, which the pointer's place is worked out in. */
  readonly width?: number;
  readonly height?: number;
  /** The screen's real pixels to a logical one, for the size the chip says. */
  readonly density?: number;
  /** A drag already under way, from a right-button press: drawing follows it at once. */
  readonly from?: { readonly x: number; readonly y: number };
  readonly onDone?: (rect: Rect) => void;
  readonly onBack?: () => void;
}

/**
 * Draft part, proposed: drawing a region on the screen. Outside the box the screen keeps picking's
 * dim; the box is clear, with the mark it keeps on air just outside it, `sharing` the thin stroke
 * wide, and its size in the screen's pixels in a Neutral chip under its start. A drag draws it,
 * from either button; Enter shares it and Esc goes back to picking.
 */
export function regionDraw(options: DrawOptions = {}): HTMLElement {
  const density = options.density ?? DP10.density;
  const width = options.width ?? DP10.width;
  const height = options.height ?? DP10.height;
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
    place(box, rect, { width, height });
    size.style.left = `${(rect.x / width) * 100}%`;
    size.style.top = `calc(${((rect.y + rect.height) / height) * 100}% + var(--space-s8))`;
    size.replaceChildren(chip({ label: regionSize(Math.round(rect.width * density), Math.round(rect.height * density)), icon: SHARE_ICONS.region }));
  };
  show();
  const logical = (event: PointerEvent) => {
    const bounds = surface.getBoundingClientRect();
    return { x: ((event.clientX - bounds.left) / bounds.width) * width, y: ((event.clientY - bounds.top) / bounds.height) * height };
  };
  // The window takes the drag's moves, so a drag begun before the surface was drawn still lands on it.
  const follow = (from: { x: number; y: number }) => {
    const drag = (moved: PointerEvent) => {
      const to = logical(moved);
      rect = { x: Math.round(Math.min(from.x, to.x)), y: Math.round(Math.min(from.y, to.y)), width: Math.round(Math.abs(to.x - from.x)), height: Math.round(Math.abs(to.y - from.y)) };
      show();
    };
    window.addEventListener("pointermove", drag);
    window.addEventListener("pointerup", () => window.removeEventListener("pointermove", drag), { once: true });
  };
  if (options.from) follow(options.from);
  surface.addEventListener("pointerdown", (event) => {
    event.stopPropagation();
    follow(logical(event));
  });
  surface.addEventListener("contextmenu", (event) => event.preventDefault());
  surface.addEventListener("keydown", (event) => {
    if (event.key === "Enter" && rect && rect.width > 0 && rect.height > 0) options.onDone?.(rect);
    else if (event.key === "Escape") options.onBack?.();
    else return;
    event.preventDefault();
    event.stopPropagation();
  });
  return surface;
}

// Picking.

/** What is lit while picking: nothing, a window, a whole screen, or a region being drawn. */
export type Lit =
  | { readonly kind: "none" }
  | { readonly kind: "window"; readonly screen: number; readonly index: number; readonly keyboard?: boolean }
  | { readonly kind: "screen"; readonly screen: number; readonly keyboard?: boolean }
  | { readonly kind: "region"; readonly screen: number; readonly rect?: Rect; readonly from?: { readonly x: number; readonly y: number } };

export const NOTHING: Lit = { kind: "none" };

/** What a share of `lit` on `monitors` is, as its pill and panel name it. */
export function shareOf(lit: Lit, monitors: readonly Monitor[] = MONITORS, asker: Asker = MEET): Share | undefined {
  if (lit.kind === "none") return undefined;
  const monitor = monitors[lit.screen] ?? monitors[0]!;
  if (lit.kind === "window") return { kind: "window", what: SHARE_WHAT.window(monitor.windows[lit.index]?.app.app ?? ""), who: asker.app };
  const place = placeOf(monitors, lit.screen);
  if (lit.kind === "screen") return { kind: "screen", what: SHARE_WHAT.screen(place), who: asker.app };
  return { kind: "region", what: SHARE_WHAT.region(place), who: asker.app };
}

/** What each part of picking does, for a live stage. */
export interface PickHandlers {
  readonly onLight?: (lit: Lit) => void;
  readonly onShare?: (lit: Lit) => void;
  readonly onCancel?: () => void;
}

export interface PickingOptions {
  readonly asker?: Asker;
  readonly monitors?: readonly Monitor[];
  readonly lit?: Lit;
  /** Picking, or shared: the dim and the bars gone, the mark kept, the pill in every bar. */
  readonly phase?: "picking" | "shared";
  /** Built with its dim off and its bars' Sheets shut, for `open` to bring them in: picking arriving live. */
  readonly arrive?: boolean;
  /**
   * Picked: the pill in the bars while picking's dim and bars are still there, and what was picked
   * wearing its mark, nothing else lit: the moment picking gives way to the share.
   */
  readonly pill?: boolean;
  /** A hint of the bar drawn up, as under the pointer: a sample of it. */
  readonly hint?: "app" | "region";
  readonly handlers?: PickHandlers;
}

/**
 * The bar on one screen, which hangs on a Sheet in the middle of the frame's bottom edge, one row:
 * the asking app's icon, which names it in its Tooltip, then Share this screen, Region and Cancel,
 * each saying its key in its Tooltip. Region's Tooltip carries, in its body under the name, the
 * right-button drag that also draws a region.
 */
export function pickBar(
  asker: Asker,
  options: { lit?: "screen" | "region"; hint?: "app" | "region"; onScreen?: () => void; onPoint?: (on: boolean) => void; onRegion?: () => void; onCancel?: () => void } = {},
): HTMLElement {
  // Who asked, as the system knows it: the app, not the site inside it.
  const app = iconTile({ name: asker.app, size: "Sm", ...(asker.picture ? { picture: asker.picture } : {}) });
  const appTip = tooltip(WORDS.asked(asker.app), "");
  // The tile is no control, so its Tooltip shows on the pointer by its attribute, as a sample shows one.
  app.addEventListener("pointerenter", () => appTip.toggleAttribute("data-shown", true));
  app.addEventListener("pointerleave", () => appTip.toggleAttribute("data-shown", false));
  const share = button({ label: WORDS.screen, kind: "Tonal", size: "Sm", icon: SHARE_ICONS.screen, ...(options.lit === "screen" ? { pinned: "lit" as const } : {}) });
  share.classList.add("share-pick-screen");
  const region = button({ label: WORDS.region, kind: "Text", size: "Sm", icon: SHARE_ICONS.region, ...(options.lit === "region" ? { pinned: "pressed" as const } : {}) });
  region.classList.add("share-pick-region");
  // Its key Tooltip names it, as the other buttons' do, so its own Hint would say it twice.
  const cancel = iconButton({ icon: "close", label: WORDS.cancel, size: "Sm", kind: "Text", hint: false });
  if (options.onScreen) share.addEventListener("click", options.onScreen);
  if (options.onPoint) {
    const point = options.onPoint;
    share.addEventListener("pointerenter", () => point(true));
    share.addEventListener("pointerleave", () => point(false));
  }
  if (options.onRegion) region.addEventListener("click", options.onRegion);
  if (options.onCancel) cancel.addEventListener("click", options.onCancel);
  const line = row(
    "s8",
    "centre",
    floatingTooltip(app, appTip, { side: "above", ...(options.hint === "app" ? { shown: true } : {}) }),
    floatingTooltip(share, keyTooltip(WORDS.screen, PICK_KEYS.screen), { side: "above" }),
    floatingTooltip(region, keyTooltip(WORDS.region, PICK_KEYS.region, WORDS.regionHint), { side: "above", ...(options.hint === "region" ? { shown: true } : {}) }),
    floatingTooltip(cancel, keyTooltip(WORDS.cancel, "Esc"), { side: "above" }),
  );
  // What hangs on the bottom edge's Sheet: `space.s16` inside on every side, as the OSD's strip.
  const bar = inset(line, "s16", "s16");
  bar.classList.add("share-pick-bar");
  return bar;
}

/**
 * The label on a window while picking, built with the window and shown while it is lit: what a
 * click does, and the window's name; Enter, while the keyboard is on it (`data-keyboard`).
 */
export function pickLabel(title: string): HTMLElement {
  const label = keyTooltip(WORDS.click, "Enter", title);
  label.classList.add("share-pick-label");
  return label;
}

/** Workspaces 1 to 3, each with a window: what the bar shows once the person has gone to the third. */
const THREE: readonly Group[] = [{ block: 0, spaces: [{ id: 1, occupied: true }, { id: 2, occupied: true }, { id: 3, occupied: true }] }];

/** Picking's screens, built once, and what lights one of their parts. */
export interface Picking {
  readonly element: HTMLElement;
  /**
   * Shows `lit`: sets the attributes of the windows, the bars' buttons and the screens that
   * change, and draws a region's surface only while one is being drawn. It builds nothing else.
   */
  readonly light: (lit: Lit) => void;
  /** Brings the dim in and opens each bar's Sheet, by the Sheet's own move. */
  readonly open: () => void;
  /** Lifts the dim and shuts each bar's Sheet, the same move played back. */
  readonly close: () => void;
}

/** Sets or removes `name` on `element`, only where that changes it. */
function mark(element: HTMLElement, name: string, on: boolean, value = ""): void {
  if (on ? element.getAttribute(name) === value : !element.hasAttribute(name)) return;
  if (on) element.setAttribute(name, value);
  else element.removeAttribute(name);
}

/**
 * One screen while picking or once shared: its windows, its bar and its mark, built once. Picked,
 * or shared, it is built with what was picked wearing its mark, as the move to sharing plays on.
 */
function pickingScreen(monitor: Monitor, index: number, options: PickingOptions): Picking {
  const asker = options.asker ?? MEET;
  const built = options.lit ?? NOTHING;
  const phase = options.phase ?? "picking";
  // Shared, or picked and giving way to the share: what was picked shows its mark, and nothing lights.
  const picked = phase === "shared" || options.pill === true;
  const share = shareOf(built, options.monitors ?? MONITORS, asker);
  const pill = picked && share ? { sharing: share } : {};
  const made = barScreen(monitor.width, `calc(${monitor.height} * var(--px))`, {
    content: { ...EVERYDAY, ...pill },
    ...(monitor.workspace ? { workspaces: { groups: THREE, active: monitor.workspace } } : {}),
  });
  const element = made.element;
  element.classList.add("share-screen");
  const grid = tracks(monitor.windows);
  const desk = tiling(document.createElement("span"), grid.columns, grid.rows);
  const handlers = options.handlers ?? {};
  const windows = monitor.windows.map((placed, at) => {
    const drawn = deskWindow({ ...placed, focused: false });
    if (!picked) {
      drawn.append(pickLabel(placed.title));
      const window: Lit = { kind: "window", screen: index, index: at };
      // Nothing under the pointer is drawn again as it lights, so entering is enough.
      drawn.addEventListener("pointerenter", () => handlers.onLight?.(window));
      drawn.addEventListener("click", () => handlers.onShare?.(window));
    }
    desk.append(drawn);
    return drawn;
  });
  element.querySelector(".k-screen-dim")!.before(desk);
  element.append(screenMark());
  if (picked) {
    const here = built.kind !== "none" && built.screen === index;
    const at = here && built.kind === "window" ? built.index : -1;
    const window = windows[at];
    if (window) {
      window.toggleAttribute("data-shared", true);
      window.toggleAttribute("data-lifted", true);
      // Still picking as it gives way, the window keeps its label, which the move to sharing fades.
      if (phase === "picking") window.append(pickLabel(monitor.windows[at]!.title));
    }
    mark(element, "data-lit-screen", here && built.kind === "screen");
    if (here && built.kind === "region" && built.rect) element.append(regionMark(built.rect, monitor));
  }
  if (phase === "shared") return { element, light: () => undefined, open: () => undefined, close: () => undefined };

  element.dataset.picking = "";
  if (!options.arrive) element.dataset.open = "";
  let shown: Lit = NOTHING;
  const bar = pickBar(asker, {
    ...(options.hint ? { hint: options.hint } : {}),
    onScreen: () => handlers.onShare?.({ kind: "screen", screen: index }),
    // Leaving the button undoes only its own light, this screen lit whole, never a pick made since.
    onPoint: (on) => {
      if (on) handlers.onLight?.({ kind: "screen", screen: index });
      else if (shown.kind === "screen" && shown.screen === index) handlers.onLight?.(NOTHING);
    },
    onRegion: () => handlers.onLight?.({ kind: "region", screen: index }),
    onCancel: () => handlers.onCancel?.(),
  });
  // The bar is part of the frame: its Sheet hangs in the middle of the bottom edge, on this screen's own frame.
  const barSheet = sheet({ content: bar, edge: "bottom" });
  hang(made, barSheet);
  const shut = options.arrive ? liven(barSheet, false) : undefined;
  const screenButton = bar.querySelector<HTMLElement>(".share-pick-screen")!;
  const regionButton = bar.querySelector<HTMLElement>(".share-pick-region")!;
  if (!picked) {
    // A right-button drag anywhere draws a region at once, from where it was pressed.
    element.addEventListener("contextmenu", (event) => event.preventDefault());
    element.addEventListener("pointerdown", (event) => {
      if (event.button !== 2) return;
      // The press draws the drawing surface, which takes the keyboard; left to its default, the
      // press would then move the keyboard to the page, and Enter would not share the region.
      event.preventDefault();
      const bounds = element.getBoundingClientRect();
      const from = { x: ((event.clientX - bounds.left) / bounds.width) * monitor.width, y: ((event.clientY - bounds.top) / bounds.height) * monitor.height };
      handlers.onLight?.({ kind: "region", screen: index, from });
    });
  }

  let surface: HTMLElement | null = null;
  let drawing = "";
  const light = (lit: Lit) => {
    if (picked) return;
    shown = lit;
    const here = lit.kind !== "none" && lit.screen === index;
    windows.forEach((drawn, at) => {
      const on = here && lit.kind === "window" && lit.index === at;
      mark(drawn, "data-lifted", on);
      mark(drawn, "data-shared", on);
      mark(drawn, "data-keyboard", on && lit.kind === "window" && lit.keyboard === true);
    });
    mark(element, "data-lit-screen", here && lit.kind === "screen");
    mark(screenButton, "data-pinned", here && lit.kind === "screen", "lit");
    mark(regionButton, "data-pinned", here && lit.kind === "region", "pressed");
    // A region being drawn is the one part drawn anew: when drawing starts, or a new drag starts it again.
    const wanted = here && lit.kind === "region" ? JSON.stringify({ rect: lit.rect, from: lit.from }) : "";
    if (wanted === drawing) return;
    drawing = wanted;
    surface?.remove();
    surface = null;
    mark(element, "data-drawing", wanted !== "");
    if (lit.kind !== "region" || !here) return;
    surface = regionDraw({
      ...(lit.rect ? { rect: lit.rect } : {}),
      ...(lit.from ? { from: lit.from } : {}),
      width: monitor.width,
      height: monitor.height,
      density: monitor.density,
      onDone: (rect) => handlers.onShare?.({ kind: "region", screen: index, rect }),
      onBack: () => handlers.onLight?.(NOTHING),
    });
    element.append(surface);
  };
  return {
    element,
    light,
    open: () => {
      mark(element, "data-open", true);
      shut?.(true);
    },
    close: () => {
      mark(element, "data-open", false);
      shut?.(false);
    },
  };
}

/** Every screen side by side, top-aligned as Hyprland lays them out, built once for picking, or shared. */
export function buildPicking(options: PickingOptions = {}): Picking {
  const monitors = options.monitors ?? MONITORS;
  const laid = document.createElement("span");
  laid.className = "share-screens";
  const screens = monitors.map((monitor, index) => pickingScreen(monitor, index, { ...options, monitors }));
  laid.append(...screens.map((each) => each.element));
  return {
    element: laid,
    light: (lit) => screens.forEach((each) => each.light(lit)),
    open: () => screens.forEach((each) => each.open()),
    close: () => screens.forEach((each) => each.close()),
  };
}

/** Picking, or shared, drawn still with `options.lit` lit: a sample. */
export function picking(options: PickingOptions = {}): HTMLElement {
  const made = buildPicking(options);
  made.light(options.lit ?? NOTHING);
  return made.element;
}

// The panels the pills open.

/**
 * The bar over a panel on the screen's corner: the on-air pills, the mute pill, the bell, status.
 * The pill whose panel is open rests: lit, its hint would lie over the panel's title.
 */
function cornerBar(sharing?: readonly Share[], camera?: readonly CameraUse[]): Element[] {
  return [...(sharing ? [sharingPill(sharing)] : []), ...(camera ? [cameraPill(camera)] : []), mutePill({ input: { recording: true } }), bellPill(), statusPill()];
}

/** A panel open on the screen's corner under the bar, drawn still unless `reveal` says how far. */
export function onCorner(panel: HTMLElement, bar: Element[], reveal?: number): HTMLElement {
  const room = screen({ width: CORNER, height: TALL, wallpaper, bar });
  room.element.dataset.open = "";
  hang(room, sheet({ content: panel, edge: "right", ...(reveal !== undefined ? { reveal } : {}) }));
  return room.element;
}

export interface SharingPanelOptions {
  readonly shares?: readonly ShareView[];
  /** Change what's shared, on a share by its index: picking opens again, and its pick replaces the share. */
  readonly onChange?: (index: number) => void;
  /** Stop sharing, on a share by its index. */
  readonly onStop?: (index: number) => void;
}

/**
 * What a share's card says: what is shared, and where it is, the monitor's model last on that
 * line as the quiet detail; a screen's where line is its model alone.
 */
function shareWords(view: ShareView): { readonly title: string; readonly where: string } {
  const place = SCREEN_WORDS[placeOf(view.desk, view.screen)];
  const model = view.desk[view.screen]!.model;
  if (view.kind === "screen") return { title: place.name, where: model };
  if (view.kind === "window") return { title: WORDS.windowTitle(view.app?.app ?? "", view.title ?? ""), where: WORDS.withModel(place.workspace(view.workspace ?? 1), model) };
  return { title: WORDS.region, where: WORDS.withModel(place.region(regionSize(view.rect?.width ?? 0, view.rect?.height ?? 0)), model) };
}

/**
 * One share's card: the desk's map, then what is shared after its tile (a window's app icon, or the
 * kind's symbol), where it is, to whom since when, and its own Change what's shared and Stop sharing. Each line is cut with
 * an ellipsis where it runs past the card, so a long title never grows it.
 */
function shareCard(view: ShareView, index: number, options: SharingPanelOptions): HTMLElement {
  const words = shareWords(view);
  const title = clamped(words.title, "body_strong", "on_surface");
  title.classList.add("share-card-line");
  // Beside a tile the eye sets the title's capitals against the tile's symbol, so every title is
  // centred on a capital's height, whatever its words weigh: the cards' titles sit alike.
  centreOnInk(title, "body_strong", "H");
  // Every card leads with an Sm tile, so the three kinds' titles line up: a window's app icon, or
  // the kind's own symbol, the one its Change what's shared button carries.
  const app = view.kind === "window" && view.app ? view.app : undefined;
  const tile = app ? iconTile({ name: app.app, size: "Sm", ...(app.picture ? { picture: app.picture } : {}) }) : iconTile({ glyph: SHARE_ICONS[view.kind], size: "Sm" });
  const head = row("s8", "centre", tile, title);
  head.classList.add("share-card-line");
  const said = stack(
    "s4",
    "start",
    filling(head),
    filling(clamped(words.where, "small", "on_surface_variant")),
    filling(clamped(WORDS.toWhom(view.site ?? view.who, view.since), "small", "on_surface_variant")),
  );
  const change = button({ label: WORDS.change, kind: "Tonal", size: "Sm", icon: SHARE_ICONS[view.kind] });
  const stop = button({ label: WORDS.stop, kind: "Tonal", size: "Sm", icon: "stop" });
  if (options.onChange) change.addEventListener("click", () => options.onChange?.(index));
  if (options.onStop) stop.addEventListener("click", () => options.onStop?.(index));
  const made = card({
    body: [shareMap(view.desk, { kind: view.kind, screen: view.screen, ...(view.rect ? { rect: view.rect } : {}), ...(view.workspace !== undefined ? { workspace: view.workspace } : {}) }), said],
    actions: row("s8", "centre", change, stop),
  });
  made.classList.add("share-card");
  return made;
}

/**
 * The sharing panel, a PanelPage: that notifications are held under the title, then a Card for
 * each share, saying on a map of the desk and in words what is shared, where it is, and to whom
 * since when, with its own Change what's shared, which opens picking again, and Stop sharing.
 */
export function sharingPanel(options: SharingPanelOptions = {}): HTMLElement {
  const shares = options.shares ?? [VIEWS.three.screen];
  const body = stack("s16", "start", ...shares.map((share, index) => filling(shareCard(share, index, options))));
  const page = panelPage({ title: "Sharing", subtitle: WORDS.held, body });
  page.element.classList.add("share-panel");
  return page.element;
}

export interface CameraPanelOptions {
  readonly cameras?: readonly CameraView[];
  /** Close the app, on a camera by its index. */
  readonly onClose?: (index: number) => void;
}

/**
 * One camera's card: what it sees, live, as a Media; the app that has it, with its icon, in a
 * one-row List; the quality it runs at, its resolution and its frame rate as segmented Tabs of the
 * modes the camera offers; and Close the app, the only way the camera goes off when an app opened
 * it directly.
 */
function cameraCard(view: CameraView, index: number, options: CameraPanelOptions): HTMLElement {
  const preview = media({ corner: role("control"), aspect: "wallpaper", ...(view.picture ? { source: view.picture } : { arrival: "loading" as const }) });
  const app = list([
    {
      title: view.asker.app,
      subtitle: WORDS.cameraWho(view.use.device ?? CAMERA, view.use.since ?? SINCE.camera),
      tile: true,
      icon: "video",
      named: true,
      ...(view.asker.picture ? { picture: view.asker.picture } : { initial: view.asker.app }),
    },
  ]);
  const quality = (name: string, choices: readonly string[], picked: number) =>
    stack("s8", "start", clamped(name, "label", "on_surface_variant"), filling(tabs({ choices: choices.map((label) => ({ label })), picked })));
  const close = button({ label: WORDS.close(view.asker.app), kind: "Tonal", icon: "close", size: "Sm" });
  if (options.onClose) close.addEventListener("click", () => options.onClose?.(index));
  return card({
    body: [
      stack(
        "s16",
        "start",
        filling(preview),
        filling(app),
        filling(quality("Resolution", RESOLUTIONS, view.resolution ?? 0)),
        filling(quality("Frame rate", FRAME_RATES, view.rate ?? 0)),
        text(WORDS.cameraNote, "small", "on_surface_variant"),
      ),
    ],
    actions: row("s8", "centre", close),
  });
}

/** The camera panel, a PanelPage: who has a camera, or how many are in use, then a Card for each camera. */
export function cameraPanel(options: CameraPanelOptions = {}): HTMLElement {
  const cameras = options.cameras ?? [CAMERAS.first];
  const only = cameras.length === 1 ? cameras[0] : undefined;
  const subtitle = only ? WORDS.withWhom(only.asker.app, only.use.since ?? SINCE.camera) : WORDS.cameras(cameras.length);
  const body = stack("s16", "start", ...cameras.map((view, index) => filling(cameraCard(view, index, options))));
  const page = panelPage({ title: "Camera", subtitle, body });
  page.element.classList.add("camera-panel");
  return page.element;
}

/** The sharing panel open under its pill, the camera's pill beside it. */
export function sharingShot(options: SharingPanelOptions = {}, camera = true): HTMLElement {
  return onCorner(sharingPanel(options), cornerBar((options.shares ?? [VIEWS.three.screen]).map(shareFor), camera ? [CAMERAS.first.use] : undefined));
}

/** The camera panel open under its pill, the sharing pill beside it. */
export function cameraShot(options: CameraPanelOptions = {}, sharing = true): HTMLElement {
  return onCorner(cameraPanel(options), cornerBar(sharing ? [SHARES.screen] : undefined, (options.cameras ?? [CAMERAS.first]).map((each) => each.use)));
}
