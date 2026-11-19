// Settings › Apps: the apps and Suites KubaOS keeps, the default apps, and what starts with you.
// Updates are About's (`settings-about.ts`).

import { screen, type Variant } from "../../../site/page.ts";
import { APPS, type Values } from "./settings-data.ts";
import { sectionPage } from "./settings-section-page.ts";
import { settingsWindow } from "./settings-window.ts";

/** The section's own values over its defaults, under its key. */
const own = (apps: object, more: Partial<Values> = {}): Partial<Values> => ({ seed: { apps }, ...more });

/** The window with sub-page `id` open, with `apps`. */
const opened = (id: string, apps: object) => screen(() => settingsWindow({ section: APPS, values: own(apps, { opened: id }) }));

const VARIANTS: Variant[] = [
  { name: "No startup apps", for: "EmptyState: where an app is added from", sample: opened("startup", { noStartup: true }) },
];

export default sectionPage({
  from: ["empty-state"],
  index: APPS,
  slug: "settings-apps",
  title: "Settings: Apps",
  lede: "The apps KubaOS keeps, what opens what, and what starts with you.",
  hint: "open Start on login; hold a reset",
  seed: own({ solaarStarts: false, pictures: 1 }),
  variants: VARIANTS,
});
