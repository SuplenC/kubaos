// The full-screen moments the Backdrop page shows: polkit's prompt over the desktop, Kōbō's welcome
// and done screens, and the glass panel Kōbō's step-rail screens hold. Their words and parts are
// stand-ins from the library; each screen's own organisms (the done screen's neon strokes) are its
// surface's pages.

import "./backdrops.css";
import blurred from "../../../../../crates/kubaos-kobo/assets/wallpaper-blur.jpg?url";
import wallpaper from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import sign from "../../../../../crates/kubaos-kobo/assets/logo/kubaos-sign-1024.png?url";
import mark from "../../../../../crates/kubaos-kobo/assets/logo/kubaos-mark.svg?url";
import { backdrop, givesWay, glassPanel } from "../../lib/backdrop.ts";
import { button } from "../../lib/button.ts";
import type { Words } from "../../lib/content.ts";
import { dialog } from "../../lib/dialog.ts";
import { field, labelled } from "../../lib/field.ts";
import { keyHints } from "../../lib/keycap.ts";
import { gap, row, stack } from "../../lib/layout.ts";
import { spinner } from "../../lib/spinner.ts";
import { text, wrapped } from "../../lib/text.ts";
import { label } from "../../site/dom.ts";
import { desk } from "./panels.ts";

/** The size a sample is drawn at: an app's least width, and its default height, so Kōbō's sign shows. */
export const SAMPLE: { readonly width: string; readonly height: string } = { width: "var(--size-app_min_width)", height: "var(--size-app_height)" };

/** What a screen shows: at rest, waiting on the machine, or after something went wrong. */
export type Moment = "rest" | "loading" | "error";

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

/** A screen at the toolbar's window size: each of the moments, as the reflow shows them. */
export function atSize(which: "polkit" | "welcome" | "done" | "host", width: string, height: string): HTMLElement {
  const size = { width, height };
  switch (which) {
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
