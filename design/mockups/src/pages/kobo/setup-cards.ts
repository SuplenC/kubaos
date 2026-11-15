// Kōbō's first boot, card by card: every card the setup engine has (`kubaos_core::setup`), every
// view a card can be in (the engine's state, a prompt while its work runs, a failure), and what
// each view asks. The words are the engine's and the app's own (`crates/kubaos-core/src/setup/`,
// `crates/kubaos-kobo/src/setup/`), so the mockup and the build say the same thing.
//
// The common path is upfront on every card: what it is for and what skipping it costs, where it
// stands, and the one thing to press. What only an advanced user wants (the device, the command
// that does the same, the log) is behind the card's own "Under the hood", and an advanced choice
// (another finger, another way to log in) sits where its question is asked.

import type { LogLine } from "../../lib/log.ts";
import type { Duration } from "../../site/page.ts";

export type CardId = "tuxedo" | "fingerprint" | "yubikey" | "bitwarden" | "git" | "phone";

/** The engine's order, which the rail keeps. */
export const CARD_IDS: readonly CardId[] = ["tuxedo", "fingerprint", "yubikey", "bitwarden", "git", "phone"];

/** `kubaos_core::setup::State`. */
export type Engine = "done" | "needed" | "reboot" | "waiting" | "absent";

/**
 * What the person has to do right now, shown big: `kubaos_core::setup::PromptKind` while a card's
 * work runs, and a card waiting on hardware (a key to plug in, a phone to ask) the same way.
 */
export type PromptKind = "password" | "finger" | "key" | "wait" | "plug" | "phone" | "pair";

/** A card's work is running: its prompt is the engine's, not a wait on hardware. */
export function running(view: View): boolean {
  return view.prompt !== undefined && !["plug", "phone", "pair"].includes(view.prompt.kind);
}

export interface Prompt {
  readonly kind: PromptKind;
  readonly text: string;
  /** Touches done, of how many: a finger's enrolment. */
  readonly count?: readonly [number, number];
  /** What to compare, alone on its line: a phone's pairing key. */
  readonly key?: string;
}

/** What the card shows: the engine's state and summary, and its run, if one is going or failed. */
export interface View {
  readonly state: Engine;
  readonly summary: string;
  /**
   * What detection found, beside the state where the engine's summary would only say again what
   * the card is for: the machine, the reader, nothing plugged in.
   */
  readonly found?: string;
  /** Something asks for the person now: its words and symbol stand for the state. */
  readonly wants?: { readonly label: string; readonly glyph: string };
  readonly prompt?: Prompt;
  readonly failed?: { readonly error: string; readonly hint: string };
  /** What a run that finished found, said in its own slot: the phone's test copy. */
  readonly result?: string;
  readonly log?: readonly LogLine[];
}

/** The `Filled` button's work on a view: its words and symbol, and what stops it. */
export interface Action {
  readonly label: string;
  readonly icon: string;
}

/** What a card asks on a view, by the parts `setup-parts.ts` draws. */
export type Ask =
  | { readonly is: "para"; readonly words: string }
  | { readonly is: "banner"; readonly tone: "Info" | "Warning"; readonly icon: string; readonly title: string; readonly lines: readonly string[] }
  | { readonly is: "notice"; readonly icon: string; readonly title: string; readonly lines: readonly string[] }
  | { readonly is: "mok"; readonly mismatch: boolean }
  | { readonly is: "finger" }
  | { readonly is: "fingers"; readonly names: readonly string[] }
  | { readonly is: "bitwarden"; readonly method: "totp" | "apikey"; readonly server: number; readonly missing?: "code" }
  | { readonly is: "unlock" }
  | { readonly is: "keys" }
  | { readonly is: "git"; readonly email: string }
  | { readonly is: "buttons"; readonly labels: readonly string[] };

export interface Card {
  readonly id: CardId;
  readonly title: string;
  readonly icon: string;
  /** The small Japanese label over its title. */
  readonly accent: string;
  /** What it is for, one plain sentence. */
  readonly why: string;
  /** What skipping it costs, one plain sentence. */
  readonly skip: string;
  /** The Command menu's Item that does it later. */
  readonly later: string;
  readonly views: Readonly<Record<string, View>>;
  /** The view on the sample laptop at its first boot. */
  readonly first: string;
  readonly asks: (view: string) => readonly Ask[];
  /** What detection found, for Under the hood. */
  readonly facts: (view: string) => readonly { readonly label: string; readonly value: string }[];
  /** The command that does the same in a terminal. */
  readonly command: string;
  readonly action: (view: string) => Action | undefined;
  /** Why the `Filled` button waits, on a view with a form. */
  readonly blocker?: (view: string) => string | undefined;
  /** Live: where the `Filled` button leads from a view. */
  readonly act: Readonly<Record<string, string>>;
  /** Live: views that move on by themselves, after a duration's worth of it. */
  readonly auto: Readonly<Record<string, readonly [string, Duration]>>;
}

/** The sample: Mika's TUXEDO, Secure Boot on, a Goodix reader, a YubiKey Bio in a drawer. */
export const USER = { name: "Mika Nowak", login: "mika", email: "mika@nowak.dev", host: "tuxedo", product: "TUXEDO InfinityBook Pro AMD Gen10" };

const PASSWORD: Prompt = { kind: "password", text: "Your password, in the dialog that just opened." };
const HELPER = "/usr/lib/kubaos/setup-helper";

/** What the person sees under a prompt's words: where to do it. */
export const PROMPT_WHERE: Record<PromptKind, string> = {
  password: "polkit's dialog. Your password, or your finger on the enrolled key.",
  finger: "On the laptop's reader.",
  key: "On the YubiKey's own sensor, not the laptop's.",
  wait: "Nothing for you to do. Under the hood has the log, if you like that sort of thing.",
  plug: "Then Check again. Or skip it; I'll be here.",
  phone: "The request shows up here within a few seconds. No need to press anything.",
  pair: "Accept only a phone you're holding: other KDE Connect devices on this network can ask too.",
};

const step = (text: string): LogLine => ({ level: "step", text });
const info = (text: string): LogLine => ({ level: "info", text });
const warn = (text: string): LogLine => ({ level: "warn", text });
const error = (text: string): LogLine => ({ level: "error", text });

// ---------------------------------------------------------------- TUXEDO --

const TUXEDO_NEEDS = "Keyboard backlight, fans and power limits need TUXEDO's drivers.";
const TUXEDO_LOG: readonly LogLine[] = [
  info(`$ pkexec ${HELPER} tuxedo --mok-password-stdin`),
  step("Adding TUXEDO's apt repository"),
  info("ok  deb.tuxedocomputers.com/ubuntu resolute main"),
  step("Installing tuxedo-drivers and the kernel headers"),
  info("ok  tuxedo-drivers 4.12.1, linux-headers-7.0.0-43-generic"),
  step("Building the modules with DKMS"),
  info("ok  tuxedo_keyboard, tuxedo_io, clevo_wmi"),
  step("Queueing the signing key for MokManager"),
];

const SECURE_BOOT: Ask = {
  is: "banner",
  tone: "Warning",
  icon: "lock",
  title: "Secure Boot is on",
  lines: ["Pick a one-time password. You type it once, on a blue screen after the reboot, so the firmware trusts the drivers."],
};

/** The machine as detection found it, beside the state. */
const MACHINE = (secureBoot: boolean, drivers: string) => `${USER.product}. Secure Boot ${secureBoot ? "on" : "off"}, ${drivers}.`;

const tuxedo: Card = {
  id: "tuxedo",
  title: "TUXEDO drivers",
  icon: "developer_board",
  accent: "ドライバー",
  why: "TUXEDO's own drivers run the keyboard backlight, the fans and the power limits.",
  skip: "Skip it and those stay wherever the firmware left them.",
  later: "Set up TUXEDO drivers",
  first: "secure",
  views: {
    install: { state: "needed", summary: TUXEDO_NEEDS, found: MACHINE(false, "no drivers yet") },
    secure: { state: "needed", summary: TUXEDO_NEEDS, found: MACHINE(true, "no drivers yet") },
    mismatch: { state: "needed", summary: TUXEDO_NEEDS, found: MACHINE(true, "no drivers yet") },
    password: { state: "needed", summary: TUXEDO_NEEDS, found: MACHINE(true, "no drivers yet"), prompt: PASSWORD, log: TUXEDO_LOG.slice(0, 1) },
    installing: { state: "needed", summary: TUXEDO_NEEDS, found: MACHINE(true, "no drivers yet"), prompt: { kind: "wait", text: "Building the modules with DKMS…" }, log: TUXEDO_LOG.slice(0, 6) },
    queued: { state: "reboot", summary: "Key queued. Reboot and enroll it on the blue MokManager screen.", log: TUXEDO_LOG },
    load: { state: "reboot", summary: "Installed. The drivers load after a reboot." },
    done: { state: "done", summary: "Drivers loaded: tuxedo_keyboard, tuxedo_io, clevo_wmi." },
    offline: { state: "waiting", summary: `${TUXEDO_NEEDS} Needs the internet first.`, found: `${MACHINE(true, "no drivers yet")} Needs the internet first.` },
    closed: { state: "needed", summary: TUXEDO_NEEDS, found: MACHINE(true, "no drivers yet"), failed: { error: "the password dialog was closed", hint: "Nothing changed. Run it again when you're ready." }, log: TUXEDO_LOG.slice(0, 1) },
    nokey: {
      state: "needed",
      summary: "Drivers are in, but Secure Boot won't load them until you enroll their key.",
      failed: { error: "the drivers are in, but DKMS made no signing key to enroll", hint: "Rebuild them so DKMS makes one (sudo dpkg-reconfigure tuxedo-drivers), then try again." },
      log: [...TUXEDO_LOG.slice(0, 7), warn("SKIP: /var/lib/dkms/mok.pub doesn't exist"), error("no key to enroll")],
    },
    absent: { state: "absent", summary: "Not on this machine: Dell Inc. isn't a TUXEDO." },
  },
  asks: (view) => {
    switch (view) {
      case "install":
        return [{ is: "para", words: "Secure Boot is off, so the drivers load as they are. No key to enroll." }];
      case "secure":
      case "closed":
        return [SECURE_BOOT, { is: "mok", mismatch: false }];
      case "mismatch":
        return [SECURE_BOOT, { is: "mok", mismatch: true }];
      case "nokey":
        return [SECURE_BOOT, { is: "mok", mismatch: false }];
      case "queued":
        return [
          {
            is: "banner",
            tone: "Warning",
            icon: "restart_alt",
            title: "Reboot, then enroll the key on the blue screen",
            lines: [
              "Before KubaOS starts, a blue screen called MokManager appears. It waits about 10 seconds; press a key to stop it.",
              "Enroll MOK → Continue → Yes → type the password → Reboot.",
              "Type the one-time password you picked here. If you miss the screen, the drivers won't load: open setup again and enroll the key again.",
            ],
          },
        ];
      case "load":
        return [{ is: "para", words: "They load on the next boot. Reboot when it suits you." }];
      case "offline":
        return [{ is: "para", words: "Connect to a network from the bar, then Check again." }];
      case "absent":
        return [{ is: "para", words: "Nothing to install. The TUXEDO drivers only make sense on a TUXEDO." }];
      default:
        return [];
    }
  },
  facts: (view) =>
    view === "absent"
      ? [
          { label: "Machine", value: "Dell Inc. XPS 13 9340" },
          { label: "Architecture", value: "amd64" },
        ]
      : [
          { label: "Machine", value: USER.product },
          { label: "Installs", value: "tuxedo-drivers from TUXEDO's repository; no Control Center, power profiles do that job" },
          { label: "Secure Boot", value: view === "install" ? "Off" : "On" },
          { label: "Drivers", value: ["done", "queued", "load", "nokey"].includes(view) ? "tuxedo-drivers 4.12.1" : "Not installed" },
          { label: "Signing key", value: view === "queued" ? "Queued for MokManager" : view === "done" ? "Enrolled" : view === "install" ? "Not needed" : "Not enrolled" },
          { label: "Modules", value: view === "done" ? "tuxedo_keyboard, tuxedo_io, clevo_wmi" : "None loaded" },
        ],
  command: `pkexec ${HELPER} tuxedo --mok-password-stdin`,
  action: (view) => {
    if (["install", "secure", "mismatch", "password", "installing"].includes(view)) return { label: "Install the drivers", icon: "download" };
    if (view === "closed") return { label: "Try again", icon: "restart_alt" };
    if (view === "nokey") return { label: "Enroll the key", icon: "key" };
    if (view === "queued" || view === "load") return { label: "Reboot now", icon: "restart_alt" };
    if (view === "offline") return { label: "Check again", icon: "restart_alt" };
    return undefined;
  },
  blocker: (view) => (view === "mismatch" ? "The two passwords don't match." : undefined),
  act: { secure: "password", install: "password", closed: "password", offline: "secure" },
  auto: { password: ["installing", "scene"], installing: ["queued", "scene"] },
};

// ------------------------------------------------------------- Fingerprint --

const READER = "Goodix MOC Fingerprint Sensor (27c6:609c)";
const NO_FINGER = "Goodix reader (27c6:609c). No finger enrolled yet.";
const ENROLL_LOG: readonly LogLine[] = [
  info(`$ pkexec ${HELPER} fingerprint`),
  step("Installing fprintd"),
  info("ok  fprintd, libpam-fprintd"),
  info("ok  pam-auth-update --enable fprintd"),
  info("$ fprintd-enroll -f right-index-finger"),
  info("Enroll result: enroll-stage-passed"),
];

const fingerprint: Card = {
  id: "fingerprint",
  title: "Fingerprint",
  icon: "fingerprint",
  accent: "指紋",
  why: "Your finger instead of your password at sudo and in password dialogs.",
  skip: "Skip it and you type the password, as ever.",
  later: "Enroll a fingerprint",
  first: "enroll",
  views: {
    enroll: { state: "needed", summary: NO_FINGER },
    password: { state: "needed", summary: NO_FINGER, prompt: PASSWORD, log: ENROLL_LOG.slice(0, 1) },
    touch: { state: "needed", summary: NO_FINGER, prompt: { kind: "finger", text: "Put your right index finger on the reader.", count: [0, 8] }, log: ENROLL_LOG.slice(0, 5) },
    touching: { state: "needed", summary: NO_FINGER, prompt: { kind: "finger", text: "Lift it, touch again. Shift it a little each time.", count: [3, 8] }, log: ENROLL_LOG },
    last: { state: "needed", summary: NO_FINGER, prompt: { kind: "finger", text: "Lift it. Once more.", count: [7, 8] }, log: ENROLL_LOG },
    done: { state: "done", summary: "1 finger: right index. sudo and password dialogs take it." },
    pam: { state: "needed", summary: "1 finger enrolled, but sudo doesn't ask for it yet." },
    failed: { state: "needed", summary: NO_FINGER, failed: { error: "fprintd-enroll ended without finishing", hint: "Run it again and keep touching until it says done." }, log: [...ENROLL_LOG, error("Enroll result: enroll-disconnected")] },
    absent: { state: "absent", summary: "Not on this machine: no fingerprint reader." },
  },
  asks: (view) => {
    switch (view) {
      case "enroll":
      case "failed":
        return [{ is: "para", words: "The password keeps working, so a cut finger is a bad day, not a lockout. You'll touch the reader eight-ish times; move the finger a little each time." }, { is: "finger" }];
      case "done":
        return [{ is: "para", words: "sudo, polkit dialogs and the tty login take your finger first and wait 10 seconds before they ask for the password." }, { is: "fingers", names: ["Right index"] }];
      case "pam":
        return [{ is: "para", words: "Enroll a finger and setup turns it on for sudo and password dialogs too." }, { is: "finger" }];
      case "absent":
        return [{ is: "para", words: "No reader that fprintd knows. Plugged one in? Check again from a fresh start of setup." }];
      default:
        return [];
    }
  },
  facts: (view) =>
    view === "absent"
      ? [{ label: "Reader", value: "None found" }]
      : [
          { label: "Reader", value: READER },
          { label: "fprintd", value: ["enroll", "password"].includes(view) ? "Not installed yet" : "Installed" },
          { label: "Fingers", value: view === "done" || view === "pam" ? "Right index" : "None" },
          { label: "sudo and polkit", value: view === "done" ? "Take it" : "Don't ask for it yet" },
        ],
  command: "fprintd-enroll -f right-index-finger",
  action: (view) => {
    if (view === "failed") return { label: "Try again", icon: "restart_alt" };
    if (view === "done" || view === "absent") return undefined;
    return { label: "Enroll a finger", icon: "fingerprint" };
  },
  act: { enroll: "password", failed: "password", pam: "touch" },
  auto: { password: ["touch", "scene"], touch: ["touching", "scene"], touching: ["last", "scene"], last: ["done", "scene"] },
};

// ----------------------------------------------------------------- YubiKey --

const KEY_LOG: readonly LogLine[] = [info("$ fido2-token -L"), info("/dev/hidraw4: vendor=0x1050, product=0x0402 (Yubico YubiKey Bio)"), info("$ pamu2fcfg -n -u mika -V")];

const yubikey: Card = {
  id: "yubikey",
  title: "YubiKey",
  icon: "key",
  accent: "鍵",
  why: "Your YubiKey and a finger on its sensor stand in for the password at sudo, polkit and the lock screen.",
  skip: "Skip it and the password does all of that, as ever.",
  later: "Enroll a security key",
  first: "plug",
  views: {
    plug: { state: "waiting", summary: "Plug in your YubiKey to use it for sudo, polkit and the lock screen.", found: "No YubiKey plugged in, none enrolled yet.", prompt: { kind: "plug", text: "Plug in your YubiKey." } },
    enroll: { state: "needed", summary: "YubiKey Bio plugged in, not enrolled yet." },
    prep: { state: "needed", summary: "This key needs a PIN and a fingerprint of its own first." },
    touch: { state: "needed", summary: "YubiKey Bio plugged in, not enrolled yet.", prompt: { kind: "key", text: "Put an enrolled finger on the key's sensor when it blinks." }, log: KEY_LOG },
    password: { state: "needed", summary: "YubiKey Bio plugged in, not enrolled yet.", prompt: PASSWORD, log: [...KEY_LOG, info(`$ pkexec ${HELPER} u2f-add`)] },
    done: { state: "done", summary: "This key is enrolled (one key in total). Enroll a backup key too: keys get lost." },
    two: { state: "waiting", summary: "More than one key plugged in. Leave only the one to enroll." },
    failed: { state: "needed", summary: "YubiKey Bio plugged in, not enrolled yet.", failed: { error: "the key didn't confirm", hint: "Wrong finger, or too slow. After 3 bad scans the key wants its PIN: unplug it, plug it back, try again." }, log: [...KEY_LOG, error("pamu2fcfg: unable to generate registration")] },
  },
  asks: (view) => {
    const backup: Ask = { is: "para", words: "The password always keeps working. Enroll a second key as the backup: keys get lost." };
    switch (view) {
      case "plug":
        return [backup];
      case "enroll":
      case "failed":
        return [backup];
      case "prep":
        return [
          {
            is: "banner",
            tone: "Warning",
            icon: "key",
            title: "The key needs its own setup first",
            lines: [
              "It has no FIDO2 PIN or no fingerprint yet, and it won't enroll without both. That part talks to the key directly, so it runs in a terminal:",
              "ykman fido access change-pin, then ykman fido fingerprints add right-index (and a finger from the other hand).",
              "Come back and press Check again.",
            ],
          },
        ];
      case "done":
        return [{ is: "para", words: "Got a backup key? Plug it in instead of this one and Check again." }, { is: "buttons", labels: ["Check again"] }];
      case "two":
        return [{ is: "para", words: "Unplug the others, then Check again." }];
      default:
        return [];
    }
  },
  facts: (view) => [
    { label: "Plugged in", value: view === "plug" ? "Nothing" : view === "two" ? "2 keys" : "YubiKey Bio (FIDO edition)" },
    { label: `Enrolled for ${USER.login}`, value: view === "done" ? "1 key, this one" : "None" },
    { label: "PIN and fingers", value: view === "prep" ? "No PIN yet" : view === "plug" ? "Unknown" : "PIN set, 2 fingers" },
    { label: "Mappings", value: "/etc/u2f_mappings" },
  ],
  command: `pamu2fcfg -n -u ${USER.login} -V | pkexec ${HELPER} u2f-add`,
  action: (view) => {
    if (view === "plug" || view === "two") return { label: "Check again", icon: "restart_alt" };
    if (view === "prep") return { label: "Set it up in a terminal", icon: "terminal" };
    if (view === "failed") return { label: "Try again", icon: "restart_alt" };
    if (view === "done") return undefined;
    return { label: "Enroll this key", icon: "key" };
  },
  act: { plug: "enroll", enroll: "touch", failed: "touch", two: "enroll" },
  auto: { touch: ["password", "scene"], password: ["done", "scene"] },
};

// --------------------------------------------------------------- Bitwarden --

const BW_LOG: readonly LogLine[] = [
  step("Logging in to bitwarden.eu"),
  info("ok  logged in as mika@nowak.dev"),
  step("Writing SSH keys"),
  info("ok  ~/.ssh/github.pub, ~/.ssh/hetzner.pub"),
  info("ok  Host blocks in ~/.ssh/config.d/bitwarden"),
  step("Setting up commit signing"),
  info("ok  git signs with github"),
  step("Logging gh in"),
  info("ok  gh as mikanowak"),
  step("Writing project env"),
  info("ok  3 projects"),
];

const bitwarden: Card = {
  id: "bitwarden",
  title: "Bitwarden",
  icon: "lock",
  accent: "金庫",
  why: "Log in once and your SSH keys, commit signing, the gh token and your projects' env come from the vault.",
  skip: "Skip it and none of that is here: no SSH keys, unsigned commits, gh logged out.",
  later: "Set up Bitwarden",
  first: "login",
  views: {
    login: { state: "needed", summary: "Not logged in." },
    code: { state: "needed", summary: "Not logged in." },
    apikey: { state: "needed", summary: "Not logged in." },
    unlock: { state: "needed", summary: `Logged in as ${USER.email}, but nothing's pulled from the vault yet.` },
    pulling: { state: "needed", summary: "Not logged in.", prompt: { kind: "wait", text: "Writing SSH keys…" }, log: BW_LOG.slice(0, 4) },
    done: { state: "done", summary: `Logged in as ${USER.email}. SSH keys, commit signing and project env come from the vault.`, log: BW_LOG },
    offline: { state: "waiting", summary: "Not logged in. Needs the internet first." },
    wrong: { state: "needed", summary: "Not logged in.", failed: { error: "wrong email or master password", hint: "Check both and try again, slower this time." }, log: [BW_LOG[0]!, error("Username or password is incorrect. Try again.")] },
    absent: { state: "absent", summary: "Not on this machine: bw isn't installed (system/70-bitwarden.sh does it)." },
  },
  asks: (view) => {
    switch (view) {
      case "login":
      case "wrong":
        return [{ is: "bitwarden", method: "totp", server: 0 }];
      case "code":
        return [{ is: "bitwarden", method: "totp", server: 0, missing: "code" }];
      case "apikey":
        return [{ is: "bitwarden", method: "apikey", server: 2 }];
      case "unlock":
        return [{ is: "unlock" }];
      case "offline":
        return [{ is: "para", words: "Connect to a network from the bar, then Check again." }];
      case "done":
        return [
          { is: "keys" },
          { is: "para", words: "Bitwarden desktop starts when you close setup. Last bit by hand: log in there, Settings, turn on “Enable SSH agent”. It has no config file for that." },
          { is: "buttons", labels: ["Pull again", "Log out"] },
        ];
      case "absent":
        return [{ is: "para", words: "Nothing to do here." }];
      default:
        return [];
    }
  },
  facts: (view) => [
    { label: "Server", value: view === "apikey" ? "https://vault.nowak.dev" : "bitwarden.eu" },
    { label: "Account", value: view === "unlock" || view === "done" ? USER.email : "None yet" },
    { label: "bw", value: view === "absent" ? "Not installed" : view === "done" ? "Unlocked" : view === "unlock" ? "Locked" : "Logged out" },
    { label: "Desktop app", value: "Installed, starts after setup" },
  ],
  command: "kubaos bitwarden setup",
  action: (view) => {
    if (view === "done" || view === "absent") return undefined;
    if (view === "offline") return { label: "Check again", icon: "restart_alt" };
    if (view === "wrong") return { label: "Try again", icon: "restart_alt" };
    if (view === "unlock") return { label: "Unlock and pull", icon: "lock_open" };
    return { label: "Log in and pull", icon: "lock_open" };
  },
  blocker: (view) => (view === "code" ? "The two-step code." : undefined),
  act: { login: "pulling", wrong: "pulling", unlock: "pulling", apikey: "pulling", offline: "login" },
  auto: { pulling: ["done", "slow"] },
};

// --------------------------------------------------------------------- Git --

const IDENTITY = "~/.config/git-identity/identity.gitconfig";

const git: Card = {
  id: "git",
  title: "Git",
  icon: "code",
  accent: "名前",
  why: "The name and email git puts on every commit.",
  skip: "Skip it and git refuses to commit until it has them.",
  later: "Git name and email",
  first: "fill",
  views: {
    fill: { state: "needed", summary: "No name and email for commits yet." },
    invalid: { state: "needed", summary: "No name and email for commits yet." },
    done: { state: "done", summary: `Commits as ${USER.name} <${USER.email}>.` },
    wins: {
      state: "needed",
      summary: "No name and email for commits yet.",
      failed: { error: "git still says user.email = mika@old-job.example", hint: "~/.gitconfig sets it and wins over the include. Delete the [user] lines there, or change them there." },
      log: [info(`ok  ${IDENTITY}: ${USER.name} <${USER.email}>`)],
    },
  },
  asks: (view) => {
    switch (view) {
      case "invalid":
        return [{ is: "git", email: "mika@nowak" }];
      case "done":
        return [{ is: "git", email: USER.email }, { is: "buttons", labels: ["Save changes"] }];
      default:
        return [{ is: "git", email: USER.email }];
    }
  },
  facts: (view) => [
    { label: "Saved to", value: IDENTITY },
    { label: "Included by", value: "~/.config/git/config" },
    { label: "git says", value: view === "done" ? `${USER.name} <${USER.email}>` : view === "wins" ? "user.email = mika@old-job.example" : "Nothing yet" },
  ],
  command: `git config --file ${IDENTITY} user.email ${USER.email}`,
  action: (view) => (view === "done" ? undefined : view === "wins" ? { label: "Try again", icon: "restart_alt" } : { label: "Save", icon: "save" }),
  blocker: (view) => (view === "invalid" ? "That doesn't look like an email." : undefined),
  act: { fill: "done", wins: "done" },
  auto: {},
};

// ------------------------------------------------------------------ Phone --

const PHONE_KEY = "5A1C 9E07 33B2";

const phone: Card = {
  id: "phone",
  title: "Your phone",
  icon: "smartphone",
  accent: "電話",
  why: "Share the clipboard and files with your phone, iPhone or Android, over KDE Connect.",
  skip: "Skip it and nothing on this laptop listens for phones: the link stays off.",
  later: "Set up a phone",
  first: "off",
  views: {
    off: { state: "needed", summary: "Off. Set it up to share the clipboard and files with your phone.", found: `Off. Phones can't see this laptop; once it's on, it shows up as ${USER.host}.` },
    starting: { state: "needed", summary: "Off. Set it up to share the clipboard and files with your phone.", prompt: { kind: "wait", text: "Starting KDE Connect…" } },
    waiting: { state: "waiting", summary: `Waiting for your phone: open KDE Connect on it and tap ${USER.host}.`, prompt: { kind: "phone", text: `Open KDE Connect on the phone, tap ${USER.host}, then Request pairing.` } },
    address: { state: "waiting", summary: `Waiting for your phone: open KDE Connect on it and tap ${USER.host}.`, prompt: { kind: "phone", text: `Open KDE Connect on the phone, tap ${USER.host}, then Request pairing.` } },
    request: { state: "needed", summary: "Pixel 9 wants to pair. Check the key, then accept.", wants: { label: "Wants to pair", glyph: "smartphone" }, prompt: { kind: "pair", text: "Does the phone show the same key?", key: PHONE_KEY } },
    pairing: { state: "needed", summary: "Pixel 9 wants to pair. Check the key, then accept.", prompt: { kind: "wait", text: "Pairing…" } },
    done: { state: "done", summary: "Paired with Pixel 9, in reach, battery 82%." },
    sent: { state: "done", summary: "Paired with Pixel 9, in reach, battery 82%.", result: "Sent “Hello from KubaOS”. Paste it on Pixel 9: that's what should show up." },
    stopped: { state: "needed", summary: "On, but KDE Connect isn't running." },
    failed: { state: "needed", summary: "On, but KDE Connect isn't running.", failed: { error: "KDE Connect isn't running", hint: "journalctl --user -u kubaos-phone says why. Turning the phone link off and on again restarts it." } },
  },
  asks: (view) => {
    switch (view) {
      case "off":
        return [
          { is: "para", words: "It stays off until you set it up: until then nothing on this laptop listens for phones or announces itself on the network." },
          { is: "notice", icon: "download", title: "First, on the phone", lines: ["iPhone and iPad: KDE Connect from the App Store.", "Android: KDE Connect from Google Play or F-Droid.", `Same Wi-Fi as the laptop. It will show up there as ${USER.host}.`] },
        ];
      case "address":
        return [{ is: "notice", icon: "public", title: "No phone yet?", lines: ["Some networks, at offices and hotels, keep devices from finding each other. Restarting the app on the phone helps sometimes.", "Or add the laptop by its address in the phone's app (Add devices by IP): 192.168.1.23."] }];
      case "done":
      case "sent":
        return [
          {
            is: "notice",
            icon: "check_circle",
            title: "What works each way",
            lines: [
              "Laptop to phone: copy here and it's on the phone's clipboard. Files go from the Command menu's Send a file to the phone.",
              "Phone to laptop: open KDE Connect and tap Send clipboard, or share to KDE Connect. iOS never sends it on its own.",
            ],
          },
          { is: "buttons", labels: ["Send a test copy", "Check what the phone sent", "Turn the phone link off"] },
        ];
      case "stopped":
      case "failed":
        return [{ is: "para", words: "Start it again; if it stops once more, journalctl --user -u kubaos-phone says why." }];
      default:
        return [];
    }
  },
  facts: (view) => [
    { label: "Shows up as", value: USER.host },
    { label: "Address", value: "192.168.1.23" },
    { label: "KDE Connect", value: view === "off" ? "Off" : view === "stopped" || view === "failed" ? "Not running" : "Running" },
    { label: "Paired", value: view === "done" || view === "sent" ? "Pixel 9" : "None" },
  ],
  command: "kubaos-phone on",
  action: (view) => {
    if (view === "off") return { label: "Set up a phone", icon: "smartphone" };
    if (view === "starting") return { label: "Set up a phone", icon: "smartphone" };
    if (view === "waiting" || view === "address") return { label: "Check again", icon: "restart_alt" };
    if (view === "request" || view === "pairing") return { label: "Accept the pairing", icon: "check" };
    if (view === "stopped" || view === "failed") return { label: "Start KDE Connect again", icon: "restart_alt" };
    return undefined;
  },
  act: { off: "starting", request: "pairing", stopped: "starting", failed: "starting" },
  auto: { starting: ["waiting", "scene"], waiting: ["request", "slow"], pairing: ["done", "scene"] },
};

export const CARDS: Readonly<Record<CardId, Card>> = { tuxedo, fingerprint, yubikey, bitwarden, git, phone };

/** The views a card is at: the sample laptop's first boot, with `more` on top. */
export type Views = Readonly<Record<CardId, string>>;

export function firstViews(more: Partial<Views> = {}): Views {
  return { tuxedo: tuxedo.first, fingerprint: fingerprint.first, yubikey: yubikey.first, bitwarden: bitwarden.first, git: git.first, phone: phone.first, ...more };
}

/** A desktop that isn't a TUXEDO and has no reader: two cards not there. */
export const TOWER: Partial<Views> = { tuxedo: "absent", fingerprint: "absent" };

/** What every card ends at once it was done, for the ending. */
export const ALL_DONE: Views = { tuxedo: "done", fingerprint: "done", yubikey: "done", bitwarden: "done", git: "done", phone: "done" };

/** Whether the card wants attention: `State::open`. */
export function open(view: View): boolean {
  return view.state === "needed" || view.state === "waiting";
}
