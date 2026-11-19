// The DescriptionList page: `docs/design/components/description-list.md`. Facts beside their labels,
// as a summary reads them: Kōbō's Last chance cards; and lines with an action at their end, shown
// while pointed at, as Settings › About copies a fact.

import type { Words } from "../../lib/content.ts";
import { iconButton } from "../../lib/icon-button.ts";
import { type Fact, descriptionList } from "../../lib/description.ts";
import { stack, within } from "../../lib/layout.ts";
import { doesntApply, type Move, nonEmpty, page } from "../../site/page.ts";

/** A card's content: the column less a card's padding. */
const ROOM = "calc(var(--size-column) - 2 * var(--space-s16))";

/** What Kōbō's Last chance says it will do. */
const INSTALL: readonly Fact[] = [
  { label: "Disk", value: "Samsung SSD 990 PRO, 2 TB", detail: "/dev/nvme0n1" },
  { label: "Encryption", value: "On, with a passphrase and your YubiKey" },
  { label: "Account", value: "Kuba", detail: "kuba@hoshi.local" },
  { label: "Version", value: "KubaOS 0.4", detail: "0.4.0+20261101.5cb2a85" },
];

const VALUE: Words = {
  short: "On",
  long: "Everything on the disk is erased, then KubaOS takes all of it and encrypts it with your passphrase and your YubiKey",
};
const LABEL: Words = { short: "Disk", long: "Where everything goes" };
const DETAIL: Words = { short: "/dev/sda", long: "/dev/disk/by-id/nvme-Samsung_SSD_990_PRO_2TB_S73WNJ0W123456X-part2" };

function facts(list: readonly Fact[], room = ROOM): HTMLElement {
  return within(room, descriptionList(list));
}

/** A line's copy action, pinned lit for a still sample. */
function copy(label: string, lit = false): HTMLElement {
  return iconButton({ icon: "content_paste", label: `Copy ${label.toLowerCase()}`, kind: "Text", size: "Sm", ...(lit ? { pinned: "lit" as const } : {}) });
}

/** About's System facts, each line with its copy action; `lit` names the line drawn pointed at. */
const SYSTEM = (lit?: string): Fact[] =>
  [
    { label: "Kernel", value: "6.17.0-5-generic" },
    { label: "Desktop", value: "Hyprland 0.52.1" },
    { label: "KubaOS", value: "Shinjuku 0.4.0", detail: "on Ubuntu 26.04 · 42c4611" },
    { label: "Session", value: "Wayland" },
  ].map((each) => ({ ...each, action: copy(each.label, each.label === lit) }));

/** An action's line at its longest: the value wraps before the action's column. */
const LONG_MODEL = "Lenovo ThinkPad X1 Carbon Gen 12 Aura Edition (21KC00BWPB) with Intel vPro";

const ACTION = "a line's action";
const SHOWS: Move = {
  name: "An action shows",
  what: "Pointed at, or the keyboard on it, it fades in over `short` on `standard`; nothing moves.",
  tracks: [{ part: "its fade", on: ACTION, target: ".k-fact-action > .k-button[data-pinned]", property: "opacity", from: 0, to: 1, duration: "short", curve: "standard" }],
  reduced: "same",
  back: "plays",
  sample: () => facts(SYSTEM("Kernel")),
};

export default page({
  level: "molecules",
  slug: "description-list",
  title: "DescriptionList",
  lede: "Facts beside their labels, as a summary reads them: a label a quarter of the width, its value the rest.",
  code: { rust: "description::description_list", qml: "DescriptionList", unbuilt: "a line's action: not built yet" },
  stage: {
    live: () => facts(INSTALL, "100%"),
    hint: "words, not a control: Tab passes it",
    daylight: true,
  },
  variants: [
    { name: "Facts", for: "a label and a value each", sample: () => facts(INSTALL.slice(1, 2)) },
    { name: "With a machine's line", for: "a path, an email or a version, in `mono`", sample: () => facts([INSTALL[0]!, INSTALL[3]!]) },
    { name: "With actions, at rest", for: "their column kept; nothing shows", sample: () => facts(SYSTEM()) },
    { name: "A line pointed at", for: "its action shows at the end, lit", sample: () => facts(SYSTEM("Kernel")) },
    { name: "A two-line fact's action", for: "centred down value and machine's line", sample: () => facts(SYSTEM("KubaOS")) },
    { name: "A line with none", for: "its column kept empty; the values still line up", sample: () => facts([...SYSTEM().slice(0, 2), { label: "Serial number", value: "Hidden" }]) },
  ],
  sizes: doesntApply("as wide as its place, as tall as its facts."),
  states: {
    shown: ["rest", "lit"],
    absent: {
      pressed: "takes no press",
      disabled: "nothing in it turns off",
      selected: "nothing in it is picked",
      loading: "its facts are known before it shows",
      empty: "with nothing to say, it isn't shown",
      error: "what failed is an `ErrorBlock`",
    },
    rows: [
      { name: "Last chance", cell: () => facts(INSTALL) },
      { name: "With actions", cell: (state: "rest" | "lit") => facts(SYSTEM(state === "lit" ? "Kernel" : undefined)) },
    ],
    looks: { rest: "labels and machine lines `on_surface_variant`, values `on_surface`", lit: "a line with an action: the action shows, lit" },
  },
  focus: doesntApply("the list takes no keyboard; a line's action lights as an IconButton does."),
  content: {
    text: [
      { name: "A value", words: VALUE, sample: (words: string) => facts([{ label: "Encryption", value: words }, INSTALL[2]!]) },
      { name: "A label", words: LABEL, sample: (words: string) => facts([{ label: words, value: "Samsung SSD 990 PRO, 2 TB" }, INSTALL[2]!]) },
      { name: "A machine's line", words: DETAIL, sample: (words: string) => facts([{ label: "Disk", value: "Samsung SSD 990 PRO, 2 TB", detail: words }]) },
      { name: "A value beside an action", words: { short: "Lenovo ThinkPad", long: LONG_MODEL }, sample: (words: string) => facts([{ label: "Maker and model", value: words, action: copy("Maker and model", true) }, ...SYSTEM().slice(0, 1)]) },
    ],
    missingPicture: doesntApply("it draws no pictures."),
    counts: [
      { name: "One", for: "a single fact", sample: () => facts(INSTALL.slice(1, 2)) },
      { name: "Several", for: "`space.s8` apart", sample: () => facts(INSTALL) },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its card."),
    leaves: doesntApply("it goes with its card."),
    between: [SHOWS],
  },
  keyboard: {
    keys: [
      { keys: ["Tab", "Shift+Tab"], does: "a line's action, each in reading order; it shows" },
      { keys: ["Enter", "Space"], does: "on an action: does it" },
    ],
    order: "Without actions Tab passes it; with them, each line's action from the top.",
    lands: "Nowhere on its own: it holds the keyboard only on an action.",
  },
  measures: {
    samples: nonEmpty([
      { name: "Last chance", for: "the label's quarter, the gaps, the machine's line", sample: () => facts(INSTALL) },
      { name: "With actions", for: "the action column, its gap, the action centred", sample: () => facts(SYSTEM("Kernel")) },
    ]),
    measures: [
      { part: "Between facts, label → value", token: "space-s8" },
      { part: "Value → machine's line", token: "space-s4" },
      { part: "Label line", token: "type-small-line" },
      { part: "Value line", token: "type-body-line" },
      { part: "Machine's line", token: "type-mono-line" },
      { part: "Action column", token: "size-control_sm" },
      { part: "Value → action column", token: "space-s8" },
    ],
    reflow: {
      note: "The label stays a quarter: narrow, words wrap and a machine's line ends in an ellipsis.",
      sample: () => stack("s16", "start", facts(INSTALL, "100%"), facts(INSTALL, "calc(var(--size-panel) / 2)")),
    },
  },
});
