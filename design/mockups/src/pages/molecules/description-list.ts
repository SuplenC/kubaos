// The DescriptionList page: `docs/design/components/description-list.md`. Facts beside their labels,
// as a summary reads them: Kōbō's Last chance cards.

import type { Words } from "../../lib/content.ts";
import { type Fact, descriptionList } from "../../lib/description.ts";
import { stack, within } from "../../lib/layout.ts";
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
  lede: "Facts beside their labels, as a summary reads them: a label a quarter of the width, its value the rest.",
  code: { rust: "description::description_list", qml: "DescriptionList" },
  stage: {
    live: () => facts(INSTALL, "100%"),
    hint: "words, not a control: Tab passes it",
    daylight: true,
  },
  variants: [
    { name: "Facts", for: "a label and a value each", sample: () => facts(INSTALL.slice(1, 2)) },
    { name: "With a machine's line", for: "a path, an email or a version, in `mono`", sample: () => facts([INSTALL[0]!, INSTALL[3]!]) },
  ],
  sizes: doesntApply("as wide as its place, as tall as its facts."),
  states: {
    shown: ["rest"],
    absent: {
      lit: "words, not a control",
      pressed: "takes no press",
      disabled: "nothing in it turns off",
      selected: "nothing in it is picked",
      loading: "its facts are known before it shows",
      empty: "with nothing to say, it isn't shown",
      error: "what failed is an `ErrorBlock`",
    },
    rows: [{ name: "Last chance", cell: () => facts(INSTALL) }],
    looks: { rest: "labels and machine lines `on_surface_variant`, values `on_surface`" },
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
      { name: "One", for: "a single fact", sample: () => facts(INSTALL.slice(1, 2)) },
      { name: "Several", for: "`space.s8` apart", sample: () => facts(INSTALL) },
    ],
  },
  motion: {
    arrives: doesntApply("it comes with its card."),
    leaves: doesntApply("it goes with its card."),
    between: doesntApply("it has no states to move between."),
  },
  keyboard: doesntApply("it takes no keyboard; Tab passes it."),
  measures: {
    samples: nonEmpty([{ name: "Last chance", for: "the label's quarter, the gaps, the machine's line", sample: () => facts(INSTALL) }]),
    measures: [
      { part: "Between facts, label → value", token: "space-s8" },
      { part: "Value → machine's line", token: "space-s4" },
      { part: "Label line", token: "type-small-line" },
      { part: "Value line", token: "type-body-line" },
      { part: "Machine's line", token: "type-mono-line" },
    ],
    reflow: {
      note: "The label stays a quarter: narrow, words wrap and a machine's line ends in an ellipsis.",
      sample: () => stack("s16", "start", facts(INSTALL, "100%"), facts(INSTALL, "calc(var(--size-panel) / 2)")),
    },
  },
});
