// About: the machine's name and how to change it (`hostnamectl`), where updates stand (`kubaos
// update`), the hardware at a glance and the facts in groups, and Copy all for a bug report. Its one
// page, Updates, holds everything about updating: the card in its states, what an update holds, how
// it checks and installs on its own, the channel and what may sit out, its steps and its history.

import type { Fact } from "../../../../lib/description.ts";
import type { Setting, SettingsSection } from "../settings-parts.ts";
import { own, type Values } from "../settings-values.ts";
import { dialogOver } from "../../../../lib/dialog.ts";
import { section, stack } from "../../../../lib/layout.ts";
import { durationMs } from "../../../../lib/tokens.ts";
import { settings } from "../../../../site/settings.ts";
import { copiedFact, copiedWords, copyAll, factLine, factsCard, factsText, glance, glanceCard, HIDDEN, hardwareFacts, identityFacts, MACHINES, nameCard, renameDialog, restartBanner, sayIn, systemFacts } from "./about-parts.ts";
import { CHANNELS, changesCard, channelQuestion, historyWell, holdsCard, play, rollBackQuestion, stepsWell, type UpdateValues, updateCard } from "./updates-parts.ts";
import { choice, fact, pick, toggle } from "./kit.ts";

/** The section's own values, beside the update's. */
export interface AboutValues extends UpdateValues {
  /** The machine's name now. */
  name: string;
  /** Its name before a rename, until the restart: the banner stands while there is one. */
  was: string | null;
  /** Which sample machine it is (`MACHINES`). */
  machine: number;
  autoCheck: boolean;
  checkEvery: number;
  autoInstall: boolean;
  quietTime: number;
  notice: number;
  /** The hidden facts shown so far, by label: they stay shown for the window. */
  shown: string[];
  /** The fact whose copy button a sample draws shown and lit, by label. */
  pinCopy?: string;
}

/** KubaOS's defaults: what every reset puts back. */
const DEFAULTS: AboutValues = {
  channel: 0,
  firmware: true,
  bitwarden: true,
  nix: true,
  update: "current",
  at: 0,
  logOpen: false,
  version: "0.4.0",
  name: "kuba-pc",
  was: null,
  machine: 0,
  autoCheck: true,
  checkEvery: 0,
  autoInstall: false,
  quietTime: 0,
  notice: 1,
  shown: [],
};

/** How often it checks on its own. */
const EVERY = ["Daily", "Weekly"] as const;
/** When an automatic install runs. */
const QUIET = ["At night", "When idle"] as const;
/** How long before a restart the notice comes. */
const NOTICE = ["5 minutes", "15 minutes", "1 hour"] as const;

/** About, whole. */
export function about(values: Values): SettingsSection {
  const a = own<AboutValues>(values, "about", () => ({ ...DEFAULTS, shown: [] }));
  const machine = MACHINES[a.machine] ?? MACHINES[0]!;
  const shown = new Set(a.shown);
  const frame = () => values.window;
  /** The region drawn again: the cards and the rows follow what changed. */
  const redraw = () => frame()?.querySelector<HTMLElement>(".st-region")?.dispatchEvent(new CustomEvent("st-redraw"));
  /** The keyboard back on `id`'s row once a question is answered. */
  const backTo = (id: string) =>
    requestAnimationFrame(() => {
      const row = frame()?.querySelector<HTMLElement>(`.k-list-row[data-setting="${id}"]`);
      row?.dispatchEvent(new PointerEvent("pointermove"));
      row?.closest<HTMLElement>(".k-list")?.focus({ preventScroll: true });
    });

  const run = () => {
    a.update = "updating";
    a.at = 0;
    a.logOpen = false;
    redraw();
    play(a, frame, redraw);
  };
  // A check asks the release server: the mockup stands its answer in after a scene's length.
  const check = () => {
    a.update = "checking";
    redraw();
    setTimeout(() => {
      if (a.update !== "checking") return;
      a.update = "waiting";
      redraw();
    }, durationMs("scene") / settings().speed);
  };
  const act = {
    check,
    update: run,
    rollBack: () => {
      const window = frame();
      if (!window) return;
      dialogOver(window, rollBackQuestion(a.version), (answer) => {
        if (answer !== "Roll back") return;
        a.update = "current";
        a.logOpen = false;
        redraw();
      });
    },
    open: () => frame()?.querySelector<HTMLElement>(".st-region")?.dispatchEvent(new CustomEvent("st-open", { detail: "updates" })),
  };
  const rename = () => {
    const window = frame();
    if (!window) return;
    renameDialog(window, a.name, (name) => {
      // A second rename before the restart still names the first: that is what open apps know.
      a.was ??= a.name;
      a.name = name;
      if (a.was === a.name) a.was = null;
      redraw();
    });
  };
  // The restart stands in for the machine's own: after it, nothing knows the old name.
  const restart = () => {
    a.was = null;
    redraw();
  };
  const groups = () =>
    [
      ["Hardware", hardwareFacts(machine)],
      ["System", systemFacts(a.version)],
      ["Network identity", identityFacts(machine)],
    ] as const;
  const copy = () => {
    const said = factsText(groups(), a.name, shown);
    navigator.clipboard?.writeText(said.text).catch(() => undefined);
    const window = frame();
    if (window) sayIn(window, copiedWords(said.count, said.left));
  };
  /** One fact copied as "Label: value", the toast saying which. */
  const copyOne = (each: Fact) => {
    navigator.clipboard?.writeText(factLine(each)).catch(() => undefined);
    const window = frame();
    if (window) sayIn(window, copiedFact(each));
  };
  const remember = (label: string) => {
    shown.add(label);
    a.shown = [...shown];
  };

  // A channel is the machine's, written as root, and changes what every update after it takes: it
  // asks first, a reset's included, and changes nothing until it is answered yes.
  const channel = choice("channel", "Channel", "Stable takes signed releases; edge takes main as it lands, bugs included.", CHANNELS, () => a.channel, (index) => {
    const window = frame();
    if (!window || index === a.channel) return;
    dialogOver(window, channelQuestion(index, a.version), (answer) => {
      if (answer.startsWith("Follow")) a.channel = index;
      redraw();
      backTo("channel");
    });
  }, { initial: () => DEFAULTS.channel });

  const automatic: Setting[] = [
    toggle("auto-check", "Check automatically", "Asks for a new release as you, and puts a dot in the bar. Never installs.", () => a.autoCheck, (on) => (a.autoCheck = on), { unbuilt: true, drives: true, initial: () => DEFAULTS.autoCheck }),
    choice("check-every", "How often", "A check takes a moment and downloads nothing.", EVERY, () => a.checkEvery, (index) => (a.checkEvery = index), { unbuilt: true, shows: () => a.autoCheck, initial: () => DEFAULTS.checkEvery }),
    toggle("auto-install", "Install automatically", "Only on mains power, with nothing open that would lose work.", () => a.autoInstall, (on) => (a.autoInstall = on), { unbuilt: true, drives: true, initial: () => DEFAULTS.autoInstall }),
    pick("quiet-time", "When", "At night runs from 02:00; when idle, after an hour away.", QUIET, () => a.quietTime, (index) => (a.quietTime = index), { unbuilt: true, shows: () => a.autoInstall, initial: () => DEFAULTS.quietTime }),
    pick("restart-notice", "Notice before a restart", "A notification first, which can put the restart off.", NOTICE, () => a.notice, (index) => (a.notice = index), { unbuilt: true, shows: () => a.autoInstall, initial: () => DEFAULTS.notice }),
  ];

  const what: Setting[] = [
    channel,
    toggle("firmware", "Firmware", "Docks and receivers update with the rest; what needs a reboot asks first.", () => a.firmware, (on) => (a.firmware = on), { initial: () => DEFAULTS.firmware }),
    toggle("bitwarden", "Bitwarden desktop", "Moves to its series' newest release, checked against GitHub's digest.", () => a.bitwarden, (on) => (a.bitwarden = on), { initial: () => DEFAULTS.bitwarden }),
    toggle("nix", "Nix", "Moves the Nix daemon to its version's newest patch, then restarts it.", () => a.nix, (on) => (a.nix = on), { initial: () => DEFAULTS.nix }),
    fact("suite-tools", "Suites' tools", "Take their patch releases; a minor or major one waits for you.", "Patches"),
  ];

  /** The Updates page's parts before its rows: the card, and what the update holds while one is ready or running. */
  const updatesBefore = () => {
    const parts = [updateCard(a, act)];
    if (a.update === "waiting" || a.update === "updating") parts.push(holdsCard(), changesCard());
    return stack("s16", "start", ...parts);
  };

  return {
    ...(values.opened ? { opened: values.opened } : {}),
    icon: "info",
    title: "About",
    accent: "概要",
    lede: "This machine, by name.",
    subs: [
      {
        title: "This machine",
        before: () => stack("s16", "start", ...(a.was ? [restartBanner(a.was, restart)] : []), nameCard(a.name, machine, a.version, rename)),
        wells: [],
      },
      {
        title: "Updates",
        before: (sub) => glanceCard(a, act, sub.partTip),
        wells: [],
      },
      {
        title: "Hardware",
        body: "What this machine is made of.",
        before: () => stack("s16", "start", glance(machine), factsCard(hardwareFacts(machine), copyOne, [], undefined, a.pinCopy)),
        wells: [],
      },
      {
        title: "System",
        body: "What it runs, and since when.",
        before: () => factsCard(systemFacts(a.version), copyOne, [], undefined, a.pinCopy),
        wells: [],
      },
      {
        title: "Network identity",
        body: "What tells this machine apart; each hidden until you show it.",
        before: () => stack("s16", "start", factsCard(identityFacts(machine), copyOne, HIDDEN, { has: (label: string) => shown.has(label), add: remember }, a.pinCopy), copyAll(copy)),
        wells: [],
      },
    ],
    pages: [
      {
        id: "updates",
        title: "Updates",
        body: "What's waiting, how it comes, and what came before. Nothing installs until you say, unless you let it.",
        back: "About",
        before: updatesBefore,
        wells: [automatic, what],
        after: () =>
          stack(
            "s32",
            "start",
            section("What an update does", "Every step, in the order it runs. Only a failed one stops the rest.", stepsWell(a)),
            section("History", "Every update this machine ran, the newest first.", historyWell()),
          ),
      },
    ],
  };
}
