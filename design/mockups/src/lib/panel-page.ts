// The inside of a shell panel's `Sheet`: the PanelPage template (`docs/design/components/panel-page.md`),
// which the system monitor, the notifications, the control center, the clock and the wallpaper
// picker are built on.
//
// From the top: the header band, its title in `heading` over its subtitle line in `small`, and the
// panel's icon buttons at its end, centred on the two lines; the tabs, if it has them, `space.s16`
// under it; the body, `space.s16` under those, the only part that scrolls, in a `Scroll` with a
// `HeaderFade` over the gap above it; and the footer band, if it has one, `space.s16` under the body,
// its action at the end. The panel is `space.s16` inside at its top and its foot, and its bands and
// its body take the container's padding at their sides. The subtitle line is always there and
// always has a pun in it, so nothing under it ever moves.
//
// It is as tall as its content, up to the room the screen has inside the frame (`--sheet-room`);
// past that only the body scrolls, and the bands stay. It is `size.panel` wide, or as wide as its
// content.

import "./panel-page.css";
import { container, fill, filling, row, stack } from "./layout.ts";
import { probe } from "./probe.ts";
import { scroll } from "./scroll.ts";
import { type TabsOptions, tabs } from "./tabs.ts";
import { clamped } from "./text.ts";

export interface PanelPageOptions {
  readonly title: string;
  /** The line under the title: the panel's pun. Never empty, so the header never shifts. */
  readonly subtitle: string;
  /** The panel's icon buttons, at the end of the header band. */
  readonly actions?: readonly HTMLElement[];
  /** Tabs right under the header, which don't scroll. */
  readonly tabs?: TabsOptions;
  /** What the body holds: the panel's organisms. */
  readonly body: HTMLElement;
  /** The footer band's action, at its end. */
  readonly footer?: readonly HTMLElement[];
  /** As wide as its content, as the clock and the wallpaper picker are, rather than `size.panel`. */
  readonly sized?: boolean;
  /** Scrolled this share of the way down, as a sample of it scrolled. */
  readonly down?: number;
  /** The header fade pinned shown, as while scrolled. */
  readonly fadePinned?: boolean;
}

export interface PanelPage {
  readonly element: HTMLElement;
  /** The body's scroller, which the owner scrolls a control into view in. */
  readonly view: HTMLElement;
  /** The tabs' well, when it has them. */
  readonly tabs: HTMLElement | null;
  /** Puts `words` in the subtitle line, for a pun that follows the panel's state. */
  readonly subtitle: (words: string) => void;
}

/** The subtitle line: one line, cut with an ellipsis where its words run past the band. */
function subtitleLine(words: string): HTMLElement {
  const line = clamped(words, "small", "on_surface_variant");
  line.classList.add("k-panel-subtitle");
  return line;
}

/** The header band: the title over its pun, and the panel's buttons at its end, centred on the two. */
function headerBand(options: PanelPageOptions): HTMLElement {
  const lines = stack(0, "start", clamped(options.title, "heading"), subtitleLine(options.subtitle));
  const band = row("s8", "centre", filling(lines), ...(options.actions?.length ? options.actions : []));
  const box = container(band);
  box.classList.add("k-panel-band", "k-panel-header");
  return box;
}

function footerBand(actions: readonly HTMLElement[]): HTMLElement {
  const box = container(row("s8", "centre", fill(), ...actions));
  box.classList.add("k-panel-band", "k-panel-footer");
  return box;
}

export function panelPage(options: PanelPageOptions): PanelPage {
  const element = document.createElement("span");
  element.className = "k-panel-page";
  element.toggleAttribute("data-sized", options.sized ?? false);
  const header = headerBand(options);
  const well = options.tabs ? tabs(options.tabs) : null;
  const tabsBand = well ? container(well) : null;
  tabsBand?.classList.add("k-panel-band", "k-panel-tabs");

  const { box: scroller, view } = scroll(options.body, {
    height: "var(--natural)",
    above: "s16",
    below: "s16",
    fade: "s16",
    ...(options.down !== undefined ? { down: options.down } : {}),
    ...(options.fadePinned ? { fadePinned: true } : {}),
  });
  const body = container(scroller);
  body.classList.add("k-panel-body");
  // The body is as tall as what it holds until the panel reaches its room: the content's height is
  // what only layout knows, and the panel's flex column shrinks the body below it.
  new ResizeObserver(() => scroller.style.setProperty("--natural", `${options.body.offsetHeight}px`)).observe(options.body);

  element.append(header, ...(tabsBand ? [tabsBand] : []), body, ...(options.footer?.length ? [footerBand(options.footer)] : []));
  probe(element, { role: "component", name: "PanelPage", variant: [options.tabs ? "tabs" : "", options.footer ? "footer" : "", options.sized ? "sized" : ""].filter(Boolean).join(" "), fixed: { width: !(options.sized ?? false), height: false, holds: true } });
  const subtitle = (words: string) => header.querySelector(".k-panel-subtitle")?.replaceWith(subtitleLine(words));
  return { element, view, tabs: well, subtitle };
}
