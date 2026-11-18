// Settings › Phone: the phone link, the phone paired with it, and what the two share. Its pairing
// is Kōbō's "Your phone" card's, so a phone pairs the same way at first boot and here.

import { screen, type Variant } from "../../../site/page.ts";
import { PHONE } from "./settings-data.ts";
import { sectionPage } from "./settings-section-page.ts";
import { landedOn, litOn, settingsWindow } from "./settings-window.ts";
import { dialogOver } from "../../../lib/dialog.ts";
import { ASKING, forgetQuestion, type PhoneValues } from "./sections/phone-parts.ts";

const AWAY = { ...ASKING, reachable: false };
const IPAD = { id: "ipad", name: "iPad Air", reachable: false };

/** The window's values with the section's own `phone` over its defaults. */
const seeded = (phone: Partial<PhoneValues>) => ({ seed: { phone } });

/** The window on Your phone with the link on and `phone` reported. */
const yours = (phone: Partial<PhoneValues>) => screen(() => settingsWindow({ section: PHONE, sub: 1, values: seeded({ link: true, ...phone }) }));

/** The window on a sub-page, the link on and `phone` reported. */
const onPage = (opened: string, phone: Partial<PhoneValues>) => () => settingsWindow({ section: PHONE, values: { opened, ...seeded({ link: true, ...phone }) } });

const variants: Variant[] = [
  { name: "Off: the app first", for: "Kōbō's notice; Paired phones waits for the link", sample: screen(() => settingsWindow({ section: PHONE, sub: 1 })) },
  { name: "Waiting for a phone", for: "Kōbō's prompt, humming; nothing to press", sample: yours({}) },
  { name: "Still no phone", for: "after 30 s: the address, and why networks hide it", sample: yours({ pairing: "address" }) },
  { name: "A firewall on", for: "what KDE Connect needs let through", sample: yours({ pairing: "address", firewall: true }) },
  { name: "A phone asks", for: "the key to compare; Accept the pairing", sample: yours({ pairing: "request" }) },
  { name: "Paired, in reach", for: "its row: battery, and Ring", sample: yours({ phones: [ASKING] }) },
  { name: "Paired, away", for: "its row says how to reach it; Ring waits", sample: yours({ phones: [AWAY] }) },
  { name: "KDE Connect not running", for: "Kōbō's error, and Start it again", sample: yours({ pairing: "stopped" }) },
  { name: "Two phones paired", for: "each with Forget", sample: screen(onPage("paired", { phones: [ASKING, IPAD] })) },
  { name: "No phone paired", for: "the empty state says where pairing is", sample: screen(onPage("paired", {})) },
  {
    name: "Forget, asked",
    for: "the Danger dialog over the window; Cancel first",
    sample: screen(() => {
      const frame = settingsWindow({ section: PHONE, values: { opened: "paired", ...seeded({ link: true, phones: [ASKING, IPAD] }) } });
      dialogOver(frame, forgetQuestion(ASKING), () => {}, true);
      return frame;
    }),
  },
  { name: "A risky one allowed", for: "Run commands on: its warning under the well", sample: screen(onPage("may", { commands: true })) },
  { name: "A name it can't take", for: "the field red, the line says why", sample: screen(() => settingsWindow(litOn(PHONE, "shown-name", seeded({ name: "Kuba's (work) laptop" })))) },
  { name: "Landed from search", for: "Received files go to, lit", sample: screen(() => settingsWindow(landedOn(PHONE, "inbox"))) },
];

export default sectionPage({
  from: ["button-bar", "card", "dialog", "empty-state", "error-block"],
  index: PHONE,
  slug: "settings-phone",
  title: "Settings: Phone",
  lede: "The phone link, the phone paired with it, and what the two share.",
  hint: "turn the link off; forget the phone",
  seed: seeded({ link: true, phones: [ASKING], passwords: false, inbox: "~/Phone" }),
  variants,
  measures: [
    { part: "Pairing to the rows", token: "space-s16" },
    { part: "The prompt's symbol", token: "icon-s48" },
  ],
});
