// What the wallpaper picker, the power menu and the OSD are drawn on: the shots' screen, 1600 by
// 1000, with the bar as it is now, and a sheet hung on it from the library. Their pages build their
// samples from these.

import { containerSteps } from "../../lib/layout.ts";
import { type Edge, hang, type Screen, type Sheet, screen, sheet } from "../../lib/sheet.ts";
import { length } from "../../lib/tokens.ts";
import { barScreen, bellPill, statusPill, sysmonPill, wallpaper } from "./bar-parts.ts";
import { CORNER, TALL } from "./control-center-parts.ts";

/** The shots' screen width, in logical pixels. */
export const SCREEN_WIDTH = 1600;
export { TALL };

/** A narrower screen, an app's least width: where a wide sheet meets its sides. */
export const NARROW_WIDTH = 900;

export interface OnScreen {
  readonly screen: Screen;
  readonly sheet: Sheet;
}

export interface HungOptions {
  readonly edge: Edge;
  /**
   * The screen's corner a small sheet is drawn in, a panel wide and an app's least height, with the
   * bar's right end over it: the power menu, the OSD.
   */
  readonly corner?: boolean;
  readonly width?: number;
  readonly height?: string;
  /** How far open it is drawn, 0 to 1. */
  readonly reveal?: number;
  /** The desktop dimmed under it: a panel's, or a dialog's darker dim. */
  readonly dim?: "panel" | "dialog";
  /** On the right edge, how far below the bar it starts: a length's CSS. */
  readonly start?: string;
}

/** `content` on a sheet of a screen with the bar as it is now, drawn still. */
export function hung(content: HTMLElement, options: HungOptions): OnScreen {
  const room = options.corner
    ? screen({ width: CORNER, height: options.height ?? "var(--size-app_min_height)", wallpaper, bar: [sysmonPill(), bellPill(), statusPill()] })
    : barScreen(options.width ?? SCREEN_WIDTH, options.height ?? TALL);
  if (options.dim) {
    room.element.dataset.open = "";
    if (options.dim === "dialog") room.element.dataset.dim = "dialog";
  }
  const made = sheet({
    content,
    edge: options.edge,
    ...(options.reveal !== undefined ? { reveal: options.reveal } : {}),
    ...(options.start !== undefined ? { start: options.start } : {}),
  });
  hang(room, made);
  return { screen: room, sheet: made };
}

/** A sheet made live: shut until shown, opening and shutting as the library's do. */
export function live(made: Sheet, open: boolean): (open: boolean) => void {
  for (const part of made.parts) part.dataset.live = "";
  const set = (shown: boolean) => {
    for (const part of made.parts) {
      part.toggleAttribute("data-shown", shown);
      if (!shown) delete part.dataset.settled;
    }
    if (shown) requestAnimationFrame(() => requestAnimationFrame(() => made.parts.forEach((part) => (part.dataset.settled = ""))));
  };
  set(open);
  return set;
}

/**
 * The container padding round content `inner` logical pixels wide: the step whose width the content
 * and that padding on both sides reach, as `Tokens.containerPaddingAround` works it out.
 */
export function paddingAround(inner: number): string {
  const steps = containerSteps();
  for (let i = steps.length - 1; i > 0; i--) {
    if (inner + 2 * length(`space-${steps[i]!.space}`) >= steps[i]!.from) return steps[i]!.space;
  }
  return steps[0]!.space;
}
