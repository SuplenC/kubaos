// Kōbō's first boot, card by card: every card of first boot, every view a card can be in (the
// engine's state, a prompt while its work runs, a failure), and what each view asks. The words of
// the cards the setup engine has are its own and the app's (`crates/kubaos-core/src/setup/`,
// `crates/kubaos-kobo/src/setup/`), so the mockup and the build say the same thing; Make it yours
// and Drivers are ahead of the engine, their data stand-ins (`setup-data.ts`).
//
// The common path is upfront on every card: what it is for and what skipping it costs, where it
// stands, and the one thing to press. What only an advanced user wants (the device, the command
// that does the same, the log) is behind the card's own "Under the hood", and an advanced choice
// (another finger, another way to log in) sits where its question is asked.

import type { LogLine } from "../../lib/log.ts";
import type { Duration } from "../../site/page.ts";
import { KNOWN, type Maker, MAKERS } from "./setup-data.ts";

export type CardId = "yours" | "kotodama" | "git" | "drivers" | "fingerprint" | "yubikey" | "phone";

/**
 * The order the rail keeps. Make it yours leads: it is the one card every machine has, and its
 * installs run in the background while the person goes through the rest. Kotodama next, so the one
 * key that finds those apps, and everything else, is learnt while they land. Git after it, while
 * what the machine is for is in mind. Drivers next, so a reboot it needs is known before the
 * cards that only want a finger, a key or a phone.
 */
export const CARD_IDS: readonly CardId[] = ["yours", "kotodama", "git", "drivers", "fingerprint", "yubikey", "phone"];

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
  | { readonly is: "git"; readonly email: string }
  | { readonly is: "parts"; readonly maker: string; readonly ticked: readonly number[] | undefined }
  | { readonly is: "works"; readonly maker: string; readonly fixed?: boolean }
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
export const USER = { name: "Mika Nowak", login: "mika", email: "mika@nowak.dev", host: "tuxedo" };

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

// --------------------------------------------------------------------- Git --

const IDENTITY = "~/.config/git-identity/identity.gitconfig";

/**
 * Git's name and email: there for everyone, since the person who will want it is not always the
 * one who picked Developer, and said plainly to be skippable by anyone who doesn't know what it is.
 */
const git: Card = {
  id: "git",
  title: "Git",
  icon: "git_branch",
  accent: "名前",
  why: "The name and email on every commit you make.",
  skip: "Skip it and git asks for them the first time you commit.",
  later: "Git name and email",
  first: "empty",
  views: {
    empty: { state: "needed", summary: "No name and email for commits yet." },
    filled: { state: "needed", summary: "No name and email for commits yet." },
    invalid: { state: "needed", summary: "No name and email for commits yet." },
    done: { state: "done", summary: `Commits as ${USER.name} <${USER.email}>.` },
  },
  asks: (view) => {
    const plain: Ask = { is: "para", words: "No idea what Git is? Then you don't need it: skip it. It's for writing code, and nothing else here cares." };
    switch (view) {
      case "filled":
        return [plain, { is: "git", email: USER.email }];
      case "invalid":
        return [plain, { is: "git", email: "mika@nowak" }];
      case "done":
        return [{ is: "git", email: USER.email }, { is: "buttons", labels: ["Save changes"] }];
      default:
        return [plain, { is: "git", email: "" }];
    }
  },
  facts: (view) => [
    { label: "Saved to", value: IDENTITY },
    { label: "Included by", value: "~/.config/git/config" },
    { label: "git says", value: view === "done" ? `${USER.name} <${USER.email}>` : "Nothing yet" },
  ],
  command: `git config --file ${IDENTITY} user.email ${USER.email}`,
  action: (view) => (view === "done" ? undefined : { label: "Save", icon: "save" }),
  blocker: (view) => (view === "invalid" ? "That doesn't look like an email." : view === "empty" ? "Your email first, or Skip." : undefined),
  act: { filled: "done" },
  auto: {},
};

// ---------------------------------------------------------------- Drivers --

/** The maker a Drivers view is about: the sample laptop's, unless the view names another. */
function makerOf(view: string): Maker {
  const named: Record<string, string> = { system76: "system76", works: "framework", unknown: "unknown" };
  return MAKERS[named[view] ?? "tuxedo"]!;
}

/** What a maker's needs are, in a line: the parts its drivers bring, as the status says them. */
function needs(maker: Maker): string {
  return `${maker.machine}. Graphics, Wi-Fi and sound work; ${maker.parts[0]!.title.toLowerCase()} need ${maker.name}'s drivers.`;
}

const TUXEDO = MAKERS.tuxedo!;
const TUXEDO_NEEDS = needs(TUXEDO);
const DRIVERS_LOG: readonly LogLine[] = [
  info(`$ pkexec ${HELPER} drivers tuxedo --mok-password-stdin`),
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

const drivers: Card = {
  id: "drivers",
  title: "Drivers",
  icon: "developer_board",
  accent: "ドライバー",
  why: "Some laptops need their maker's drivers before every key, light and fan works. I checked this one.",
  skip: "Skip it and those parts stay as the firmware left them.",
  later: "Set up drivers",
  first: "secure",
  views: {
    secure: { state: "needed", summary: TUXEDO_NEEDS },
    choose: { state: "needed", summary: TUXEDO_NEEDS },
    install: { state: "needed", summary: TUXEDO_NEEDS },
    mismatch: { state: "needed", summary: TUXEDO_NEEDS },
    password: { state: "needed", summary: TUXEDO_NEEDS, prompt: PASSWORD, log: DRIVERS_LOG.slice(0, 1) },
    installing: { state: "needed", summary: TUXEDO_NEEDS, prompt: { kind: "wait", text: "Building the modules with DKMS…" }, log: DRIVERS_LOG.slice(0, 6) },
    queued: { state: "reboot", summary: "Installed. Reboot and enrol their key on the blue MokManager screen.", log: DRIVERS_LOG },
    load: { state: "reboot", summary: "Installed. They load after a reboot." },
    done: { state: "done", summary: `${TUXEDO.machine}. Everything works, the keyboard light and fans included.` },
    system76: { state: "needed", summary: needs(MAKERS.system76!) },
    works: { state: "done", summary: `${MAKERS.framework!.machine}. Everything works with what's already here.` },
    unknown: { state: "done", summary: `${MAKERS.unknown!.machine}. I don't know its maker's drivers, so I've nothing to add.` },
    offline: { state: "waiting", summary: `${TUXEDO_NEEDS} Needs the internet first.` },
    closed: { state: "needed", summary: TUXEDO_NEEDS, failed: { error: "the password dialog was closed", hint: "Nothing changed. Run it again when you're ready." }, log: DRIVERS_LOG.slice(0, 1) },
    nokey: {
      state: "needed",
      summary: "The drivers are in, but Secure Boot won't load them until you enrol their key.",
      failed: { error: "the drivers are in, but DKMS made no signing key to enrol", hint: "Rebuild them so DKMS makes one (sudo dpkg-reconfigure tuxedo-drivers), then try again." },
      log: [...DRIVERS_LOG.slice(0, 7), warn("SKIP: /var/lib/dkms/mok.pub doesn't exist"), error("no key to enrol")],
    },
  },
  asks: (view) => {
    const parts: Ask = { is: "parts", maker: makerOf(view).id, ticked: view === "choose" ? [0, 1] : undefined };
    switch (view) {
      case "secure":
      case "choose":
      case "closed":
        return [parts, SECURE_BOOT, { is: "mok", mismatch: false }];
      case "mismatch":
        return [parts, SECURE_BOOT, { is: "mok", mismatch: true }];
      case "nokey":
        return [SECURE_BOOT, { is: "mok", mismatch: false }];
      case "install":
        return [parts, { is: "para", words: "Secure Boot is off, so they load as they are. No key to enrol." }];
      case "system76":
        return [parts];
      case "queued":
        return [
          {
            is: "banner",
            tone: "Warning",
            icon: "restart_alt",
            title: "Reboot, then enrol the key on the blue screen",
            lines: [
              "Before KubaOS starts, a blue screen called MokManager appears. It waits about 10 seconds; press a key to stop it.",
              "Enroll MOK → Continue → Yes → type the password → Reboot.",
              "Type the one-time password you picked here. If you miss the screen, the drivers won't load: open setup again and enrol the key again.",
            ],
          },
        ];
      case "load":
        return [{ is: "para", words: "They load on the next boot. Reboot when it suits you." }];
      case "done":
        return [{ is: "works", maker: "tuxedo", fixed: true }];
      case "works":
        return [{ is: "works", maker: "framework" }, { is: "para", words: "Nothing to install. Firmware updates come through fwupd, which is on already." }];
      case "unknown":
        return [
          { is: "works", maker: "unknown" },
          { is: "para", words: "Most hardware runs on the kernel's own drivers, and what I checked here does. Something not working? Updates bring new drivers, and Set up drivers in the Command menu checks again." },
        ];
      case "offline":
        return [parts, { is: "para", words: "Connect to a network from the bar, then Check again." }];
      default:
        return [];
    }
  },
  facts: (view) => {
    const maker = makerOf(view);
    if (maker.parts.length === 0) return [
      { label: "Machine", value: maker.machine },
      { label: "Makers I know", value: KNOWN.join(", ") },
      { label: "Installs", value: "Nothing" },
    ];
    const installed = ["done", "queued", "load", "nokey"].includes(view);
    return [
      { label: "Machine", value: maker.machine },
      { label: "From", value: maker.source },
      { label: "Installs", value: maker.parts.filter((part, index) => part.offered || (view === "choose" && index === 1)).map((part) => part.package).join(", ") },
      { label: "Secure Boot", value: view === "install" || maker.id === "system76" ? "Off" : "On" },
      { label: "Signing key", value: view === "queued" ? "Queued for MokManager" : view === "done" ? "Enrolled" : view === "install" || maker.id === "system76" ? "Not needed" : "Not enrolled" },
      { label: "Modules", value: view === "done" ? "tuxedo_keyboard, tuxedo_io, clevo_wmi" : installed ? "Built, not loaded" : "None loaded" },
    ];
  },
  command: `pkexec ${HELPER} drivers tuxedo --mok-password-stdin`,
  action: (view) => {
    if (["install", "secure", "choose", "mismatch", "password", "installing", "system76"].includes(view)) return { label: "Install the drivers", icon: "download" };
    if (view === "closed") return { label: "Try again", icon: "restart_alt" };
    if (view === "nokey") return { label: "Enrol the key", icon: "key" };
    if (view === "queued" || view === "load") return { label: "Reboot now", icon: "restart_alt" };
    if (view === "offline") return { label: "Check again", icon: "restart_alt" };
    return undefined;
  },
  blocker: (view) => (view === "mismatch" ? "The two passwords don't match." : undefined),
  act: { secure: "password", choose: "password", install: "password", closed: "password", offline: "secure", system76: "password" },
  auto: { password: ["installing", "scene"], installing: ["queued", "scene"] },
};

// ----------------------------------------------------------- Make it yours --

/**
 * Make it yours draws its own body, live (`setup-yours-card.ts`): what it asks is the profiles, the
 * apps they bring and the settings that come with them, so it asks nothing through `asks`. Its
 * views are where it starts; its summary here is what the arrival and the ending read.
 */
const yours: Card = {
  id: "yours",
  title: "Make it yours",
  icon: "palette",
  accent: "好み",
  why: "Say what you use it for and I'll install the apps for it. Pick any, all or none.",
  skip: "Skip it and you get KubaOS as it comes: a terminal, a browser, the basics.",
  later: "Make it yours",
  first: "none",
  views: {
    none: { state: "needed", summary: "Nothing picked yet." },
    developer: { state: "needed", summary: "Developer picked." },
    gaming: { state: "needed", summary: "Gaming picked." },
    all: { state: "needed", summary: "All four picked." },
    apps: { state: "needed", summary: "Developer picked, Discord added." },
    "apps-none": { state: "needed", summary: "Nothing picked yet." },
    vscode: { state: "needed", summary: "Developer picked, VS Code added." },
    go: { state: "needed", summary: "Developer picked, two apps added." },
    "go-none": { state: "needed", summary: "Nothing to install." },
    "go-gaming": { state: "needed", summary: "Gaming picked." },
    installing: { state: "needed", summary: "Installing your apps." },
    done: { state: "done", summary: "Your apps are in." },
    failed: { state: "needed", summary: "One app didn't install.", failed: { error: "Steam didn't install: Valve's server timed out", hint: "The rest are in. Try it again, or carry on without it." } },
  },
  asks: () => [],
  facts: () => [],
  command: "kubaos apps install",
  action: (view) => (["none", "installing", "done"].includes(view) ? undefined : view === "failed" ? { label: "Try again", icon: "restart_alt" } : { label: "Install", icon: "download" }),
  act: {},
  auto: {},
};

// ---------------------------------------------------------------- Kotodama --

/**
 * Kotodama, the Command menu, draws its own body, live (`setup-kotodama-card.ts`): what it is, why,
 * its settings, then the menu itself to try. It installs nothing and needs nothing, so every view
 * but the last is the same need: go through it, or skip it.
 */
const kotodama: Card = {
  id: "kotodama",
  title: "Kotodama",
  icon: "search",
  accent: "言霊",
  why: "The Command menu: one key, type what you want, Enter. Everything on this machine is in it.",
  skip: "Skip it and it works as it is: Super + Space, any time.",
  later: "Set up the Command menu",
  first: "what",
  views: {
    what: { state: "needed", summary: "Not seen yet." },
    why: { state: "needed", summary: "Not seen yet." },
    "why-mid": { state: "needed", summary: "Not seen yet." },
    "why-tip": { state: "needed", summary: "Not seen yet." },
    "why-reply": { state: "needed", summary: "Not seen yet." },
    yours: { state: "needed", summary: "Not seen yet." },
    "yours-changed": { state: "needed", summary: "Alt + Space, no web search." },
    try: { state: "needed", summary: "Not tried yet." },
    "try-open": { state: "needed", summary: "Opened." },
    "try-typed": { state: "needed", summary: "Opened, wifi typed." },
    "try-ticked": { state: "needed", summary: "Tried all three." },
    done: { state: "done", summary: "Super + Space, everything in it." },
  },
  asks: () => [],
  facts: () => [],
  command: "kotodama config",
  action: () => undefined,
  act: {},
  auto: {},
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
    starting: { state: "needed", summary: "Off. Set it up to share the clipboard and files with your phone.", found: `Off. Phones can't see this laptop; once it's on, it shows up as ${USER.host}.`, prompt: { kind: "wait", text: "Starting KDE Connect…" } },
    waiting: { state: "waiting", summary: `Waiting for your phone: open KDE Connect on it and tap ${USER.host}.`, found: `On, showing up as ${USER.host}. No phone has asked yet.`, prompt: { kind: "phone", text: `Open KDE Connect on the phone, tap ${USER.host}, then Request pairing.` } },
    address: { state: "waiting", summary: `Waiting for your phone: open KDE Connect on it and tap ${USER.host}.`, found: `On, showing up as ${USER.host}. No phone has asked in 30 s.`, prompt: { kind: "phone", text: `Open KDE Connect on the phone, tap ${USER.host}, then Request pairing.` } },
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

export const CARDS: Readonly<Record<CardId, Card>> = { yours, kotodama, git, drivers, fingerprint, yubikey, phone };

/** The views a card is at: the sample laptop's first boot, with `more` on top. */
export type Views = Readonly<Record<CardId, string>>;

export function firstViews(more: Partial<Views> = {}): Views {
  return { yours: yours.first, kotodama: kotodama.first, git: git.first, drivers: drivers.first, fingerprint: fingerprint.first, yubikey: yubikey.first, phone: phone.first, ...more };
}

/** A desktop of a maker the Drivers card doesn't know, with no reader: one card not there. */
export const TOWER: Partial<Views> = { drivers: "unknown", fingerprint: "absent" };

/** What every card ends at once it was done, for the ending. */
export const ALL_DONE: Views = { yours: "done", kotodama: "done", git: "done", drivers: "done", fingerprint: "done", yubikey: "done", phone: "done" };

/** Whether the card wants attention: `State::open`. */
export function open(view: View): boolean {
  return view.state === "needed" || view.state === "waiting";
}
