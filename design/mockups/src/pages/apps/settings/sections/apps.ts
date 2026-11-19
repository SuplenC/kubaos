// Apps: the apps and Suites KubaOS keeps, what opens what (`mimeapps.list`), and what starts with
// you (`kubaos autostart`). Updates live in About › Updates.

import type { Setting, SettingsSection } from "../settings-parts.ts";
import { own, type Values } from "../settings-values.ts";
import { emptyState } from "../../../../lib/empty-state.ts";
import { fact, opens, pick, toggle } from "./kit.ts";

/** The section's own values. */
interface AppsValues {
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

/** Start on login with nothing in it, and where an app is added from. */
function noStartup(): HTMLElement {
  return emptyState({ icon: "rocket_launch", title: "Nothing starts with you", body: "In the Command menu, find an app and pick Start on login from its Actions." });
}

/** Apps, whole. */
export function apps(values: Values): SettingsSection {
  const a = own(values, "apps", () => ({ ...DEFAULTS }));

  const startup: Setting[] = [
    toggle("start-bitwarden", "Bitwarden", "Its SSH agent signs for ssh and git, so KubaOS starts it.", () => a.bitwardenStarts, (on) => (a.bitwardenStarts = on), { initial: () => DEFAULTS.bitwardenStarts }),
    toggle("start-solaar", "Solaar", "Logitech's receivers and their batteries; it starts hidden in the tray.", () => a.solaarStarts, (on) => (a.solaarStarts = on), { initial: () => DEFAULTS.solaarStarts }),
    toggle("start-user-dirs", "User folders update", "Ubuntu's: checks Downloads and the rest are where they were.", () => a.userDirsStarts, (on) => (a.userDirsStarts = on), { initial: () => DEFAULTS.userDirsStarts }),
  ];
  const starting = () => (a.noStartup ? 0 : [a.bitwardenStarts, a.solaarStarts, a.userDirsStarts].filter(Boolean).length);

  return {
    ...(values.opened ? { opened: values.opened } : {}),
    icon: "app_window",
    title: "Apps",
    accent: "アプリ",
    lede: "Yours to keep, yours to open.",
    subs: [
      {
        title: "Installed",
        body: "The apps and language setups KubaOS keeps for you.",
        wells: [
          [
            fact("catalog", "From the catalog", "Type an app's name in the Command menu to add or remove it.", "3 of 6"),
            fact("suites", "Suites", "Whole language setups. Type the language in the Command menu.", "Go, Rust"),
          ],
        ],
      },
      {
        title: "Defaults & startup",
        body: "What opens what, and what starts with you.",
        wells: [
          [
            opens("defaults", "Default apps", "What opens a link, a folder, a picture or a PDF.", "defaults", () => "5 kinds"),
            opens("startup", "Start on login", "The apps that open as you log in.", "startup", () => startCount(starting())),
          ],
        ],
      },
    ],
    pages: [
      {
        id: "defaults",
        title: "Default apps",
        body: "Your pick wins over KubaOS's; a reset hands it back.",
        back: "Defaults & startup",
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
        back: "Defaults & startup",
        ...(a.noStartup ? { before: noStartup } : {}),
        wells: a.noStartup ? [] : [startup],
      },
    ],
  };
}
