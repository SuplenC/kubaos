// The full-screen moments the Backdrop page shows: the lock screen, polkit's prompt over the
// desktop, Kōbō's welcome and done screens, and the glass panel Kōbō's step-rail screens hold. Their
// words and parts are stand-ins from the library; each screen's own organisms (the done screen's
// neon strokes, the lock's authentication) are its surface's pages.

import "./backdrops.css";
import blurred from "../../../../../crates/kubaos-kobo/assets/wallpaper-blur.jpg?url";
import wallpaper from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import sign from "../../../../../crates/kubaos-kobo/assets/logo/kubaos-sign-1024.png?url";
import mark from "../../../../../crates/kubaos-kobo/assets/logo/kubaos-mark.svg?url";
import { backdrop, givesWay, glassPanel } from "../../lib/backdrop.ts";
import { button } from "../../lib/button.ts";
import type { Words } from "../../lib/content.ts";
import { dialog } from "../../lib/dialog.ts";
import { field, fieldRoom, labelled, note } from "../../lib/field.ts";
import { icon } from "../../lib/icon.ts";
import { keyHints } from "../../lib/keycap.ts";
import { gap, row, stack } from "../../lib/layout.ts";
import { spinner } from "../../lib/spinner.ts";
import { text, wrapped } from "../../lib/text.ts";
import { raw } from "../../lib/tokens.ts";
import { label } from "../../site/dom.ts";
import { settings } from "../../site/settings.ts";
import { desk } from "./panels.ts";

/** The size a sample is drawn at: an app's least width, and its default height, so Kōbō's sign shows. */
export const SAMPLE: { readonly width: string; readonly height: string } = { width: "var(--size-app_min_width)", height: "var(--size-app_height)" };

/** What a screen shows: at rest, waiting on the machine, or after something went wrong. */
export type Moment = "rest" | "loading" | "error";

export const LOCK_ERROR: Words = {
  short: "Wrong password. Fat fingers? Try again.",
  long: "Fingerprint not recognised. Wrong finger? Try again or type your password",
};

export const HERO: Words = { short: "Let's fix this shit.", long: "Let's fix this shit, once and for all, properly this time." };

export const LEDE: Words = {
  short: "Whatever's on here, it's had its chance.",
  long: "Whatever's on here, it's had its chance. Kuba's own Linux: a fast, keyboard-driven desktop for writing Rust all day without the OS getting in the way. I update when you say so, I never nag, and if you break me there's always a way back.",
};

export const DONE: Words = { short: "Fixed. You're welcome.", long: "Fixed, finally. You're very welcome, by the way." };

/** A column of the thing's parts, centred under one another, `width` wide (a length's CSS). */
function column(width: string, ...parts: Element[]): HTMLElement {
  const box = stack(0, "centre", ...parts);
  box.style.width = width;
  return box;
}

/** The lock's field, `state`'s: waiting while the password is checked, wrong after a miss. */
function lockField(moment: Moment): HTMLElement {
  const busy = moment === "loading";
  return field({ icon: busy ? "hourglass_top" : "lock", placeholder: busy ? "Checking…" : "Password", secret: true, ...(moment === "error" ? { state: "Invalid" as const } : {}) });
}

/** The line under the lock's field: the other ways in, or what went wrong. */
function lockStatus(moment: Moment, error: string): HTMLElement {
  if (moment === "error") return note(error, true);
  return row("s8", "centre", icon("fingerprint", "var(--type-small-icon)", "var(--color-primary)"), text("Or touch the fingerprint reader", "small", "on_surface_variant"));
}

/** The lock's column: the accent, the clock, the date, the field and the line under it. */
export function lockColumn(moment: Moment = "rest", error = LOCK_ERROR.short): HTMLElement {
  const room = fieldRoom("var(--size-password)", lockField(moment));
  room.classList.add("k-lock-field");
  return column(
    "var(--size-dialog)",
    text("東京・月曜日", "accent", "primary"),
    text("21:47", "clock"),
    gap("s8"),
    text("Monday, 2 November", "body_large", "on_surface_variant"),
    gap("s48"),
    room,
    gap("s8"),
    lockStatus(moment, error),
  );
}

export function lock(moment: Moment = "rest", error?: string): HTMLElement {
  return backdrop({ ...SAMPLE, picture: blurred, scrim: "band", content: lockColumn(moment, error), lift: "s64" });
}

/** The lock, live: Enter with a password shakes the field and says so; Esc clears it. */
export function liveLock(width: string = SAMPLE.width, height: string = SAMPLE.height): HTMLElement {
  const content = lockColumn();
  const screen = backdrop({ width, height, picture: blurred, scrim: "band", content, lift: "s64" });
  const room = content.querySelector<HTMLElement>(".k-lock-field")!;
  const status = () => room.parentElement!.lastElementChild as HTMLElement;
  const fresh = (moment: Moment) => {
    const next = lockField(moment);
    room.replaceChildren(next);
    status().replaceWith(lockStatus(moment, LOCK_ERROR.short));
    requestAnimationFrame(() => next.focus({ focusVisible: true } as FocusOptions));
    return next;
  };
  room.addEventListener("keydown", (event) => {
    const typed = room.querySelector(".k-field-room")?.textContent ?? "";
    if (event.key === "Escape") fresh("rest");
    if (event.key !== "Enter" || typed === "") return;
    event.preventDefault();
    fresh("error");
    if (!settings().reduced) room.animate(shakeFrames(), { duration: 3 * millis("micro") + millis("short") });
  });
  return screen;
}

/** A duration token in milliseconds, as the toolbar's speed divides it. */
function millis(name: string): number {
  return Number.parseFloat(raw(`duration-${name}`)) / settings().speed;
}

/** The wrong password's shake: `space.s12`, back `space.s8`, `space.s4`, each over `micro`, then home over `short`. */
function shakeFrames(): Keyframe[] {
  const total = 3 * millis("micro") + millis("short");
  const at = (n: number) => (n * millis("micro")) / total;
  return [
    { "--jolt": "0", offset: 0 },
    { "--jolt": "3", offset: at(1) },
    { "--jolt": "-2", offset: at(2) },
    { "--jolt": "1", offset: at(3), easing: raw("easing-standard_decel") },
    { "--jolt": "0", offset: 1 },
  ];
}

/** Polkit's prompt over the desktop it asks on, the desktop dimmed under it. */
export function polkit(moment: Moment = "rest"): HTMLElement {
  const password = field({ icon: "key", secret: true, ...(moment === "error" ? { value: "hunter", state: "Invalid" as const } : {}) });
  const asked = dialog({
    icon: "admin_panel_settings",
    title: "Authentication required",
    body: "Installing btop needs your password.",
    asks: labelled("Password for Kuba", password, moment === "error" ? { note: "That didn't work. Fat fingers? Try again.", invalid: true } : {}),
    dismiss: "Cancel",
    actions: [{ label: "Authenticate", ...(moment === "loading" ? { disabled: true } : {}) }],
    landed: true,
  });
  const { screen } = desk(SAMPLE.width, SAMPLE.height);
  return backdrop({ ...SAMPLE, scrim: "dim", content: asked.element, under: screen.element });
}

/** Kōbō's neon sign, `size.column` wide, as tall as its picture is at that width on a whole device pixel. */
function neonSign(): HTMLElement {
  const picture = document.createElement("img");
  picture.className = "k-backdrop-sign";
  picture.src = sign;
  picture.alt = "KubaOS";
  // Its height is the picture's own proportion at the column's width, which only the picture knows.
  const fit = () => {
    if (picture.naturalWidth > 0 && picture.offsetWidth > 0) picture.style.height = `${Math.round((picture.offsetWidth * picture.naturalHeight) / picture.naturalWidth)}px`;
  };
  picture.addEventListener("load", fit);
  new ResizeObserver(fit).observe(picture);
  return givesWay(picture);
}

/** The welcome's action and the line beside it: Let's go, waiting on the hardware, or Try again. */
function welcomeAction(moment: Moment): HTMLElement {
  if (moment === "loading") return row("s20", "centre", button({ label: "Let's go", size: "Lg", icon: "arrow_forward", trailing: true, disabled: true }), row("s8", "centre", spinner("s20"), text("Poking at the hardware…", "body", "on_surface_variant")));
  if (moment === "error") return row("s20", "centre", button({ label: "Try again", size: "Lg", icon: "restart_alt" }), text("Couldn't read this machine.", "body", "error"));
  return button({ label: "Let's go", size: "Lg", icon: "arrow_forward", trailing: true });
}

export function welcomeColumn(moment: Moment = "rest", hero = HERO.short, lede = LEDE.long): HTMLElement {
  return column(
    "var(--size-column)",
    text("ようこそ", "accent", "primary"),
    gap("s8"),
    neonSign(),
    gap("s20"),
    wrapped(hero, "hero"),
    gap("s16"),
    wrapped(lede, "body_large", "on_surface_variant"),
    gap("s40"),
    welcomeAction(moment),
    gap("s16"),
    keyHints([["Enter", moment === "error" ? "try again" : "start"]]),
  );
}

export function welcome(moment: Moment = "rest", hero?: string, lede?: string, size = SAMPLE): HTMLElement {
  return backdrop({ ...size, picture: wallpaper, scrim: "band", content: welcomeColumn(moment, hero, lede) });
}

export function doneColumn(headline = DONE.short): HTMLElement {
  const neon = document.createElement("img");
  neon.className = "k-backdrop-mark";
  neon.src = mark;
  neon.alt = "クバ";
  return column(
    "var(--size-column)",
    givesWay(neon),
    gap("s24"),
    wrapped(headline, "hero"),
    gap("s16"),
    wrapped("That's it. Done in 4 min 12 s. Pull the stick and reboot. Type the disk passphrase at boot and you're on the desktop as kuba.", "body_large", "on_surface_variant"),
    gap("s40"),
    row("s8", "centre", button({ label: "Reboot", size: "Lg", icon: "power_settings_new" }), button({ label: "Stay in the live system", kind: "Text", size: "Lg" })),
    gap("s16"),
    keyHints([
      ["Enter", "reboot"],
      ["Tab", "move"],
    ]),
  );
}

export function done(headline?: string): HTMLElement {
  return backdrop({ ...SAMPLE, picture: wallpaper, scrim: "band", content: doneColumn(headline) });
}

/** Kōbō's glass panel over the dimmed wallpaper: the room StepRailPage fills, on its own page. */
export function stepRailHost(width: string = SAMPLE.width, height: string = SAMPLE.height): HTMLElement {
  const panel = glassPanel(blurred, label("StepRailPage, on its own page", "small"));
  panel.classList.add("k-backdrop-host");
  return backdrop({ width, height, picture: wallpaper, scrim: "dim", content: panel });
}

/** A screen at the toolbar's window size: each of the four moments, as the reflow shows them. */
export function atSize(which: "lock" | "polkit" | "welcome" | "done" | "host", width: string, height: string): HTMLElement {
  const size = { width, height };
  switch (which) {
    case "lock":
      return liveLock(width, height);
    case "polkit": {
      const { screen } = desk(width, height);
      return backdrop({ ...size, scrim: "dim", content: polkitDialog(), under: screen.element });
    }
    case "welcome":
      return welcome("rest", undefined, undefined, size);
    case "done":
      return backdrop({ ...size, picture: wallpaper, scrim: "band", content: doneColumn() });
    case "host":
      return stepRailHost(width, height);
  }
}

function polkitDialog(): HTMLElement {
  return polkit().querySelector<HTMLElement>(".k-dialog")!;
}
