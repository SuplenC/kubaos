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
import { tipLine } from "../../lib/banner.ts";
import { button } from "../../lib/button.ts";
import { card } from "../../lib/card.ts";
import { chip } from "../../lib/chip.ts";
import { focusRing, windowRing } from "../../lib/focus.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { iconTile } from "../../lib/icon-tile.ts";
import { keycap } from "../../lib/keycap.ts";
import { filling, row, stack } from "../../lib/layout.ts";
import { list } from "../../lib/list.ts";
import { media } from "../../lib/media.ts";
import { panelPage } from "../../lib/panel-page.ts";
import { nested, role } from "../../lib/radius.ts";
import { hang, screen, sheet } from "../../lib/sheet.ts";
import { tabs } from "../../lib/tabs.ts";
import { clamped, text } from "../../lib/text.ts";
import { floatingTooltip, keyTooltip, popover, tooltip } from "../../lib/tooltip.ts";
import { mutePill } from "./bar-audio-parts.ts";
import { barScreen, bellPill, type CameraUse, cameraPill, EVERYDAY, type Share, SHARE_ICONS, sharingPill, statusPill, wallpaper } from "./bar-parts.ts";
import { CORNER, TALL } from "./control-center-parts.ts";
import { SCREEN_WIDTH } from "./screen-parts.ts";
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

/** A window where it lies on its screen. */
export interface Placed {
  readonly app: AppWindows;
  readonly title: string;
  readonly rect: Rect;
}

/** A screen: its connector, its size in logical pixels, the screen's own pixels to one, and its windows. */
export interface Monitor {
  readonly connector: string;
  readonly width: number;
  readonly height: number;
  readonly density: number;
  readonly windows: readonly Placed[];
  /** The workspace it shows, where it isn't the first. */
  readonly workspace?: number;
}

/** DP-10's windows: VS Code on the left, Firefox and Discord stacked on the right. */
export const DESK: readonly Placed[] = [
  { app: VSCODE, title: VSCODE.titles[0]!, rect: { x: 16, y: 48, width: 780, height: 936 } },
  { app: FIREFOX, title: FIREFOX.titles[0]!, rect: { x: 804, y: 48, width: 780, height: 464 } },
  { app: DISCORD_APP, title: DISCORD_APP.titles[0]!, rect: { x: 804, y: 520, width: 780, height: 464 } },
];

export const DP10: Monitor = { connector: "DP-10", width: SCREEN_WIDTH, height: 1000, density: 1.6, windows: DESK };
/** The laptop's screen beside it, smaller: Spotify and a terminal side by side. */
export const EDP1: Monitor = {
  connector: "eDP-1",
  width: 1440,
  height: 900,
  density: 2,
  windows: [
    { app: SPOTIFY, title: SPOTIFY.titles[0]!, rect: { x: 16, y: 48, width: 700, height: 836 } },
    { app: GHOSTTY, title: GHOSTTY.titles[0]!, rect: { x: 724, y: 48, width: 700, height: 836 } },
  ],
};
/** DP-10 on its third workspace: what the picker shows once the person switches to it. */
export const DP10_THIRD: Monitor = {
  ...DP10,
  workspace: 3,
  windows: [
    { app: FIREFOX, title: FIREFOX.titles[1]!, rect: { x: 16, y: 48, width: 780, height: 936 } },
    { app: STEAM, title: STEAM.titles[0]!, rect: { x: 804, y: 48, width: 780, height: 936 } },
  ],
};
/** The narrowest screen the bar is laid out for, an app's least window wide. */
export const NARROW: Monitor = {
  connector: "HDMI-A-1",
  width: 720,
  height: 480,
  density: 1,
  windows: [{ app: FIREFOX, title: FIREFOX.titles[0]!, rect: { x: 16, y: 48, width: 688, height: 416 } }],
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

/** The words for what a share takes, by its kind. */
export const SHARE_WHAT = {
  screen: (connector: string) => connector,
  window: (app: string) => `${app}'s window`,
  region: (connector: string) => `A region of ${connector}`,
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
  rightDrag: "Drag with the right button anywhere to share a region at once.",
  withWhom: (who: string, since: string) => `With ${who} since ${since}`,
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

/** The shares the pills and panels show: Meet in Chromium, and a second, Firefox's window to Discord. */
export const SHARES = {
  screen: { kind: "screen", what: SHARE_WHAT.screen("DP-10"), who: MEET.app, site: MEET.site!, since: SINCE.sharing },
  window: { kind: "window", what: SHARE_WHAT.window("Firefox"), who: MEET.app, site: MEET.site!, since: SINCE.sharing },
  region: { kind: "region", what: SHARE_WHAT.region("DP-10"), who: MEET.app, site: MEET.site!, since: SINCE.sharing },
  second: { kind: "window", what: SHARE_WHAT.window("Firefox"), who: DISCORD.app, since: "21:40" },
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

/** A box's place and size as the CSS draws it at the scale picked. */
function place(element: HTMLElement, rect: Rect): void {
  element.style.left = `calc(${rect.x} * var(--px))`;
  element.style.top = `calc(${rect.y} * var(--px))`;
  element.style.width = `calc(${rect.width} * var(--px))`;
  element.style.height = `calc(${rect.height} * var(--px))`;
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
 * Draft part, proposed: a shared screen's marks, four squares of `sharing` as tall as the bar in
 * the screen's own outer corners, square and unrounded, over the frame's rounded hole.
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
      ? SHARES.screen
      : shared.kind === "window"
        ? { kind: "window", what: SHARE_WHAT.window(DESK[shared.index]?.app.app ?? FIREFOX.app), who: MEET.app }
        : SHARES.region
    : undefined;
  const made = barScreen(SCREEN_WIDTH, TALL, { content: { ...EVERYDAY, ...(share ? { sharing: share } : {}), ...(options.camera ? { camera: { who: DISCORD.app } } : {}) } });
  const desk = document.createElement("span");
  desk.className = "share-desk";
  desk.append(...DESK.map((each, index) => deskWindow({ ...each, focused: index === (options.focused ?? 0), shared: shared?.kind === "window" && shared.index === index })));
  made.element.querySelector(".k-screen-dim")!.before(desk);
  if (shared?.kind === "region") made.element.append(regionMark(shared.rect ?? REGION));
  if (shared?.kind === "screen") made.element.append(screenMarks());
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
    place(box, rect);
    size.style.left = `calc(${rect.x} * var(--px))`;
    size.style.top = `calc(${rect.y + rect.height} * var(--px) + var(--space-s8))`;
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
  if (lit.kind === "screen") return { kind: "screen", what: SHARE_WHAT.screen(monitor.connector), who: asker.app };
  return { kind: "region", what: SHARE_WHAT.region(monitor.connector), who: asker.app };
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
  /** The pill in the bars while still picking, shut: the sample the move to sharing plays on. */
  readonly pill?: boolean;
  /** The right-button tip under each bar's row. */
  readonly tip?: boolean;
  readonly handlers?: PickHandlers;
}

/**
 * The bar on one screen: who asks, Share this screen, Region and Cancel on a Popover, each button
 * saying its key in its Tooltip, and the right-button tip under them where it is shown.
 */
export function pickBar(asker: Asker, options: { lit?: "screen" | "region"; tip?: boolean; onScreen?: () => void; onPoint?: (on: boolean) => void; onRegion?: () => void; onCancel?: () => void } = {}): HTMLElement {
  const who = clamped(WORDS.title(asker.site ?? asker.app), "body_strong", "on_surface");
  who.classList.add("share-pick-who");
  const share = button({ label: WORDS.screen, kind: "Tonal", size: "Sm", icon: SHARE_ICONS.screen, ...(options.lit === "screen" ? { pinned: "lit" as const } : {}) });
  const region = button({ label: WORDS.region, kind: "Text", size: "Sm", icon: SHARE_ICONS.region, ...(options.lit === "region" ? { pinned: "pressed" as const } : {}) });
  const cancel = iconButton({ icon: "close", label: WORDS.cancel, size: "Sm" });
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
    who,
    floatingTooltip(share, keyTooltip(WORDS.screen, PICK_KEYS.screen), { side: "above" }),
    floatingTooltip(region, keyTooltip(WORDS.region, PICK_KEYS.region), { side: "above" }),
    floatingTooltip(cancel, keyTooltip(WORDS.cancel, "Esc"), { side: "above" }),
  );
  const bar = popover(2, options.tip ? stack("s8", "start", line, tipLine(WORDS.rightDrag)) : line);
  bar.classList.add("share-pick-bar");
  return bar;
}

/** The label on a window lit while picking: what a click does, and the window's name; Enter, from the keyboard. */
export function pickLabel(title: string, keyboard = false): HTMLElement {
  const label = tooltip(WORDS.click, title, keyboard ? row("s4", "centre", keycap("Enter")) : undefined);
  label.classList.add("share-pick-label");
  return label;
}

/** Workspaces 1 to 3, each with a window: what the bar shows once the person has gone to the third. */
const THREE: readonly Group[] = [{ block: 0, spaces: [{ id: 1, occupied: true }, { id: 2, occupied: true }, { id: 3, occupied: true }] }];

/** One screen while picking or once shared: its windows, its bar, and what is lit on it. */
export function pickingScreen(monitor: Monitor, index: number, options: PickingOptions = {}): HTMLElement {
  const asker = options.asker ?? MEET;
  const lit = options.lit ?? NOTHING;
  const phase = options.phase ?? "picking";
  const here = lit.kind !== "none" && lit.screen === index;
  const share = shareOf(lit, options.monitors ?? MONITORS, asker);
  const pill = (phase === "shared" || options.pill) && share ? { sharing: share } : {};
  const made = barScreen(monitor.width, `calc(${monitor.height} * var(--px))`, {
    content: { ...EVERYDAY, ...pill },
    ...(monitor.workspace ? { workspaces: { groups: THREE, active: monitor.workspace } } : {}),
  });
  const element = made.element;
  element.classList.add("share-screen");
  const desk = document.createElement("span");
  desk.className = "share-desk";
  const handlers = options.handlers ?? {};
  monitor.windows.forEach((placed, at) => {
    const on = here && lit.kind === "window" && lit.index === at;
    const drawn = deskWindow({ ...placed, shared: on, focused: false });
    if (on) {
      drawn.dataset.lifted = "";
      if (phase === "picking") drawn.append(pickLabel(placed.title, lit.kind === "window" && lit.keyboard === true));
    }
    if (phase === "picking") {
      const window: Lit = { kind: "window", screen: index, index: at };
      drawn.addEventListener("pointerenter", () => handlers.onLight?.(window));
      drawn.addEventListener("click", () => handlers.onShare?.(window));
    }
    desk.append(drawn);
  });
  element.querySelector(".k-screen-dim")!.before(desk);
  if (here && lit.kind === "screen") element.append(screenMarks());
  if (here && lit.kind === "region" && phase === "shared" && lit.rect) element.append(regionMark(lit.rect));
  if (phase === "picking") {
    element.dataset.picking = "";
    // A little dim on every screen; a screen lit whole comes out of it.
    if (!(here && lit.kind === "screen")) element.dataset.open = "";
    if (here && lit.kind === "region") {
      element.dataset.drawing = "";
      element.append(
        regionDraw({
          ...(lit.rect ? { rect: lit.rect } : {}),
          ...(lit.from ? { from: lit.from } : {}),
          width: monitor.width,
          height: monitor.height,
          density: monitor.density,
          onDone: (rect) => handlers.onShare?.({ kind: "region", screen: index, rect }),
          onBack: () => handlers.onLight?.(NOTHING),
        }),
      );
    }
    element.append(
      pickBar(asker, {
        ...(here && (lit.kind === "screen" || lit.kind === "region") ? { lit: lit.kind } : {}),
        ...(options.tip ? { tip: true } : {}),
        onScreen: () => handlers.onShare?.({ kind: "screen", screen: index }),
        onPoint: (on) => handlers.onLight?.(on ? { kind: "screen", screen: index } : NOTHING),
        onRegion: () => handlers.onLight?.({ kind: "region", screen: index }),
        onCancel: () => handlers.onCancel?.(),
      }),
    );
    // A right-button drag anywhere draws a region at once, from where it was pressed.
    element.addEventListener("contextmenu", (event) => event.preventDefault());
    element.addEventListener("pointerdown", (event) => {
      if (event.button !== 2) return;
      const bounds = element.getBoundingClientRect();
      const from = { x: ((event.clientX - bounds.left) / bounds.width) * monitor.width, y: ((event.clientY - bounds.top) / bounds.height) * monitor.height };
      handlers.onLight?.({ kind: "region", screen: index, from });
    });
  }
  return element;
}

/** Every screen side by side, top-aligned as Hyprland lays them out, picking or shared. */
export function picking(options: PickingOptions = {}): HTMLElement {
  const monitors = options.monitors ?? MONITORS;
  const laid = document.createElement("span");
  laid.className = "share-screens";
  laid.append(...monitors.map((monitor, index) => pickingScreen(monitor, index, { ...options, monitors })));
  return laid;
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
  readonly shares?: readonly Share[];
  /** Change what's shared, on a share by its index: picking opens again, and its pick replaces the share. */
  readonly onChange?: (index: number) => void;
  /** Stop sharing, on a share by its index. */
  readonly onStop?: (index: number) => void;
}

/** One share's card: what is shared, with whom since when, Change what's shared and Stop sharing. */
function shareCard(share: Share, index: number, options: SharingPanelOptions): HTMLElement {
  const change = button({ label: WORDS.change, kind: "Tonal", size: "Sm", icon: SHARE_ICONS[share.kind] });
  const stop = button({ label: WORDS.stop, kind: "Tonal", size: "Sm", icon: "stop" });
  if (options.onChange) change.addEventListener("click", () => options.onChange?.(index));
  if (options.onStop) stop.addEventListener("click", () => options.onStop?.(index));
  return card({
    header: { glyph: SHARE_ICONS[share.kind], title: share.what, subtitle: WORDS.withWhom(share.site ?? share.who, share.since ?? SINCE.sharing) },
    actions: row("s8", "centre", change, stop),
  });
}

/**
 * The sharing panel, a PanelPage: that notifications are held under the title, then a Card for
 * each share, saying what is shared, with whom and since when, with its own Change what's shared,
 * which opens picking again, and Stop sharing.
 */
export function sharingPanel(options: SharingPanelOptions = {}): HTMLElement {
  const shares = options.shares ?? [SHARES.screen];
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
    actions: close,
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
  return onCorner(sharingPanel(options), cornerBar(options.shares ?? [SHARES.screen], camera ? [CAMERAS.first.use] : undefined));
}

/** The camera panel open under its pill, the sharing pill beside it. */
export function cameraShot(options: CameraPanelOptions = {}, sharing = true): HTMLElement {
  return onCorner(cameraPanel(options), cornerBar(sharing ? [SHARES.screen] : undefined, (options.cameras ?? [CAMERAS.first]).map((each) => each.use)));
}
