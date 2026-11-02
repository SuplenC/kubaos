// The Page and Container page: `docs/design/components/layout.md`, "Page" and "Container". The two
// boxes that pad a surface's content at its sides by the container's step for their own width;
// a page fills its room and keeps `space.s32` above and `space.s24` below, a container is as tall
// as what it holds.

import { button } from "../../lib/button.ts";
import { container, containerSteps, divider, header, inset, page, row, stack, within } from "../../lib/layout.ts";
import { text } from "../../lib/text.ts";
import { length } from "../../lib/tokens.ts";
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

/** Each step of the container, with the widths in it. */
const STEPS = (): string =>
  containerSteps()
    .map((step) => `from ${step.from}: \`space.${step.space}\` (${length(`space-${step.space}`)})`)
    .join(", ");

export default mockPage({
  level: "layout",
  slug: "page",
  title: "Page and Container",
  summary: "The two boxes a surface's content sits in: padded at its sides by the container's step for their own width, and nothing else.",
  live: () => barred(),
  description: [
    "Rust `layout::page`, `layout::page_clear` and `layout::container`; QML `Page` and `Container`. Every page and every panel's body is in one of them, so the content of every surface starts the same way in from its sides, and nothing else in the surface pads it again.",
    "Page is a whole page: it fills its room both ways, draws `surface` behind it, and puts its content the container padding in at the sides, `space.s32` below its top and `space.s24` above its foot, where a footer's buttons sit. A clear page (`page_clear`) draws nothing behind it, over a surface of its own: Kōbō's glass panel, a pane beside a rail.",
    "Container is a panel's body or a toolbar band: as wide as its room and as tall as what it holds, padded at its sides alone. Above and below, what it holds keeps its own step: a toolbar band a small control with `space.s16` above and below, a panel its own step (the control center `space.s16`, the clock `space.s20`).",
    "The container's padding is Tailwind's container on its breakpoints, by the box's own width, not the screen's: 16 under 640, 24 from 640, 32 from 1024 (`[container]`). A 384 panel takes 16 on any screen; Settings' page beside its 264 sidebar takes 16 at the window's 720 minimum and 24 at its 1120 default. The Space foundation shows the steps and the rule that padding is applied once; a card or a list in a page keeps its own padding inside it, and the page's is the only one round it.",
    "When to use which: a surface that is one page, scrolling or not, is a `Page`; a band across a surface (a toolbar, a footer, a panel's body under its header) is a `Container`. A dialog, a menu, a card and a well are none of these: each pads itself by the padding ladder (`docs/DESIGN.md`, \"Radii by role\"), and nothing pads round it.",
  ],
  variants: [
    { name: "Page", for: "A whole page on the surface: `surface` behind it, `space.s32` above, `space.s24` below.", sample: () => pageAt("var(--size-column)") },
    { name: "Clear page", for: "The same over a surface of its own, drawing nothing behind it: Kōbō's glass panel.", sample: () => pageAt("var(--size-column)", undefined, true) },
    { name: "Container, a toolbar band", for: "A band across a surface: small controls, `space.s16` above and below.", sample: () => band() },
    { name: "Container, a panel's body", for: "A panel's body under its header: the panel's own step above and below.", sample: panelBody },
  ],
  sizes: doesntApply("neither has a size of its own: each is as wide as its room, its padding the container's step for that width, and a page as tall as its room."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "they take no pointer and no focus; what they hold does.",
      pressed: "they take no press.",
      disabled: "they hold controls, which are disabled each on their own.",
      selected: "they hold no choice.",
      loading: "they come with their surface; what loads in them shows its own `Skeleton`.",
      empty: "an empty page holds its `EmptyState`; a container with nothing in it isn't drawn.",
      error: "an error is said by what they hold (an `ErrorBlock`, a `Banner`).",
    },
    rows: [
      { name: "Page", cell: () => pageAt("var(--size-panel)") },
      { name: "Container", cell: () => band() },
    ],
    described: {
      rest: "Always. A page draws `surface` behind its content (a clear page nothing); a container draws nothing. By day the same role in daylight colours.",
    },
  },
  focus: doesntApply("neither is a control: they have no line and no glow, by night or by day."),
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
      { name: "Nothing in it", for: "A page with nothing to show holds its `EmptyState`, never an empty surface; a container with nothing in it isn't drawn.", sample: () => pageAt("var(--size-panel)") },
      { name: "A band over a page", for: "One container band, the hairline under it, then the page: a window's bar over its body.", sample: () => barred("var(--size-column)") },
    ],
  },
  motion: {
    arrives: doesntApply("a page comes with its window or its step, which moves it (SidebarPage's change of section, Kōbō's slide); a container comes with its surface."),
    leaves: doesntApply("they go with their window or surface."),
    between: powerMoves(() => pageAt("var(--size-panel)")),
  },
  keyboard: {
    keys: [{ keys: ["Tab", "Shift+Tab"], does: "Nothing of their own: Tab goes through what they hold in reading order, top to bottom and start to end, a band before the page under it." }],
    order: "They take no stop and are never read out; their padding is room, not a control's.",
    lands: "A window starts on its page's first thing to do (Settings' Home on its search field, a Kōbō step on its first field).",
  },
  measures: {
    samples: nonEmpty([
      { name: "Page, a column wide", for: "The container's padding at its sides, `space.s32` above and `space.s24` below its content.", sample: () => pageAt("var(--size-column)") },
      { name: "Container, a band", for: "The container's padding at its sides; the band's own `space.s16` above and below.", sample: () => band() },
      { name: "Page, an app's default width", for: "From 640 wide the padding is `space.s24`.", sample: () => pageAt("calc(var(--size-app_width) - var(--size-sidebar))") },
    ]),
    measures: [
      { what: `The container's padding at each side, under 640 wide (${STEPS()}).`, token: "container-0-padding" },
      { what: "From 640 wide.", token: "container-1-padding" },
      { what: "From 1024 wide.", token: "container-2-padding" },
      { what: "A page's room above its content.", token: "space-s32" },
      { what: "A page's room below its content, where a footer's buttons sit.", token: "space-s24" },
      { what: "A toolbar band's room above and below its small controls.", token: "space-s16" },
      { what: "The hairline between a band and the page under it.", token: "stroke-hairline_device_px" },
      { what: "Settings' sidebar, beside the page in the worked example.", token: "size-sidebar" },
    ],
    reflow: {
      described:
        "Settings' Appearance page in a window of the toolbar's size, beside the room of its sidebar. The page takes what the sidebar leaves: 456 at the 720 minimum, padded 16; 856 at the 1120 default, 24; 1016 on a 1280 laptop, still 24; 1656 on a 1080p screen, 32. Its content scrolls when it is taller than the window; nothing in it is a fixed width but the wallpapers' shape.",
      sample: settingsWindow,
    },
  },
});
