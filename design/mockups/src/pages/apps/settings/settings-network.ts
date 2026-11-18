// Settings › Network: Wi-Fi, Bluetooth's radio, the cable and the internet check, and the machine's
// name and address. Its own samples are the networks in range in each state the control center's
// Wi-Fi tab has, airplane mode, no card, the cable in, the internet check off and a wrong name.

import { screen } from "../../../site/page.ts";
import type { Net } from "./sections/network-parts.ts";
import { NETWORK } from "./settings-data.ts";
import { sectionPage } from "./settings-section-page.ts";
import type { Values } from "./settings-values.ts";
import { settingsWindow } from "./settings-window.ts";

/** A window's values with Network's own starting as `net` says. */
const seeded = (net: Partial<Net>): Partial<Values> => ({ seed: { network: net } });

/** The window on Network scrolled to sub-section `sub` with `net`. */
const at = (sub: number, net: Partial<Net>) => screen(() => settingsWindow({ section: NETWORK, sub, values: seeded(net) }));

/** The window on Network's Wi-Fi with `net`, the networks in range in view. */
const wifi = (net: Partial<Net>) => at(0, net);

// The rows the samples show are scrolled to, not lit: a lit or landed row is found by its place
// among the page's rows, which the networks' rows in range come before.
const CABLE = 2;
const MACHINE = 3;

/** Pixel 9's hotspot: joined only when picked, metered. Its place in `SAVED`. */
const HOTSPOT = 4;

export default sectionPage({
  from: ["empty-state"],
  index: NETWORK,
  slug: "settings-network",
  title: "Settings: Network",
  lede: "Wi-Fi and the saved networks, Bluetooth's radio, the cable, and what the machine calls itself.",
  hint: "press a network; hold a reset to put it back",
  seed: seeded({ unnamed: true, bluetooth: false, picked: HOTSPOT, autoJoin: [true, true, true, true, false, true], metered: [0, 0, 0, 0, 1, 0] }),
  variants: [
    { name: "Scanning", for: "the first scan: Nearby's skeletons", sample: wifi({ scan: "scanning" }) },
    { name: "Nothing else in range", for: "Known alone; Nearby says so", sample: wifi({ scan: "none" }) },
    { name: "Asking a password", for: "the control center's form, in the list's place", sample: wifi({ asking: { name: "Omoide Yokocho" } }) },
    { name: "A wrong password", for: "the field invalid, the status in `error`", sample: wifi({ asking: { name: "Omoide Yokocho", value: "hunter22", status: "Wrong password", failed: true } }) },
    { name: "Wi-Fi off", for: "the empty state turns it on", sample: wifi({ wifi: false }) },
    { name: "Airplane mode", for: "both radios off, their rows say why", sample: wifi({ airplane: true }) },
    { name: "No Wi-Fi card", for: "nothing to join; the switch says why", sample: wifi({ adapter: false }) },
    { name: "A network forgotten", for: "Golden Gai moves to Nearby", sample: wifi({ forgotten: ["Golden Gai"] }) },
    { name: "Another network joined", for: "Golden Gai first, ticked", sample: wifi({ connected: "Golden Gai" }) },
    { name: "A saved network's settings", for: "Pixel 9: by itself off, metered", sample: screen(() => settingsWindow({ section: NETWORK, values: { opened: "known", ...seeded({ picked: HOTSPOT, autoJoin: [true, true, true, true, false, true], metered: [0, 0, 0, 0, 1, 0] }) } })) },
    { name: "A cable in", for: "the cable carries it", sample: at(CABLE, { cable: true }) },
    { name: "Internet check off", for: "the warning under its well", sample: at(CABLE, { internetCheck: false }) },
    { name: "A name that can't be", for: "the line says the rule, in `error`", sample: at(MACHINE, { host: "Nerv.HQ" }) },
    { name: "Offline", for: "no address to give", sample: at(MACHINE, { wifi: false }) },
  ],
  keys: [
    { keys: ["Delete"], does: "on a saved network: forgets it, as its × does" },
    { keys: ["Esc"], does: "in the connection form: back to the list" },
  ],
  measures: [
    { part: "A network's row", token: "size-row_two_line" },
    { part: "The form's tile", token: "size-control_lg" },
    { part: "Empty state, above and below", token: "space-s24" },
  ],
});
