// Settings › Apps & updates: the update's card in each of its states and the steps an update runs,
// the channel and what an update may leave out, the apps and Suites KubaOS keeps, the default apps,
// and what starts with you. A channel or a rollback asks first, in the library's Dialog over the window.

import { type DialogOptions, dialogOver } from "../../../lib/dialog.ts";
import { screen, type Variant } from "../../../site/page.ts";
import { APPS, type Values } from "./settings-data.ts";
import { sectionPage } from "./settings-section-page.ts";
import { landedOn, settingsWindow } from "./settings-window.ts";
import { channelQuestion, FAILS_AT, rollBackQuestion } from "./sections/apps-parts.ts";

/** The section's own values over its defaults, under its key. */
const own = (apps: object, more: Partial<Values> = {}): Partial<Values> => ({ seed: { apps }, ...more });

const WAITING = { update: "waiting" };
const UPDATING = { update: "updating", at: 4 };
const FAILED = { update: "failed", at: FAILS_AT, logOpen: true };

/** The window at the Updates sub-section's top with `apps`. */
const updates = (apps: object) => screen(() => settingsWindow({ section: APPS, sub: 0, values: own(apps) }));

/** The window with sub-page `id` open, with `apps`. */
const opened = (id: string, apps: object) => screen(() => settingsWindow({ section: APPS, values: own(apps, { opened: id }) }));

/** The window with `apps`, asking `question` over it, the keyboard where it lands. */
const asking = (apps: object, question: DialogOptions) =>
  screen(() => {
    const frame = settingsWindow({ section: APPS, sub: 0, values: own(apps) });
    dialogOver(frame, question, () => {}, true);
    return frame;
  });

const VARIANTS: Variant[] = [
  { name: "Up to date", for: "the release, its channel, the last update; Check now", sample: updates({}) },
  { name: "An update waiting", for: "the release found, what it brings in a line each; Update", sample: updates(WAITING) },
  { name: "Updating", for: "the step running, its bar and its last line", sample: updates(UPDATING) },
  { name: "Updating, its output open", for: "Show output opens the log under the card", sample: updates({ ...UPDATING, logOpen: true }) },
  { name: "An update failed", for: "ErrorBlock: the step it stopped at; its log; Roll back, Try again", sample: updates(FAILED) },
  { name: "Its steps, waiting", for: "the release's changes, then every step to do", sample: opened("update", WAITING) },
  { name: "Its steps, updating", for: "done, the one running, to do", sample: opened("update", UPDATING) },
  { name: "Its steps, failed", for: "the failed step in `error`; nothing after it ran", sample: opened("update", FAILED) },
  { name: "Its steps on edge", for: "Picks the release: main's newest commit", sample: opened("update", { channel: 1 }) },
  { name: "Its steps, firmware off", for: "Updates firmware: Off", sample: opened("update", { firmware: false }) },
  { name: "Switching to edge", for: "Dialog: what edge means; nothing changes until the next update", sample: asking({}, channelQuestion(1, "0.4.0")) },
  { name: "Back to stable", for: "Dialog: signed releases only, packages alone until one is newer", sample: asking({ channel: 1 }, channelQuestion(0, "0.4.0")) },
  { name: "Rolling back", for: "Dialog: what goes back and what stays", sample: asking(FAILED, rollBackQuestion("0.4.0")) },
  { name: "Channel, landed on", for: "from Home's search: its row lit", sample: screen(() => settingsWindow(landedOn(APPS, "channel"))) },
  { name: "No startup apps", for: "EmptyState: where an app is added from", sample: opened("startup", { noStartup: true }) },
];

export default sectionPage({
  from: ["button-bar", "card", "dialog", "empty-state", "error-block", "log", "step"],
  index: APPS,
  slug: "settings-apps",
  title: "Settings: Apps & updates",
  lede: "What updates, when, from which channel, and what starts with you.",
  hint: "press Update; pick Edge; hold a reset",
  seed: own({ firmware: false, nix: false, solaarStarts: false, ...WAITING }),
  variants: VARIANTS,
  keys: [{ keys: ["Esc"], does: "in a question: Cancel, the channel left as it was" }],
  measures: [
    { part: "The update's card, inside", token: "space-s16" },
    { part: "A question's dialog", token: "size-dialog" },
    { part: "The log, open", token: "size-log" },
  ],
});
