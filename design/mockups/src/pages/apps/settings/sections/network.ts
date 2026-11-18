// Network: Wi-Fi through NetworkManager, as the control center's Wi-Fi tab uses it, the saved
// networks and how each is joined, Bluetooth's radio, the cable and NetworkManager's internet check,
// and the machine's name and address. The Proxy is a section of its own; a row here goes to it.

import { hostProblem } from "../../../kobo/kobo-parts.ts";
import type { Setting, SettingsSection } from "../settings-parts.ts";
import { own, type Values } from "../settings-values.ts";
import { choice, opens, pick, toggle, unseen } from "./kit.ts";
import { DEFAULTS, inRange, kept, type Net, online, savedList, sectionLink, wifiUsable } from "./network-parts.ts";

const PRIVACY = ["Off", "One for this network", "New each time"] as const;
const METERED = ["Auto", "Yes", "No"] as const;

/** The values whose per-network lists are their own already. */
const separate = new WeakSet<Net>();

/**
 * Network's own values in `values`. `own` copies the defaults and the seed shallowly, so the
 * per-network lists are copied once more: a change to one never reaches the defaults or a seed.
 */
function netOf(values: Values): Net {
  const net = own(values, "network", () => ({ ...DEFAULTS }));
  if (!separate.has(net)) {
    net.autoJoin = [...net.autoJoin];
    net.privacy = [...net.privacy];
    net.metered = [...net.metered];
    separate.add(net);
  }
  return net;
}

/** `setting`, dimmed with the reason `why` gives while it gives one. */
function unless(setting: Setting, why: () => string | null): Setting {
  return Object.defineProperty({ ...setting }, "disabled", { get: () => why() ?? undefined, enumerable: true });
}

/** A fact whose value follows the machine. */
function live(id: string, title: string, line: string, detail: () => string): Setting {
  return { id, title, line, control: { kind: "fact", detail } };
}

/** A saved network's setting, for the network the page sets: its value, its setter, its default, the others'. */
function perNetwork<T>(net: Net, all: T[], initial: readonly T[]) {
  return {
    at: () => all[net.picked]!,
    put: (value: T) => {
      all[net.picked] = value;
    },
    more: { initial: () => initial[net.picked]!, unseen: unseen(all, initial, () => net.picked) },
  };
}

/** The row that picks which saved network the page sets: its list follows what is forgotten. */
function settingsFor(net: Net): Setting {
  return {
    id: "known-for",
    title: "Settings for",
    line: "The saved network the rows under it set.",
    control: {
      kind: "select",
      get options() {
        return kept(net).map((each) => each.network.name);
      },
      picked: () => Math.max(0, kept(net).findIndex((each) => each.index === net.picked)),
      set: (index) => {
        net.picked = kept(net)[index]?.index ?? net.picked;
      },
    },
    drives: true,
  };
}

/** Network, whole. */
export function network(values: Values): SettingsSection {
  const net = netOf(values);
  const join = perNetwork(net, net.autoJoin, DEFAULTS.autoJoin);
  const privacy = perNetwork(net, net.privacy, DEFAULTS.privacy);
  const metered = perNetwork(net, net.metered, DEFAULTS.metered);
  const offFor = (radio: string) => () => (net.airplane ? `Off while airplane mode is on. Turn it off to use ${radio}.` : null);
  return {
    ...(values.opened ? { opened: values.opened } : {}),
    icon: "wifi",
    title: "Network",
    accent: "通信",
    lede: "No strings attached, cables optional.",
    subs: [
      {
        title: "Wi-Fi",
        body: "The networks in range. Press one to join it.",
        before: () => inRange(net),
        wells: [
          [
            unless(
              toggle("wifi", "Wi-Fi", "Off, it neither joins nor looks for networks.", () => net.wifi && wifiUsable(net), (on) => (net.wifi = on), { drives: true, initial: () => DEFAULTS.wifi }),
              () => (!net.adapter ? "This machine has no Wi-Fi card." : net.airplane ? "Off while airplane mode is on." : null),
            ),
            toggle("airplane", "Airplane mode", "Wi-Fi and Bluetooth off together. A cable still works.", () => net.airplane, (on) => (net.airplane = on), { drives: true, unbuilt: true, initial: () => DEFAULTS.airplane }),
            opens("known", "Known networks", "Every network it has joined, and how it joins each.", "known", () => `${kept(net).length} saved`),
          ],
        ],
      },
      {
        title: "Bluetooth",
        body: "Headphones, mice and keyboards. Pairing is in the control center.",
        wells: [
          [
            unless(toggle("bluetooth", "Bluetooth", "Off, paired devices drop and nothing new is found.", () => net.bluetooth && !net.airplane, (on) => (net.bluetooth = on), { initial: () => DEFAULTS.bluetooth }), offFor("Bluetooth")),
            toggle("unnamed", "Show unnamed devices", "Lists devices that send no name, by their address, under Nearby.", () => net.unnamed, (on) => (net.unnamed = on), { initial: () => DEFAULTS.unnamed }),
          ],
        ],
      },
      {
        title: "Cable & internet",
        body: "A cable goes first; Wi-Fi waits as the spare.",
        wells: [
          [
            live("cable", "Cable", "Plugged in, it carries everything while it reaches the internet.", () => (net.cable ? "Connected · 1 Gb/s" : "Unplugged")),
            toggle("internet-check", "Check for internet", "A link with nothing behind it hands over to one that has it. Asks Ubuntu every 5 min.", () => net.internetCheck, (on) => (net.internetCheck = on), {
              initial: () => DEFAULTS.internetCheck,
              warns: () => (net.internetCheck ? null : { title: "A link with no internet keeps the traffic.", body: "A dead cable won't hand over to the Wi-Fi: unplug it." }),
            }),
            { id: "work", title: "Work network", line: "The VPN, the work DNS and the company's proxy have a section of their own.", control: { kind: "part", make: () => sectionLink("Proxy", "Proxy") } },
          ],
        ],
      },
      {
        title: "This machine",
        body: "What it calls itself, and where others find it.",
        wells: [
          [
            {
              id: "host",
              title: "Machine name",
              line: "Its name on the network, to Bluetooth devices and in your prompt.",
              control: {
                kind: "field",
                value: () => net.host,
                placeholder: "tuxedo",
                check: (value) => hostProblem(value) ?? null,
                set: (value) => {
                  if (value !== "") net.host = value;
                },
              },
              unbuilt: true,
            },
            live("address", "Address on this network", "What another machine here types to reach it, as SSH does.", () => (online(net) ? "192.168.1.24" : "Not connected")),
          ],
        ],
      },
    ],
    pages: [
      {
        id: "known",
        title: "Known networks",
        body: "Every network it has joined, in range or not. Pick one to change how it joins; its × forgets it.",
        back: "Wi-Fi",
        before: () => savedList(net),
        wells: [
          [
            settingsFor(net),
            toggle("auto-join", "Join by itself", "Joins whenever it's in range. Off, only when you pick it.", join.at, join.put, join.more),
            pick("privacy", "Private address", "A made-up hardware address, so networks can't follow you from one to the next.", PRIVACY, privacy.at, privacy.put, privacy.more),
            choice("metered", "Metered", "A data plan: the daily update check waits for another network. Auto guesses.", METERED, metered.at, metered.put, { unbuilt: true, ...metered.more }),
          ],
        ],
      },
    ],
  };
}
