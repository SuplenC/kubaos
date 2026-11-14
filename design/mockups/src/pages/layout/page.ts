// The Page and Container page: `docs/design/components/layout.md`, "Page" and "Container". The two
// boxes that pad a surface's content at its sides by the container's step for their own width;
// a page fills its room and keeps `space.s32` above and `space.s24` below, a container is as tall
// as what it holds.

import { button } from "../../lib/button.ts";
import { container, divider, header, inset, page, row, stack, within } from "../../lib/layout.ts";
import { text } from "../../lib/text.ts";
import { powerMoves } from "../../site/moves.ts";
import { doesntApply, nonEmpty, page as mockPage } from "../../site/page.ts";
import { appearanceHeader, settingsWindow } from "./example.ts";

/**
 * `box` with its room's edge dashed, so a box that draws nothing (a container, a page on the site's
 * own surface) shows where it ends; over a surface of its own where it is `clear`.
 */
function room(box: HTMLElement, clear = false): HTMLElement {
  box.classList.add("l-room");
  box.toggleAttribute("data-glass", clear);
  return box;
}

/** A page `width` wide (a length's CSS) and `height` tall, holding a header and a line under it. */
function pageAt(width: string, height = "calc(var(--size-app_min_height) / 2)", clear = false): HTMLElement {
  const box = within(width, page(stack("s24", "start", appearanceHeader(), text("The page's body, where its sections go.", "body", "on_surface_variant")), { clear }));
  box.style.height = height;
  return room(box, clear);
}

/** A toolbar band: small buttons with `space.s16` above and below, the container at its sides. */
function band(width = "var(--size-panel)"): HTMLElement {
  const buttons = row("s8", "centre", button({ label: "Back", kind: "Text", size: "Sm" }), button({ label: "Save", kind: "Filled", size: "Sm" }));
  return room(within(width, container(inset(buttons, "s16"))));
}

/** A panel's body: a container holding its words, the panel's own step above and below. */
function panelBody(): HTMLElement {
  const body = stack("s8", "start", text("Wi-Fi", "heading"), text("Home network · secured", "body", "on_surface_variant"));
  return room(within("var(--size-panel)", container(inset(body, "s16"))));
}

/** A window's bar over its body: a container band, the hairline, then the page. */
function barred(width = "var(--size-app_min_width)"): HTMLElement {
  return within(width, band("100%"), divider(), pageAt("100%"));
}

/** A window's bar over its body as wide as the room it has, its corner dragged to resize it. */
function resizable(): HTMLElement {
  const box = barred("100%");
  box.style.resize = "horizontal";
  box.style.overflow = "hidden";
  box.style.minWidth = "var(--size-panel)";
  return box;
}

export default mockPage({
  level: "layout",
  slug: "page",
  title: "Page and Container",
  lede: "The two boxes a surface's content sits in, padded at their sides by the container's step for their own width.",
  code: { rust: "layout::page, layout::page_clear, layout::container", qml: "Page, Container" },
  stage: { live: resizable, hint: "drag its corner: the padding follows its width", daylight: true },
  variants: [
    { name: "Page", for: "a whole page: `surface` behind, `space.s32` above, `space.s24` below", sample: () => pageAt("var(--size-column)") },
    { name: "Clear page", for: "the same over its own surface: Kōbō's glass panel", sample: () => pageAt("var(--size-column)", undefined, true) },
    { name: "Container, a toolbar band", for: "small controls, `space.s16` above and below", sample: () => band() },
    { name: "Container, a panel's body", for: "under its header, the panel's own step above and below", sample: panelBody },
  ],
  sizes: doesntApply("as wide as its room; its padding is the step for that width."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "no pointer or focus; what they hold has its own",
      pressed: "they take no press",
      disabled: "what they hold is disabled on its own",
      selected: "they hold no choice",
      loading: "what loads shows its own `Skeleton`",
      empty: "a page holds `EmptyState`; an empty container isn't drawn",
      error: "an `ErrorBlock` or a `Banner` inside says it",
    },
    rows: [
      { name: "Page", cell: () => pageAt("var(--size-panel)") },
      { name: "Container", cell: () => band() },
    ],
    looks: { rest: "a page draws `surface`; a clear page and a container nothing" },
  },
  focus: doesntApply("neither is a control: no line and no glow."),
  content: {
    text: [
      {
        name: "A header in a page",
        words: { short: "Appearance", long: "Appearance, wallpapers and the colours they bring" },
        sample: (words: string) => {
          const box = within("var(--size-panel)", page(stack("s24", "start", header({ accent: "外観", title: words, subtitle: "Make it yours." }))));
          box.style.height = "calc(var(--size-app_min_height) / 2)";
          return box;
        },
      },
    ],
    missingPicture: doesntApply("they draw no picture."),
    counts: [
      { name: "Nothing in it", for: "a page holds `EmptyState`, never an empty surface", sample: () => pageAt("var(--size-panel)") },
      { name: "A band over a page", for: "a window's bar over its body", sample: () => barred("var(--size-column)") },
    ],
  },
  motion: {
    arrives: doesntApply("a page comes with its window or its step, which moves it."),
    leaves: doesntApply("they go with their window or surface."),
    between: powerMoves(() => pageAt("var(--size-panel)")),
  },
  keyboard: {
    keys: [{ keys: ["Tab", "Shift+Tab"], does: "through what they hold, in reading order" }],
    order: "They take no stop; their padding is room, not a control.",
    lands: "A window starts on its page's first thing to do.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Page, a column wide", for: "the container's padding, `space.s32` above, `space.s24` below", sample: () => pageAt("var(--size-column)") },
      { name: "Container, a band", for: "the container's padding, `space.s16` above and below", sample: () => band() },
      { name: "Page, an app's default width", for: "from 640 wide, `space.s24` at its sides", sample: () => pageAt("calc(var(--size-app_width) - var(--size-sidebar))") },
    ]),
    measures: [
      { part: "Side padding, under 640", token: "container-0-padding" },
      { part: "Side padding, from 640", token: "container-1-padding" },
      { part: "Side padding, from 1024", token: "container-2-padding" },
      { part: "Above a page's content", token: "space-s32" },
      { part: "Below a page's content", token: "space-s24" },
      { part: "Above and below a band's controls", token: "space-s16" },
      { part: "Between a band and its page", token: "stroke-hairline_device_px" },
      { part: "Settings' sidebar, beside the page", token: "size-sidebar" },
    ],
    reflow: {
      note: "Settings' Appearance beside its sidebar: padded 16 at the minimum, 24 at the default, 32 on a 1080p screen.",
      sample: settingsWindow,
    },
  },
});
