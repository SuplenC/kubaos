// Network's own values and its organisms: the Wi-Fi networks in range as the control center's
// Wi-Fi tab draws them (its rows, labels, scan skeletons, connection form, off and no-adapter
// states), the saved networks the Known networks page lists, and a row's way to another section.
//
// Its organisms, kept in the Settings app because only it draws them:
// - `inRange`: the control center's Wi-Fi well, as a sub-section's lead part, unscrolled;
// - `savedList`: every saved network, in range or not, in the same rows;
// - `sectionLink`: a row's button that goes to another section of Settings.

import "./network.css";
import { button } from "../../../../lib/button.ts";
import { emptyState } from "../../../../lib/empty-state.ts";
import { type ListItem, type ListRowSpec, list, well } from "../../../../lib/list.ts";
import { connectForm, type Form, WIFI_FORM, WIFI_NEARBY, wifiOff, wifiRows } from "../../../shell/control-center-parts.ts";

/** A Wi-Fi network: its name, whether it asks a password, and its bars in range, 0 out of it. */
export interface Network {
  readonly name: string;
  readonly secured: boolean;
  readonly bars: number;
}

/** The networks NetworkManager has saved, the two in range as the control center's Known has them. */
export const SAVED: readonly Network[] = [
  { name: "Shinjuku-5G", secured: true, bars: 4 },
  { name: "Golden Gai", secured: true, bars: 3 },
  { name: "Kissa", secured: true, bars: 0 },
  { name: "Hoshi 5G", secured: true, bars: 0 },
  { name: "Pixel 9", secured: true, bars: 0 },
  { name: "Narita Free Wi-Fi", secured: false, bars: 0 },
];

/** The bars a signal symbol draws (`Net.strengthIcon`). */
function barsOf(symbol: string): number {
  if (symbol === "signal_wifi_4_bar") return 4;
  return Number(/_(\d)_bar$/.exec(symbol)?.[1] ?? 0);
}

/** The unsaved networks in range: the control center's Nearby. */
const NEARBY: readonly Network[] = WIFI_NEARBY.map((spec) => ({ name: spec.title, secured: spec.subtitle === "Secured", bars: barsOf(spec.icon ?? "") }));

/** The symbol for `bars`, as `Net.strengthIcon` names KubaOS Symbols' wedges. */
function symbolOf(bars: number): string {
  if (bars >= 4) return "signal_wifi_4_bar";
  return bars <= 0 ? "signal_wifi_0_bar" : `network_wifi_${bars}_bar`;
}

/** A nearby network's form, open in the list's place. */
export interface Asking {
  readonly name: string;
  readonly value?: string;
  readonly status?: string;
  readonly failed?: boolean;
}

/** Network's values: its settings, then what the machine has and does now, which a sample sets. */
export interface Net {
  wifi: boolean;
  airplane: boolean;
  bluetooth: boolean;
  unnamed: boolean;
  internetCheck: boolean;
  host: string;
  /** The saved network the Known networks page sets, by its place in `SAVED`. */
  picked: number;
  autoJoin: boolean[];
  privacy: number[];
  metered: number[];
  /** The machine has a Wi-Fi card. */
  adapter: boolean;
  /** The scan: its first still finding nothing, nothing in range besides the saved, or done. */
  scan: "rest" | "scanning" | "none";
  /** The network in use, by name; empty for none. */
  connected: string;
  forgotten: string[];
  asking: Asking | null;
  cable: boolean;
}

/** KubaOS's defaults, and a laptop at a desk on its home network, the cable out. */
export const DEFAULTS: Readonly<Net> = {
  wifi: true,
  airplane: false,
  bluetooth: true,
  unnamed: false,
  internetCheck: true,
  host: "tuxedo",
  picked: 0,
  autoJoin: SAVED.map(() => true),
  privacy: SAVED.map(() => 0),
  metered: SAVED.map(() => 0),
  adapter: true,
  scan: "rest",
  connected: "Shinjuku-5G",
  forgotten: [],
  asking: null,
  cable: false,
};

/** The saved networks not forgotten, each with its place in `SAVED`. */
export function kept(net: Net): { network: Network; index: number }[] {
  return SAVED.map((network, index) => ({ network, index })).filter((each) => !net.forgotten.includes(each.network.name));
}

/** Whether a link is up: the cable, or Wi-Fi on a network. */
export function online(net: Net): boolean {
  return net.cable || (wifiUsable(net) && net.wifi && net.connected !== "");
}

/** Whether Wi-Fi can be turned on at all: a card, and airplane mode off. */
export function wifiUsable(net: Net): boolean {
  return net.adapter && !net.airplane;
}

/** Most signal first, then by name, as `radios.js` orders a tab: a reading nudged a point moves nothing. */
const bySignal = (a: Network, b: Network) => b.bars - a.bars || a.name.localeCompare(b.name);

/** The region drawn again: a part changed what shows. */
function redraw(from: Element): void {
  from.closest(".st-region")?.dispatchEvent(new CustomEvent("st-redraw"));
}

/** A saved network's row, as the control center's Known draws it. */
function knownRow(net: Net, network: Network, line: string, forget: () => void): ListRowSpec {
  const inUse = net.connected === network.name;
  return {
    title: network.name,
    subtitle: inUse ? "Connected · 5 GHz" : line,
    ...(network.secured ? { subtitleIcon: "lock" } : {}),
    icon: symbolOf(network.bars),
    tile: true,
    named: true,
    ...(inUse ? { accent: true, checked: true } : { checked: false, forget }),
  };
}

/** An unsaved network's row, as the control center's Nearby draws it. */
function nearbyRow(network: Network): ListRowSpec {
  return { title: network.name, subtitle: network.secured ? "Secured" : "Open", ...(network.secured ? { subtitleIcon: "lock" } : {}), icon: symbolOf(network.bars), tile: true, checked: false, named: true };
}

/** The form for `asking`: a secured network's password, or an open one's Connect. */
function formOf(asking: Asking): Form {
  const network = [...SAVED, ...NEARBY].find((each) => each.name === asking.name);
  const icon = symbolOf(network?.bars ?? 2);
  if (network && !network.secured) return { icon, title: asking.name, subtitle: "Open network", action: "Connect", focus: "action" };
  return {
    ...WIFI_FORM,
    icon,
    title: asking.name,
    ...(asking.value !== undefined ? { value: asking.value, ready: [...asking.value].length >= 8 } : {}),
    ...(asking.status !== undefined ? { status: asking.status } : {}),
    ...(asking.failed ? { failed: true, focus: "field" as const } : {}),
  };
}

/** A place with nothing to list, in the well the list would be in. */
function emptyWell(made: HTMLElement): HTMLElement {
  const room = document.createElement("span");
  room.className = "nw-empty";
  room.append(made);
  return well("List", room);
}

/**
 * The networks in range, as the control center's Wi-Fi well draws them, drawn whole in
 * the page rather than scrolled in a well of its own. A saved network connects, its × forgets it;
 * an unsaved one opens the connection form in the list's place, the well holding the list's height
 * so nothing under it moves. Off, airplane mode and no card each say so in the well.
 */
export function inRange(net: Net): HTMLElement {
  const holder = document.createElement("span");
  holder.className = "nw-range";
  const turnOn = (label: string, act: () => void) => {
    const made = button({ label, kind: "Tonal", size: "Sm" });
    made.addEventListener("click", () => {
      act();
      redraw(holder);
    });
    return made;
  };
  if (!net.adapter) holder.append(emptyWell(wifiOff(false)));
  else if (net.airplane) holder.append(emptyWell(emptyState({ icon: "wifi_off", title: "Airplane mode is on", body: "Wi-Fi and Bluetooth are off. A cable still works.", action: turnOn("Turn off", () => (net.airplane = false)) })));
  else if (!net.wifi) holder.append(emptyWell(emptyState({ icon: "wifi_off", title: "Wi-Fi is off", body: "Turn it on to see the networks in range.", action: turnOn("Turn on", () => (net.wifi = true)) })));
  else if (net.asking) holder.append(formWell(net, net.asking, holder));
  else holder.append(networkList(net, holder));
  return holder;
}

/** The form in the list's place, its Cancel and Esc back to the list. */
function formWell(net: Net, asking: Asking, holder: HTMLElement): HTMLElement {
  const form = connectForm(formOf(asking));
  const room = document.createElement("span");
  room.className = "nw-form";
  room.append(form);
  const made = well("List", room);
  const back = () => {
    net.asking = null;
    holder.replaceChildren(networkList(net, holder));
  };
  made.querySelector<HTMLElement>(".k-button")?.addEventListener("click", back);
  made.addEventListener("keydown", (event) => event.key === "Escape" && back());
  return made;
}

/** Known then Nearby, a scan's skeletons, or a note, in the control center's own words. */
function networkList(net: Net, holder: HTMLElement): HTMLElement {
  const shown = [...SAVED.filter((each) => each.bars > 0), ...NEARBY];
  const saved = (each: Network) => SAVED.includes(each) && !net.forgotten.includes(each.name);
  const known = shown.filter(saved).sort((a, b) => Number(b.name === net.connected) - Number(a.name === net.connected) || bySignal(a, b));
  const nearby = shown.filter((each) => !saved(each)).sort(bySignal);
  const forget = (name: string) => () => {
    net.forgotten = [...net.forgotten, name];
    redraw(holder);
  };
  const rows = wifiRows(net.scan, net.scan === "none" ? [] : nearby.map(nearbyRow), known.map((each) => knownRow(net, each, "Saved", forget(each.name))));
  // What a row does, by its place among the well's items, its labels counted.
  const actions = rows.map((item: ListItem) => {
    if (item instanceof HTMLElement) return () => {};
    const network = shown.find((each) => each.name === item.title);
    if (!network) return () => {};
    if (known.includes(network)) {
      return () => {
        if (net.connected === network.name) return;
        net.connected = network.name;
        redraw(holder);
      };
    }
    return () => {
      // The well keeps the list's height under the form: measured, as the list's rows decide it.
      holder.style.setProperty("--nw-held", `${holder.getBoundingClientRect().height}px`);
      net.asking = { name: network.name };
      holder.replaceChildren(formWell(net, net.asking, holder));
      holder.querySelector<HTMLElement>(".k-field, .k-button:last-of-type")?.focus({ preventScroll: true });
    };
  });
  return list(rows, { onTake: (index) => actions[index]?.() });
}

/**
 * Every saved network, in range or not, in the control center's rows: the one in use
 * ticked, each other's × forgetting it. A press picks the network the settings under it set, and
 * that row stays filled.
 */
export function savedList(net: Net): HTMLElement {
  const saved = kept(net);
  const holder = document.createElement("span");
  holder.className = "nw-range";
  const forget = (name: string) => () => {
    net.forgotten = [...net.forgotten, name];
    if (SAVED[net.picked]?.name === name) net.picked = kept(net)[0]?.index ?? 0;
    redraw(holder);
  };
  const rows = saved.map(({ network }) => knownRow(net, network, network.bars > 0 ? "In range" : "Not in range", forget(network.name)));
  // The network the settings under it set stays filled, as a pick made with the pointer is.
  holder.append(
    list(rows, {
      current: Math.max(0, saved.findIndex((each) => each.index === net.picked)),
      onTake: (index) => {
        const to = saved[index];
        if (!to || to.index === net.picked) return;
        net.picked = to.index;
        redraw(holder);
      },
    }),
  );
  return holder;
}

/**
 * A row's way to another section of Settings, a Text sm button with the arrow after its
 * words: the section is the sidebar's, so it goes there as the sidebar's row would.
 */
export function sectionLink(label: string, section: string): HTMLElement {
  const made = button({ label, kind: "Text", size: "Sm", icon: "arrow_forward", trailing: true });
  made.classList.add("nw-link");
  made.addEventListener("click", (event) => {
    event.stopPropagation();
    const window = made.closest(".t-window") ?? document;
    const rows = [...window.querySelectorAll<HTMLElement>(".k-sp-row[data-kind='section']")];
    rows.find((each) => each.textContent?.trim() === section)?.click();
  });
  return made;
}
