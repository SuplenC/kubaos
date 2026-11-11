// The control center's quick toggles (`QuickToggles.qml`, `ProxyDetail.qml`): Wi-Fi, Bluetooth, Do
// not disturb, wallpaper motion, the keyboard layout and, with a profile set up, Proxy, two to a
// row; the Proxy tile's arrow opens its detail under them.

import type { Words } from "../../lib/content.ts";
import type { Shown as ControlShown } from "../../lib/control.ts";
import { within } from "../../lib/layout.ts";
import { tile, type TileOptions } from "../../lib/tile.ts";
import { lights, powerMoves, press } from "../../site/moves.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";
import { proxyDetail, TILES, toggles } from "./control-center-parts.ts";

const ROOM = "calc(var(--size-panel) - 2 * var(--container-0-padding))";
const HALF = "calc((var(--size-panel) - 2 * var(--container-0-padding) - var(--space-s8)) / 2)";

const inRoom = (element: HTMLElement) => within(ROOM, element);
const half = (options: TileOptions) => within(HALF, tile(options));

type Shown = "rest" | "lit" | "pressed" | "selected";

const ROWS: readonly { name: string; options: TileOptions }[] = [
  { name: "A switch", options: { ...TILES.dnd } },
  { name: "With a detail arrow", options: { ...TILES.proxy, on: false, subtitle: "Off", glyph: "key" } },
];

function cell(options: TileOptions, state: Shown): HTMLElement {
  const pinned: ControlShown | undefined = state === "lit" || state === "pressed" ? state : undefined;
  return half({ ...options, on: state === "selected", ...(state === "selected" ? { subtitle: options.title === "Proxy" ? "vpn.example.org:51820" : "On" } : {}), ...(pinned ? { pinned } : {}) });
}

const STATUS: Words = { short: "Off", long: "On, screen shared with a meeting that never ends" };
const TITLE: Words = { short: "Proxy", long: "Wallpaper motion on battery" };

const TURNS: Move = {
  name: "Turning on",
  what: "A press, Enter or Space: the fill and content fade to on over `short`.",
  tracks: [{ part: "the fill and content", on: "Do not disturb", target: ".k-tile", property: "--checked", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: () => half(TILES.dnd),
};

export default page<Shown>({
  level: "shell",
  slug: "control-center-toggles",
  title: "Quick toggles",
  lede: "The control center's tiles, two to a row, and the Proxy tile's detail under them.",
  code: { qml: "QuickToggles" },
  stage: { live: () => inRoom(toggles({ live: true })), hint: "click a tile; Proxy's arrow opens its detail", daylight: true },
  variants: [
    { name: "With Proxy", for: "a profile set up: six tiles", sample: () => inRoom(toggles()) },
    { name: "No Proxy", for: "the keyboard tile across its own row", sample: () => inRoom(toggles({ proxy: false })) },
    { name: "Proxy detail, WireGuard", for: "state, VPN, server, local network, live rows", sample: () => inRoom(toggles({ detail: {} })) },
    { name: "Proxy detail, OpenVPN", for: "no handshake row", sample: () => inRoom(proxyDetail({ openvpn: true })) },
    { name: "Local network blocked", for: "the kill switch lets nothing through", sample: () => inRoom(proxyDetail({ local: false })) },
  ],
  sizes: doesntApply("tiles fill half the panel's room; one alone takes it whole."),
  states: {
    shown: ["rest", "lit", "pressed", "selected"],
    absent: {
      disabled: "never: an unavailable radio says so in its status",
      loading: "a slow switch says so: Switching…",
      empty: "every tile has its status line",
      error: "Proxy's problem shows in its detail's State row",
    },
    rows: nonEmpty(ROWS.map(({ name, options }) => ({ name, cell: (state: Shown) => cell(options, state) }))),
    looks: {
      rest: "off: `surface_container_high`, status `on_surface_variant`",
      lit: "tube inside its edge, glow at night",
      pressed: "the glow brightens; by day the tone tints it",
      selected: "on: `primary_fill`, content `on_primary_fill`",
    },
  },
  focus: {
    tubes: [
      { name: "A tile off", tone: "accent", filled: false, content: false },
      { name: "A tile on", tone: "accent", filled: true, content: false },
      { name: "The detail arrow", tone: "accent", filled: false, content: false },
    ],
    looks: {
      lit: "off, the tube inside; on, the core inside the fill",
      signsOff: "`primary_glass` off, `primary_fill_glass` on; no glow",
    },
    signsOff: (index: number) => half({ ...ROWS[index]!.options, pinned: "signs-off" }),
  },
  content: {
    text: [
      { name: "A status line", words: STATUS, sample: (words: string) => half({ ...TILES.dnd, subtitle: words }) },
      { name: "A title", words: TITLE, sample: (words: string) => half({ ...TILES.motion, title: words }) },
    ],
    missingPicture: doesntApply("its symbol is the service's own, always there."),
    counts: [
      { name: "Five tiles", for: "no Proxy: the last takes the row", sample: () => inRoom(toggles({ proxy: false })) },
      { name: "Six tiles", for: "three rows of two", sample: () => inRoom(toggles()) },
      { name: "Detail before the first reading", for: "only the steady rows", sample: () => inRoom(proxyDetail({ steady: true })) },
    ],
  },
  motion: {
    arrives: doesntApply("the tiles come with the panel."),
    leaves: doesntApply("they go with the panel."),
    between: [
      TURNS,
      {
        name: "The detail opens",
        what: "Proxy's arrow, or →: the detail fades in under the tiles over `short`.",
        tracks: [{ part: "the detail", on: "the Proxy detail", target: ".cc-detail", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
        reduced: "same",
        back: "plays",
        sample: () => inRoom(toggles({ detail: {} })),
      },
      { name: "A tile lights up", what: "Pointer or keyboard: the tube and glow fade in.", tracks: lights(".k-tile", "Do not disturb", 0, 1), reduced: "same", back: "plays", sample: () => half(TILES.dnd) },
      { name: "A tile pressed", what: "A lit tile pressed: its tube runs hot.", tracks: press(".k-tile", "Do not disturb"), reduced: "same", back: "plays", sample: () => half({ ...TILES.dnd, pinned: "lit" }) },
      ...powerMoves(() => inRoom(toggles())),
    ],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "tile to tile in reading order" },
      { keys: ["Enter", "Space"], does: "toggles; Keyboard switches to the next layout" },
      { keys: ["→"], does: "the arrow: Keyboard's tab, or Proxy's detail" },
      { keys: ["Space"], does: "on the local network's switch: flips it" },
    ],
    order: "Wi-Fi, Bluetooth, Do not disturb, motion, Keyboard, Proxy, then the detail's switch.",
    lands: "The control center opens on the Wi-Fi tile.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Six tiles", for: "each tile, the gaps between", sample: () => inRoom(toggles()) },
      { name: "The detail", for: "its rows and the switch", sample: () => inRoom(proxyDetail()) },
    ]),
    measures: [
      { part: "Tile's height", token: "size-row_two_line" },
      { part: "Between tiles, to the detail", token: "space-s8" },
      { part: "Tile inside, symbol to words", token: "space-s12" },
      { part: "Tile's corners", token: "radius-control" },
      { part: "Detail arrow", token: "size-control_sm" },
      { part: "Detail's inside", token: "space-s16" },
      { part: "Between detail rows", token: "space-s4" },
      { part: "Label to value", token: "space-s12" },
      { part: "Detail's corners", token: "radius-content" },
    ],
    reflow: doesntApply("the panel is always `size.panel` wide."),
  },
});
