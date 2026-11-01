// The DescriptionList page: `docs/design/components/description-list.md`. Facts beside their labels,
// as a summary reads them: Kōbō's Last chance cards.

import type { Words } from "../../lib/content.ts";
import { type Fact, descriptionList } from "../../lib/description.ts";
import { stack, within } from "../../lib/layout.ts";
import { text } from "../../lib/text.ts";
import { doesntApply, nonEmpty, page } from "../../site/page.ts";

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

export default page({
  level: "molecules",
  slug: "description-list",
  title: "DescriptionList",
  summary: "Facts beside their labels, as a summary reads them: a label a quarter of the width, its value the rest.",
  live: () => facts(INSTALL),
  description: [
    "Rust `description::description_list`, QML `DescriptionList`. Kōbō's Last chance cards: what is about to happen, said once more before it does.",
    "One fact under the other, `space.s8` apart: each label in `small`, `on_surface_variant`, a quarter of the width, its value in `body`, `on_surface`, the other three quarters, `space.s8` from it, the quarter split in whole device pixels. Both wrap in their room; each label sits at the top of its value. An optional `mono` line `space.s4` under the value, `on_surface_variant`, holds a path, an email or a version, on one line ending in an ellipsis, since those break badly wrapped mid-token.",
    "It is words, not a control: it takes no pointer and no keyboard.",
  ],
  variants: [
    { name: "Facts", for: "A label and a value each.", sample: () => facts(INSTALL.slice(1, 2)) },
    { name: "With a machine's line", for: "A path, an email or a version under the value, in `mono`.", sample: () => facts([INSTALL[0]!, INSTALL[3]!]) },
  ],
  sizes: doesntApply("it is as wide as its place, its label a quarter of it, and as tall as its facts."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "it is words, not a control.",
      pressed: "it takes no press.",
      disabled: "it says what will happen; nothing in it can be turned off.",
      selected: "nothing in it is picked.",
      loading: "its facts are known before it shows.",
      empty: "a summary with nothing to say isn't shown.",
      error: "a fact can't be wrong here; what failed is an ErrorBlock.",
    },
    rows: [{ name: "Last chance", cell: () => facts(INSTALL) }],
    described: { rest: "Labels `on_surface_variant`, values `on_surface`, machine lines `on_surface_variant`. By day the same roles in daylight colours." },
  },
  focus: doesntApply("it takes no keyboard."),
  content: {
    text: [
      { name: "A value", words: VALUE, sample: (words: string) => facts([{ label: "Encryption", value: words }, INSTALL[2]!]) },
      { name: "A label", words: LABEL, sample: (words: string) => facts([{ label: words, value: "Samsung SSD 990 PRO, 2 TB" }, INSTALL[2]!]) },
      { name: "A machine's line", words: DETAIL, sample: (words: string) => facts([{ label: "Disk", value: "Samsung SSD 990 PRO, 2 TB", detail: words }]) },
    ],
    missingPicture: doesntApply("it draws no pictures."),
    counts: [
      { name: "One", for: "A single fact.", sample: () => facts(INSTALL.slice(1, 2)) },
      { name: "Several", for: "`space.s8` apart.", sample: () => facts(INSTALL) },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its card."),
    leaves: doesntApply("it goes with its card."),
    between: doesntApply("it has no states to move between."),
  },
  keyboard: doesntApply("it takes no keyboard; Tab passes it."),
  measures: {
    samples: nonEmpty([{ name: "Last chance", for: "The label's quarter, the gap to the value, the gap between facts, the machine's line under its value.", sample: () => facts(INSTALL) }]),
    measures: [
      { what: "Between facts, and between a label and its value.", token: "space-s8" },
      { what: "Between a value and its machine's line.", token: "space-s4" },
      { what: "A label's line, `small`'s.", token: "type-small-line" },
      { what: "A value's line, `body`'s.", token: "type-body-line" },
      { what: "A machine's line, `mono`'s.", token: "type-mono-line" },
    ],
    reflow: {
      described: "It fills its place at every window size, the label a quarter of it: wider, values run on one line; narrow, labels and values wrap, and a machine's line ends in an ellipsis. Here as wide as the window, then in half a panel.",
      sample: () => stack("s16", "start", facts(INSTALL, "100%"), facts(INSTALL, "calc(var(--size-panel) / 2)"), text("As wide as the window, then half a panel.", "small", "on_surface_variant")),
    },
  },
});
