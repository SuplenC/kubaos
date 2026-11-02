// The worked example every layout page shows: Settings' Appearance page, built from the layout
// primitives and the library's parts alone, in a window of the toolbar's size, beside the sidebar of
// SidebarPage, the template it sits in.

import "./layout.css";
import photo from "../../../../../branding/photo/neon-sign-front@2x.jpg?url";
import { button } from "../../lib/button.ts";
import { chip } from "../../lib/chip.ts";
import { shown, type Words } from "../../lib/content.ts";
import { fill, filling, flow, grid, header, row, section, splitColumns, stack } from "../../lib/layout.ts";
import { list } from "../../lib/list.ts";
import { media } from "../../lib/media.ts";
import { role } from "../../lib/radius.ts";
import { sidebarPage } from "../../lib/sidebar-page.ts";
import { text, wrapped } from "../../lib/text.ts";
import { outline } from "../templates/settings.ts";

export const TITLE: Words = { short: "Appearance", long: "Appearance, wallpapers and the colours they bring" };
export const SUBTITLE: Words = {
  short: "Make it yours.",
  long: "Make it yours. It was already pretty, but go on, nobody's judging. Well, the wallpaper is, a little.",
};

/** The page's header: the accent, the title and its pun. */
export function appearanceHeader(words: { title?: string; subtitle?: string } = {}): HTMLElement {
  return header({ accent: "外観", title: words.title ?? shown(TITLE), subtitle: words.subtitle ?? shown(SUBTITLE) });
}

/** The theme's section: where the colours come from, as chips that flow onto a new line. */
export function themeSection(): HTMLElement {
  const chips = ["From the wallpaper", "Neon sign", "Rose", "Lagoon", "A colour of your own"].map((words, index) =>
    chip({ label: words, kind: index === 0 ? "Selected" : "Neutral" }),
  );
  return section("Theme", "Where the colours come from.", flow("s8", ...chips));
}

/** The wallpaper's section: a line with its action at the far end, then the set in a grid of three. */
export function wallpaperSection(columns = 3): HTMLElement {
  const pictures = ["Shinjuku, night", "Shinjuku, rain", "Shinjuku, dawn"].map((caption) =>
    media({ source: photo, corner: role("control"), caption }),
  );
  const line = row("s16", "centre", text("3 sets · 42 wallpapers", "body", "on_surface_variant"), fill(), button({ label: "Open the picker", kind: "Tonal", size: "Sm" }));
  return section("Wallpaper", "A new one every hour, from the sets you picked.", line, grid(columns, "s16", pictures));
}

/** The text's section: a label column a quarter wide beside its values, then a list of fonts. */
export function textSection(): HTMLElement {
  const facts = splitColumns(
    [1, 3],
    "s8",
    [
      wrapped("Size", "small", "on_surface_variant"),
      wrapped("100%, as the screen's scale has it", "body"),
      wrapped("Smoothing", "small", "on_surface_variant"),
      wrapped("Greyscale, for every screen", "body"),
    ],
    { centred: true },
  );
  const fonts = list([
    { title: "Interface", subtitle: "Inter, 14", opens: true },
    { title: "Monospace", subtitle: "JetBrains Mono, 13", opens: true },
  ]);
  return section("Text", undefined, facts, fonts);
}

/** Settings at the toolbar's window size, on Appearance: SidebarPage, the page its region. */
export function settingsWindow(): HTMLElement {
  const sections = [...outline().sections];
  const appearance = sections[0]!;
  sections[0] = {
    ...appearance,
    title: shown(TITLE),
    lede: shown(SUBTITLE),
    subs: ["Theme", "Wallpaper", "Text"].map((title) => ({ title, wells: [] })),
    region: () => stack("s32", "start", ...[themeSection(), wallpaperSection(), textSection()].map(filling)),
  };
  return sidebarPage({ outline: { ...outline(), sections }, section: 0 });
}
