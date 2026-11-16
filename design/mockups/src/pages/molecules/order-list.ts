// The OrderList page: `docs/design/components/order-list.md`. Things in an order that matters, each
// offered or not: a grab handle, a name and a Switch a row, dragged or moved with Alt and an arrow.

import type { Words } from "../../lib/content.ts";
import { within } from "../../lib/layout.ts";
import { type OrderItem, orderList } from "../../lib/order-list.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

const ROOM = "var(--size-dialog)";

const ENGINES = (): OrderItem[] => [
  { name: "DuckDuckGo", on: true },
  { name: "Google", on: true },
  { name: "Wikipedia", on: false },
];

const NAME: Words = { short: "Kagi", long: "Search the whole of the company wiki and its attachments" };

function sample(options: { items?: OrderItem[]; disabled?: boolean; lifted?: number } = {}): HTMLElement {
  return within(ROOM, orderList({ items: options.items ?? ENGINES(), ...(options.disabled ? { disabled: true } : {}), ...(options.lifted === undefined ? {} : { lifted: options.lifted }) }).box);
}

const ON = "three engines, Google moved up";

/** Alt ↑ on Google: it and DuckDuckGo change places, each gliding one row to its new one. */
const REORDER: Move = {
  name: "A row moved up",
  what: "Google glides up a row and DuckDuckGo down one, over `medium` on `standard_decel`.",
  tracks: [
    { part: "the row moved", on: ON, target: ".k-order-row:nth-child(1)", property: "--order-glide", from: 1, to: 0, duration: "medium", curve: "standard_decel" },
    { part: "the row passed", on: ON, target: ".k-order-row:nth-child(2)", property: "--order-glide", from: -1, to: 0, duration: "medium", curve: "standard_decel" },
  ],
  reduced: [
    { part: "the row moved", on: ON, target: ".k-order-row:nth-child(1)", property: "--order-glide", from: 0, to: 0, duration: "instant", curve: "standard" },
  ],
  back: "cuts",
  sample: () => sample({ items: [{ name: "Google", on: true }, { name: "DuckDuckGo", on: true }, { name: "Wikipedia", on: false }] }),
};

export default page<"rest" | "selected" | "disabled">({
  level: "molecules",
  slug: "order-list",
  title: "OrderList",
  lede: "Things in an order that matters, each offered or not: drag a row, or Alt and an arrow.",
  code: { unbuilt: "neither library has it yet" },
  stage: { live: () => sample(), hint: "drag a handle, or focus a row and press Alt ↑ ↓", daylight: true },
  variants: [
    { name: "Ordered", for: "the web search's engines", sample: () => sample() },
    { name: "Lifted", for: "a row mid-drag", sample: () => sample({ lifted: 1 }) },
  ],
  sizes: doesntApply("one size: its rows are `size.control_md`, its names `body`."),
  states: {
    shown: ["rest", "selected", "disabled"],
    absent: {
      lit: "its Switch lights; a row takes the focus tube",
      pressed: "a press on its handle lifts the row: the Lifted variant",
      loading: "its things are known when it is drawn",
      empty: "a list to order has two things at least",
      error: "nothing it does can fail",
    },
    rows: [{ name: "Three engines", cell: (state) => sample(state === "disabled" ? { disabled: true } : state === "selected" ? { lifted: 0 } : {}) }],
    looks: {
      rest: "each row's handle, name and switch",
      selected: "the first row lifted",
      disabled: "the whole list receded, still in place",
    },
  },
  focus: doesntApply("a row's focus is the library's cold tube; the Switch keeps its own."),
  content: {
    text: [{ name: "A name", words: NAME, sample: (words: string) => sample({ items: [{ name: words, on: true }, ...ENGINES()] }) }],
    missingPicture: doesntApply("its rows hold a handle, words and a switch."),
    counts: [
      { name: "Two", for: "the fewest", sample: () => sample({ items: ENGINES().slice(0, 2) }) },
      { name: "Five", for: "a longer list", sample: () => sample({ items: [...ENGINES(), { name: "Kagi", on: false }, { name: "Brave", on: true }] }) },
    ],
  },
  motion: {
    arrives: doesntApply("it shows with its page."),
    leaves: doesntApply("it goes with its page."),
    between: [REORDER],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "a row, then its switch" },
      { keys: ["Alt+↑", "Alt+↓"], does: "moves the focused row a place" },
      { keys: ["Space"], does: "on the switch, turns it" },
    ],
    order: "Each row, then its switch, top to bottom.",
    lands: "A moved row keeps the focus in its new place.",
  },
  measures: {
    samples: nonEmpty([{ name: "Ordered", for: "rows and their parts", sample: () => sample() }]),
    measures: [
      { part: "Row", token: "size-control_md" },
      { part: "Row's sides", token: "space-s16" },
      { part: "Handle → name → switch", token: "space-s12" },
      { part: "Between rows", token: "stroke-hairline_device_px" },
    ],
    reflow: { note: "Its rows take the width they are given; a long name wraps before the switch.", sample: () => within("100%", orderList({ items: ENGINES() }).box) },
  },
});
