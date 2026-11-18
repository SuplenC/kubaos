// Apps & updates: what `kubaos update` brings and from which channel (`/etc/kubaos/channel`), what it
// may leave out (the flag files in `desktop/update/`), the apps and Suites KubaOS keeps, what opens
// what (`mimeapps.list`), and what starts with you (`kubaos autostart`).

import type { Setting, SettingsSection } from "../settings-parts.ts";
import { own, type Values } from "../settings-values.ts";
import { dialogOver } from "../../../../lib/dialog.ts";
import { CHANNELS, channelQuestion, noStartup, play, rollBackQuestion, STEPS, type UpdateValues, updateCard, updatePage } from "./apps-parts.ts";
import { choice, fact, opens, pick, toggle } from "./kit.ts";

/** The section's own values, beside the update's. */
interface AppsValues extends UpdateValues {
  dailyCheck: boolean;
  folders: number;
  pictures: number;
  pdfs: number;
  bitwardenStarts: boolean;
  solaarStarts: boolean;
  userDirsStarts: boolean;
  /** No app has an autostart entry at all: a sample's. */
  noStartup: boolean;
}

/** KubaOS's defaults: what every reset puts back. */
const DEFAULTS: AppsValues = {
  channel: 0,
  dailyCheck: true,
  firmware: true,
  bitwarden: true,
  nix: true,
  update: "current",
  at: 0,
  logOpen: false,
  version: "0.4.0",
  folders: 0,
  pictures: 0,
  pdfs: 0,
  bitwardenStarts: true,
  solaarStarts: true,
  userDirsStarts: true,
  noStartup: false,
};

/** How many apps start with you, as Start on login's row says it. */
function startCount(count: number): string {
  if (count === 0) return "None";
  return count === 1 ? "1 app" : `${count} apps`;
}

/** What the update's row says beside its chevron, by where the update stands. */
function stepsDetail(values: AppsValues): string {
  switch (values.update) {
    case "current":
      return `${STEPS.length} steps`;
    case "waiting":
      return "1 waiting";
    case "updating":
      return `Step ${values.at + 1} of ${STEPS.length}`;
    case "failed":
      return `Stopped at ${values.at + 1}`;
  }
}

/** Apps & updates, whole. */
export function apps(values: Values): SettingsSection {
  const a = own(values, "apps", () => ({ ...DEFAULTS }));
  const frame = () => values.window;
  /** The region drawn again: the card and the rows follow what changed. */
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
  const act = {
    check: () => {
      a.update = "waiting";
      redraw();
    },
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

  const startup: Setting[] = [
    toggle("start-bitwarden", "Bitwarden", "Its SSH agent signs for ssh and git, so KubaOS starts it.", () => a.bitwardenStarts, (on) => (a.bitwardenStarts = on), { initial: () => DEFAULTS.bitwardenStarts }),
    toggle("start-solaar", "Solaar", "Logitech's receivers and their batteries; it starts hidden in the tray.", () => a.solaarStarts, (on) => (a.solaarStarts = on), { initial: () => DEFAULTS.solaarStarts }),
    toggle("start-user-dirs", "User folders update", "Ubuntu's: checks Downloads and the rest are where they were.", () => a.userDirsStarts, (on) => (a.userDirsStarts = on), { initial: () => DEFAULTS.userDirsStarts }),
  ];
  const starting = () => (a.noStartup ? 0 : [a.bitwardenStarts, a.solaarStarts, a.userDirsStarts].filter(Boolean).length);

  return {
    ...(values.opened ? { opened: values.opened } : {}),
    icon: "system_update_alt",
    title: "Apps & updates",
    accent: "更新",
    lede: "No surprises, just releases.",
    subs: [
      {
        title: "Updates",
        body: "What's waiting, and where it comes from. Nothing installs until you say.",
        before: () => updateCard(a, act),
        wells: [
          [
            channel,
            toggle("daily-check", "Check every day", "Asks once a day, as you, and puts a dot in the bar. Never installs.", () => a.dailyCheck, (on) => (a.dailyCheck = on), { unbuilt: true, initial: () => DEFAULTS.dailyCheck }),
            opens("update", "What an update does", "Every step in order, and what the next release brings.", "update", () => stepsDetail(a)),
          ],
        ],
      },
      {
        title: "What updates",
        body: "Ubuntu's and your Nix packages always move; these can sit out.",
        wells: [
          [
            toggle("firmware", "Firmware", "Docks and receivers update with the rest; what needs a reboot asks first.", () => a.firmware, (on) => (a.firmware = on), { initial: () => DEFAULTS.firmware }),
            toggle("bitwarden", "Bitwarden desktop", "Moves to its series' newest release, checked against GitHub's digest.", () => a.bitwarden, (on) => (a.bitwarden = on), { initial: () => DEFAULTS.bitwarden }),
            toggle("nix", "Nix", "Moves the Nix daemon to its version's newest patch, then restarts it.", () => a.nix, (on) => (a.nix = on), { initial: () => DEFAULTS.nix }),
            fact("suite-tools", "Suites' tools", "Take their patch releases; a minor or major one waits for you.", "Patches"),
          ],
        ],
      },
      {
        title: "Apps",
        body: "The apps KubaOS keeps, what opens what, and what starts with you.",
        wells: [
          [
            fact("catalog", "From the catalog", "Type an app's name in the Command menu to add or remove it.", "3 of 6"),
            fact("suites", "Suites", "Whole language setups. Type the language in the Command menu.", "Go, Rust"),
          ],
          [
            opens("defaults", "Default apps", "What opens a link, a folder, a picture or a PDF.", "defaults", () => "5 kinds"),
            opens("startup", "Start on login", "The apps that open as you log in.", "startup", () => startCount(starting())),
          ],
        ],
      },
    ],
    pages: [
      {
        id: "update",
        title: "What an update does",
        body: "Every step, in the order it runs. Only a failed one stops the rest.",
        back: "Updates",
        before: () => updatePage(a),
        wells: [],
      },
      {
        id: "defaults",
        title: "Default apps",
        body: "Your pick wins over KubaOS's; a reset hands it back.",
        back: "Apps",
        wells: [
          [
            fact("web", "Web links", "Where links open: the one browser on this machine.", "Google Chrome"),
            pick("folders", "Folders", "Files in a window, or yazi in a terminal.", ["Files", "Yazi"], () => a.folders, (index) => (a.folders = index), { unbuilt: true, initial: () => DEFAULTS.folders }),
            pick("pictures", "Pictures", "PNG, JPEG, WebP and the rest.", ["Image Viewer", "Google Chrome"], () => a.pictures, (index) => (a.pictures = index), { unbuilt: true, initial: () => DEFAULTS.pictures }),
            fact("videos", "Videos", "MP4, MKV, WebM and the rest.", "Video Player"),
            pick("pdfs", "PDFs", "Documents to read, sign or print.", ["Document Viewer", "Google Chrome"], () => a.pdfs, (index) => (a.pdfs = index), { unbuilt: true, initial: () => DEFAULTS.pdfs }),
          ],
        ],
      },
      {
        id: "startup",
        title: "Start on login",
        body: "Counts from your next login; nothing starts or stops now.",
        back: "Apps",
        ...(a.noStartup ? { before: noStartup } : {}),
        wells: a.noStartup ? [] : [startup],
      },
    ],
  };
}
