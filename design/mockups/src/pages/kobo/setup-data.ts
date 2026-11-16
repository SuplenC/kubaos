// First boot's stand-in data: the catalogue Make it yours shows, every app by its category with
// what each profile proposes, and the laptop makers the Drivers card recognises with what their
// hardware needs. The real lists come from the apps' metadata and the drivers' manifest once they
// exist; until then these are well-known Linux apps and the makers most people run Linux on, so the
// mockup reads as the real thing would.

/** What the person uses the machine for. Any number of them, none included. */
export type ProfileId = "developer" | "designer" | "casual";

export interface Profile {
  readonly id: ProfileId;
  readonly title: string;
  /** What it covers, in a few plain words. */
  readonly line: string;
  readonly icon: string;
}

/** In the order the cards stand. */
export const PROFILES: readonly Profile[] = [
  { id: "developer", title: "Developer", line: "Code, terminals, containers.", icon: "code" },
  { id: "designer", title: "Designer", line: "Pictures, vectors, 3D.", icon: "palette" },
  { id: "casual", title: "Everyday", line: "Music, films, chat, office.", icon: "sofa" },
];

export type CategoryId = "editors" | "terminals" | "dev" | "browsers" | "design" | "media" | "chat" | "office" | "games" | "passwords";

export interface Category {
  readonly id: CategoryId;
  readonly title: string;
  readonly icon: string;
}

/** In the order the catalogue lists them: a developer's first, then everyone's. */
export const CATEGORIES: readonly Category[] = [
  { id: "editors", title: "Editors", icon: "code" },
  { id: "terminals", title: "Terminals", icon: "terminal" },
  { id: "dev", title: "Dev tools", icon: "tools" },
  { id: "browsers", title: "Browsers", icon: "world" },
  { id: "design", title: "Design", icon: "palette" },
  { id: "media", title: "Media", icon: "movie" },
  { id: "chat", title: "Chat", icon: "message_circle" },
  { id: "office", title: "Office and notes", icon: "file_text" },
  { id: "games", title: "Games", icon: "gamepad" },
  { id: "passwords", title: "Passwords", icon: "lock" },
];

export interface App {
  readonly id: string;
  readonly name: string;
  readonly category: CategoryId;
  /** What it is for, one line. */
  readonly line: string;
  /** A symbol standing in for the app's own icon. */
  readonly icon: string;
  /** The profiles that propose it; none for one only ticked by hand. */
  readonly profiles: readonly ProfileId[];
}

/** VS Code's line, and what it says once ticked anyway. */
export const VSCODE = { line: "Microsoft's editor. Phones home, works anyway.", ticked: "Bold. LazyVim is right there." };

const D = "developer";
const G = "designer";
const C = "casual";

const a = (category: CategoryId, id: string, name: string, line: string, icon: string, profiles: readonly ProfileId[] = []): App => ({ id, name, category, line, icon, profiles });

/** Every app the catalogue has, by category, in the order each lists them. */
export const APPS: readonly App[] = [
  a("editors", "lazyvim", "LazyVim", "Neovim, set up properly. Your fingers will thank you", "keyboard", [D]),
  a("editors", "neovim", "Neovim", "Vim, without the 1991 parts", "terminal"),
  a("editors", "helix", "Helix", "Modal editing that works out of the box", "bolt"),
  a("editors", "zed", "Zed", "Fast editor, written in Rust", "bolt", [D]),
  a("editors", "vscode", "Visual Studio Code", VSCODE.line, "code"),
  a("editors", "jetbrains", "JetBrains Toolbox", "RustRover, IntelliJ and friends", "tools"),
  a("editors", "emacs", "Doom Emacs", "An operating system with an editor in it", "draft"),
  a("editors", "sublime", "Sublime Text", "Fast, quiet, paid for once", "draw"),
  a("terminals", "wezterm", "WezTerm", "A terminal you script in Lua", "terminal", [D]),
  a("terminals", "kitty", "kitty", "GPU terminal with pictures in it", "terminal"),
  a("terminals", "alacritty", "Alacritty", "The terminal, nothing else", "terminal"),
  a("terminals", "fish", "fish", "A shell that guesses what you meant", "terminal", [D]),
  a("terminals", "zellij", "Zellij", "Panes and sessions that survive", "list_alt"),
  a("terminals", "tmux", "tmux", "Panes and sessions, the classic", "list_alt"),
  a("terminals", "starship", "Starship", "A prompt that knows your git branch", "rocket_launch", [D]),
  a("dev", "docker", "Docker", "Containers and Compose", "package", [D]),
  a("dev", "podman", "Podman Desktop", "Containers without a daemon", "package"),
  a("dev", "lazygit", "lazygit", "Git in a terminal UI, finally sane", "git_branch", [D]),
  a("dev", "gh", "GitHub CLI", "Pull requests from the terminal", "git_branch", [D]),
  a("dev", "dbeaver", "DBeaver", "Every database, one app", "database", [D]),
  a("dev", "bruno", "Bruno", "Try APIs; collections in plain files", "api", [D]),
  a("dev", "mise", "mise", "The right Node, Python, Rust per project", "tools"),
  a("dev", "k9s", "k9s", "Kubernetes in a terminal", "deployed_code"),
  a("browsers", "firefox", "Firefox", "The browser that isn't Chrome", "world", [D, G, C]),
  a("browsers", "chromium", "Chromium", "Chrome, minus some Google", "world"),
  a("browsers", "brave", "Brave", "Blocks ads before you see them", "world"),
  a("browsers", "zen", "Zen", "Firefox, calmer, tabs on the side", "world"),
  a("design", "krita", "Krita", "Paint and draw; tablets welcome", "brush", [G]),
  a("design", "inkscape", "Inkscape", "Vector drawings, SVG", "vector_bezier", [G]),
  a("design", "gimp", "GIMP", "Edit photos", "photo_edit", [G]),
  a("design", "blender", "Blender", "3D: model, animate, render", "cube_3d", [G]),
  a("design", "darktable", "darktable", "Develop RAW photos", "aperture", [G]),
  a("design", "penpot", "Penpot", "Design and prototype, open source", "layout", [G]),
  a("design", "fontmanager", "Font Manager", "Try fonts before you install them", "typography"),
  a("media", "spotify", "Spotify", "Music and podcasts", "music", [C]),
  a("media", "vlc", "VLC", "Plays any video you throw at it", "movie", [C]),
  a("media", "mpv", "mpv", "A player with no buttons, on purpose", "play_arrow"),
  a("media", "obs", "OBS Studio", "Record or stream your screen", "video", [G]),
  a("media", "kdenlive", "Kdenlive", "Cut and edit video", "scissors"),
  a("media", "audacity", "Audacity", "Record and edit sound", "wave_sine"),
  a("chat", "signal", "Signal", "Private messages, synced with your phone", "message_circle", [C]),
  a("chat", "discord", "Discord", "Voice and chat with your people", "message", [C]),
  a("chat", "telegram", "Telegram", "Chats and channels", "message"),
  a("chat", "slack", "Slack", "Work chat. Sorry", "message"),
  a("chat", "element", "Element", "Matrix: chat you can host yourself", "message_circle"),
  a("office", "libreoffice", "LibreOffice", "Docs, sheets, slides", "file_text", [C]),
  a("office", "thunderbird", "Thunderbird", "Email and calendars", "mail", [C]),
  a("office", "obsidian", "Obsidian", "Notes in Markdown files", "notes", [D, G]),
  a("office", "calibre", "Calibre", "Your e-book library", "books"),
  a("office", "zotero", "Zotero", "Papers and citations, sorted", "books"),
  a("games", "steam", "Steam", "Games, Windows ones too", "gamepad", [C]),
  a("games", "heroic", "Heroic", "Epic and GOG games", "gamepad"),
  a("games", "lutris", "Lutris", "Every other game, set up", "gamepad"),
  a("games", "retroarch", "RetroArch", "Old consoles, one app", "gamepad"),
  a("passwords", "bitwarden", "Bitwarden", "Passwords and SSH keys", "lock", [D, G, C]),
  a("passwords", "keepassxc", "KeePassXC", "Passwords in a file you keep", "key"),
];

export function app(id: string): App {
  const found = APPS.find((each) => each.id === id);
  if (!found) throw new Error(`no app ${id}`);
  return found;
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
