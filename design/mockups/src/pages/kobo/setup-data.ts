// First boot's stand-in data: what each profile proposes on "Make it yours", and the laptop makers
// the Drivers card recognises with what their hardware needs. The real lists come from the apps'
// metadata and the drivers' manifest once they exist; until then these are well-known Linux apps
// and the makers most people run Linux on, so the mockup reads as the real thing would.

/** What the person uses the machine for. Any number of them, none included. */
export type ProfileId = "developer" | "designer" | "casual";

export interface Profile {
  readonly id: ProfileId;
  readonly title: string;
  /** What it covers, in a few plain words. */
  readonly line: string;
  readonly icon: string;
}

/** In the order the cards stand, which is also the order the apps are grouped in. */
export const PROFILES: readonly Profile[] = [
  { id: "developer", title: "Developer", line: "Code, containers, databases.", icon: "code" },
  { id: "designer", title: "Designer", line: "Pictures, vectors, 3D.", icon: "palette" },
  { id: "casual", title: "Everyday", line: "Music, films, chat, office.", icon: "sofa" },
];

export interface App {
  readonly id: string;
  readonly name: string;
  /** What it is for, one line. */
  readonly line: string;
  /** A symbol standing in for the app's own icon. */
  readonly icon: string;
  /** The profiles that propose it; none for one only added by hand. */
  readonly profiles: readonly ProfileId[];
}

/** Every app a profile proposes, then the ones only added by hand, in the order they list. */
export const APPS: readonly App[] = [
  { id: "vscode", name: "Visual Studio Code", line: "Code editor and plugins", icon: "code", profiles: ["developer"] },
  { id: "zed", name: "Zed", line: "Fast editor, in Rust", icon: "bolt", profiles: ["developer"] },
  { id: "docker", name: "Docker", line: "Containers and Compose", icon: "package", profiles: ["developer"] },
  { id: "dbeaver", name: "DBeaver", line: "Every database, one app", icon: "database", profiles: ["developer"] },
  { id: "postman", name: "Postman", line: "Try APIs before coding", icon: "api", profiles: ["developer"] },
  { id: "toolbox", name: "JetBrains Toolbox", line: "RustRover and friends", icon: "tools", profiles: ["developer"] },
  { id: "obsidian", name: "Obsidian", line: "Notes in Markdown files", icon: "notes", profiles: ["developer", "designer"] },
  { id: "bitwarden", name: "Bitwarden", line: "Passwords and SSH keys", icon: "lock", profiles: ["developer", "designer", "casual"] },
  { id: "gimp", name: "GIMP", line: "Edit photos", icon: "photo_edit", profiles: ["designer"] },
  { id: "inkscape", name: "Inkscape", line: "Vector drawings, SVG", icon: "vector_bezier", profiles: ["designer"] },
  { id: "krita", name: "Krita", line: "Paint and draw", icon: "brush", profiles: ["designer"] },
  { id: "blender", name: "Blender", line: "3D: model and render", icon: "cube_3d", profiles: ["designer"] },
  { id: "darktable", name: "darktable", line: "Develop RAW photos", icon: "aperture", profiles: ["designer"] },
  { id: "obs", name: "OBS Studio", line: "Record or stream", icon: "video", profiles: ["designer", "casual"] },
  { id: "spotify", name: "Spotify", line: "Music and podcasts", icon: "music", profiles: ["casual"] },
  { id: "vlc", name: "VLC", line: "Plays any video", icon: "movie", profiles: ["casual"] },
  { id: "signal", name: "Signal", line: "Private messages", icon: "message_circle", profiles: ["casual"] },
  { id: "thunderbird", name: "Thunderbird", line: "Email and calendars", icon: "mail", profiles: ["casual"] },
  { id: "libreoffice", name: "LibreOffice", line: "Docs, sheets, slides", icon: "file_text", profiles: ["casual"] },
  { id: "steam", name: "Steam", line: "Games, Windows ones too", icon: "gamepad", profiles: ["casual"] },
  { id: "kdenlive", name: "Kdenlive", line: "Cut and edit video", icon: "scissors", profiles: [] },
  { id: "audacity", name: "Audacity", line: "Record and edit sound", icon: "wave_sine", profiles: [] },
  { id: "discord", name: "Discord", line: "Voice and chat", icon: "message", profiles: [] },
  { id: "calibre", name: "Calibre", line: "Your e-book library", icon: "books", profiles: [] },
];

export function app(id: string): App {
  const found = APPS.find((each) => each.id === id);
  if (!found) throw new Error(`no app ${id}`);
  return found;
}

/**
 * The apps `picked` propose, each once, grouped by the first picked profile that proposes it, in
 * the profiles' order: an app two profiles share sits with the first of them.
 */
export function proposed(picked: ReadonlySet<ProfileId>): readonly { readonly app: App; readonly by: ProfileId }[] {
  const shown: { app: App; by: ProfileId }[] = [];
  for (const profile of PROFILES) {
    if (!picked.has(profile.id)) continue;
    for (const each of APPS) {
      if (each.profiles.includes(profile.id) && !shown.some((already) => already.app.id === each.id)) shown.push({ app: each, by: profile.id });
    }
  }
  return shown;
}

// ---------------------------------------------------------------- Drivers --

/** One thing a maker's drivers bring, said as what it fixes, then the package that does it. */
export interface Part {
  readonly title: string;
  readonly line: string;
  readonly icon: string;
  readonly package: string;
  /** Ticked as it is offered: what the hardware needs is; an extra app isn't. */
  readonly offered: boolean;
}

/** Something of the machine that works with what the kernel has, said plainly. */
export interface Works {
  readonly title: string;
  readonly found: string;
  readonly icon: string;
}

export interface Maker {
  readonly id: string;
  readonly name: string;
  /** The machine as DMI names it. */
  readonly machine: string;
  /** What it needs from its maker; none when everything works as it is. */
  readonly parts: readonly Part[];
  /** Where the packages come from. */
  readonly source: string;
  /** What works already, as detection found it. */
  readonly works: readonly Works[];
}

const GRAPHICS = (found: string): Works => ({ title: "Graphics", found, icon: "desktop_windows" });
const WIFI = (found: string): Works => ({ title: "Wi-Fi and Bluetooth", found, icon: "wifi" });
const SOUND: Works = { title: "Sound", found: "Speakers, headphones and the microphones", icon: "volume_up" };
const SLEEP: Works = { title: "Sleep and battery", found: "Suspends and wakes; the battery reads right", icon: "battery_full" };

/** The makers whose laptops most people run Linux on, as the card knows them. */
export const MAKERS: Readonly<Record<string, Maker>> = {
  tuxedo: {
    id: "tuxedo",
    name: "TUXEDO",
    machine: "TUXEDO InfinityBook Pro AMD Gen10",
    source: "TUXEDO's own repository",
    parts: [
      { title: "Keyboard light, fans and power limits", line: "Without it they stay wherever the firmware left them", icon: "keyboard", package: "tuxedo-drivers", offered: true },
      { title: "TUXEDO Control Center", line: "An app for fan curves and colours; optional", icon: "mode_fan", package: "tuxedo-control-center", offered: false },
    ],
    works: [GRAPHICS("AMD Radeon 880M"), WIFI("MediaTek MT7925"), SOUND, SLEEP],
  },
  system76: {
    id: "system76",
    name: "System76",
    machine: "System76 Lemur Pro (lemp13)",
    source: "System76's own repository",
    parts: [
      { title: "Hotkeys, fans and keyboard light", line: "The keys along the top, and the fans' curve", icon: "keyboard", package: "system76-driver", offered: true },
      { title: "Battery charge limits", line: "Stop at 80% to make the battery last for years", icon: "battery_full", package: "system76-power", offered: true },
      { title: "Firmware updates", line: "System76's own, from the Command menu", icon: "system_update_alt", package: "system76-firmware-daemon", offered: true },
    ],
    works: [GRAPHICS("Intel Arc"), WIFI("Intel AX211"), SOUND],
  },
  dell: {
    id: "dell",
    name: "Dell",
    machine: "Dell XPS 13 9310",
    source: "Dell's repository for Ubuntu",
    parts: [{ title: "Fingerprint reader", line: "Without it the reader shows up but can't read", icon: "fingerprint", package: "libfprint-2-tod1-goodix", offered: true }],
    works: [GRAPHICS("Intel Iris Xe"), WIFI("Killer AX1650"), SOUND, SLEEP],
  },
  framework: {
    id: "framework",
    name: "Framework",
    machine: "Framework Laptop 13 (AMD Ryzen 7040)",
    source: "Ubuntu",
    parts: [],
    works: [GRAPHICS("AMD Radeon 780M"), WIFI("MediaTek MT7922"), SOUND, SLEEP, { title: "Fingerprint reader", found: "Goodix, through fprintd", icon: "fingerprint" }],
  },
  thinkpad: {
    id: "thinkpad",
    name: "Lenovo",
    machine: "Lenovo ThinkPad T14 Gen 5",
    source: "Ubuntu",
    parts: [],
    works: [GRAPHICS("Intel Arc"), WIFI("Intel BE200"), SOUND, SLEEP, { title: "TrackPoint and keyboard light", found: "The kernel's thinkpad_acpi", icon: "keyboard" }],
  },
  slimbook: {
    id: "slimbook",
    name: "Slimbook",
    machine: "Slimbook Executive 14",
    source: "Slimbook's own repository",
    parts: [{ title: "Slimbook Battery", line: "Power profiles for long days away from a plug", icon: "battery_full", package: "slimbookbattery", offered: false }],
    works: [GRAPHICS("Intel Iris Xe"), WIFI("Intel AX201"), SOUND, SLEEP],
  },
  unknown: {
    id: "unknown",
    name: "Micro-Star",
    machine: "Micro-Star MS-7D25",
    source: "",
    parts: [],
    works: [GRAPHICS("AMD Radeon RX 7800 XT"), WIFI("Intel AX210"), SOUND],
  },
};

/** The makers the card names as the ones it knows. */
export const KNOWN = ["TUXEDO", "System76", "Framework", "Lenovo ThinkPad", "Dell", "Slimbook"];
