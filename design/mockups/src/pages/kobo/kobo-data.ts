// Kōbō's sample machine and its answers: a TUXEDO laptop with Windows on its one disk, the
// languages, layouts, time zones and networks the live system offers, the wallpaper sets in the
// payload, and an install's steps, timings and log. The words are Kōbō's own (`crates/kubaos-kobo`).

import kabukicho from "../../../../../crates/kubaos-kobo/assets/wallpaper.jpg?url";
import sign from "../../../../../branding/photo/neon-sign-banner.jpg?url";
import hero from "../../../../../branding/png/kubaos-hero.jpg?url";
import purple from "../../assets/lock/shinjuku-purple-hour.jpg?url";
import rain from "../../assets/lock/shibuya-rainy-corner.jpg?url";
import tower from "../../assets/lock/tokyo-tower-blue-hour.jpg?url";
import type { Words } from "../../lib/content.ts";

/** The release on the stick, as its manifest names it. */
export const RELEASE = { version: "0.4.0", codename: "Shinjuku", commit: "42c4611", built: "2026-11-12", kernel: "6.17.0-5-generic", arch: "x86_64" } as const;

/** A language the live system offers, and what follows from it until the person changes it. */
export interface Language {
  readonly name: string;
  readonly locale: string;
  /** Its keyboard layout, by its index in `LAYOUTS`. */
  readonly layout: number;
  /** Its time zone, by its index in `ZONES`. */
  readonly zone: number;
  /** The locale `Intl` formats its sample date and numbers in. */
  readonly tag: string;
}

export interface Layout {
  readonly name: string;
  readonly code: string;
}

export const LAYOUTS: readonly Layout[] = [
  { name: "English (US)", code: "us" },
  { name: "English (US, international)", code: "us(intl)" },
  { name: "English (UK)", code: "gb" },
  { name: "Polish", code: "pl" },
  { name: "German", code: "de" },
  { name: "French", code: "fr" },
  { name: "Italian", code: "it" },
  { name: "Spanish", code: "es" },
  { name: "Japanese", code: "jp" },
  { name: "Dvorak", code: "us(dvorak)" },
  { name: "Colemak", code: "us(colemak)" },
];

/** A time zone: its city, its region, and its IANA name. */
export interface Zone {
  readonly city: string;
  readonly region: string;
  readonly iana: string;
}

export const ZONES: readonly Zone[] = [
  { city: "New York", region: "America", iana: "America/New_York" },
  { city: "London", region: "Europe", iana: "Europe/London" },
  { city: "Warsaw", region: "Europe", iana: "Europe/Warsaw" },
  { city: "Berlin", region: "Europe", iana: "Europe/Berlin" },
  { city: "Paris", region: "Europe", iana: "Europe/Paris" },
  { city: "Rome", region: "Europe", iana: "Europe/Rome" },
  { city: "Madrid", region: "Europe", iana: "Europe/Madrid" },
  { city: "Tokyo", region: "Asia", iana: "Asia/Tokyo" },
  { city: "Los Angeles", region: "America", iana: "America/Los_Angeles" },
  { city: "Lisbon", region: "Europe", iana: "Europe/Lisbon" },
  { city: "UTC", region: "", iana: "UTC" },
];

export const LANGUAGES: readonly Language[] = [
  { name: "English (US)", locale: "en_US.UTF-8", layout: 0, zone: 0, tag: "en-US" },
  { name: "English (UK)", locale: "en_GB.UTF-8", layout: 2, zone: 1, tag: "en-GB" },
  { name: "Polski", locale: "pl_PL.UTF-8", layout: 3, zone: 2, tag: "pl-PL" },
  { name: "Deutsch", locale: "de_DE.UTF-8", layout: 4, zone: 3, tag: "de-DE" },
  { name: "Français", locale: "fr_FR.UTF-8", layout: 5, zone: 4, tag: "fr-FR" },
  { name: "Italiano", locale: "it_IT.UTF-8", layout: 6, zone: 5, tag: "it-IT" },
  { name: "Español", locale: "es_ES.UTF-8", layout: 7, zone: 6, tag: "es-ES" },
  { name: "日本語", locale: "ja_JP.UTF-8", layout: 8, zone: 7, tag: "ja-JP" },
];

/** The moment the samples show: a Saturday afternoon in November. */
export const SAMPLE_TIME = new Date(Date.UTC(2026, 10, 14, 16, 48));

/** `zone`'s name as a select shows it: the city, then its region. */
export function zoneName(zone: Zone): string {
  return zone.region ? `${zone.city} · ${zone.region}` : zone.city;
}

/** A Wi-Fi network in range. */
export interface Network {
  readonly name: string;
  /** 0 to 100. */
  readonly signal: number;
  readonly secured: boolean;
}

export const NETWORKS: readonly Network[] = [
  { name: "Hoshi 5G", signal: 92, secured: true },
  { name: "Kissa", signal: 71, secured: true },
  { name: "Shinjuku Free Wi-Fi", signal: 48, secured: false },
  { name: "Vodafone-A1B2", signal: 33, secured: true },
  { name: "DIRECT-roku-417", signal: 12, secured: true },
];

/** A network's signal as a Wi-Fi symbol. */
export function signalIcon(signal: number): string {
  if (signal >= 75) return "signal_wifi_4_bar";
  if (signal >= 50) return "network_wifi_3_bar";
  if (signal >= 25) return "network_wifi_2_bar";
  return "network_wifi_1_bar";
}

/** A partition on a disk, its size in bytes. */
export interface Partition {
  readonly name: string;
  readonly bytes: number;
  /** What it is to the person: Windows is theirs; the small ones Windows keeps are its plumbing. */
  readonly kind: "system" | "plumbing" | "data";
}

/** A disk the probe found. */
export interface Disk {
  readonly model: string;
  readonly path: string;
  readonly bytes: number;
  readonly partitions: readonly Partition[];
  /** The largest free region, where KubaOS goes alongside. */
  readonly free: number;
  /** The other system on it, if any. */
  readonly other?: string;
  /** The stick Kōbō runs from: listed, never a target. */
  readonly stick?: boolean;
}

const GB = 1e9;

/** What KubaOS takes, the payload's installed size and room to grow: under it, a disk is too small. */
export const NEEDED = 17.4 * GB;

export const LAPTOP_DISK: Disk = {
  model: "Samsung 990 Pro",
  path: "/dev/nvme0n1",
  bytes: 2000 * GB,
  other: "Windows 11",
  free: 818.8 * GB,
  partitions: [
    { name: "EFI system", bytes: 0.26 * GB, kind: "plumbing" },
    { name: "Microsoft reserved", bytes: 0.016 * GB, kind: "plumbing" },
    { name: "Windows 11", bytes: 1180 * GB, kind: "system" },
    { name: "Windows recovery", bytes: 0.75 * GB, kind: "plumbing" },
  ],
};

/** The same disk with no room left beside Windows. */
export const FULL_DISK: Disk = { ...LAPTOP_DISK, free: 4.2 * GB, partitions: LAPTOP_DISK.partitions.map((each) => (each.kind === "system" ? { ...each, bytes: 1994.6 * GB } : each)) };

export const EMPTY_DISK: Disk = { model: "WD Black SN850X", path: "/dev/nvme0n1", bytes: 1000 * GB, free: 1000 * GB, partitions: [] };

export const DATA_DISK: Disk = { model: "Crucial MX500", path: "/dev/sda", bytes: 500 * GB, free: 0, partitions: [{ name: "data", bytes: 500 * GB, kind: "data" }] };

export const SMALL_DISK: Disk = { model: "SD card", path: "/dev/mmcblk0", bytes: 16 * GB, free: 16 * GB, partitions: [] };

export const STICK: Disk = { model: "SanDisk Extreme", path: "/dev/sda", bytes: 64 * GB, free: 0, partitions: [], stick: true };

/** Bytes in the units a person reads, through the locale's number formatting. */
export function bytes(count: number): string {
  const unit = (value: number, name: "terabyte" | "gigabyte" | "megabyte", digits: number) =>
    new Intl.NumberFormat("en-GB", { style: "unit", unit: name, unitDisplay: "short", maximumFractionDigits: digits }).format(value);
  if (count >= 1000 * GB) return unit(count / (1000 * GB), "terabyte", 1);
  if (count >= 10 * GB) return unit(count / GB, "gigabyte", 0);
  if (count >= GB) return unit(count / GB, "gigabyte", 1);
  return unit(count / 1e6, "megabyte", 0);
}

/** A wallpaper in the payload's sets. */
export interface Wallpaper {
  readonly name: string;
  readonly picture: string;
  readonly video?: boolean;
}

export interface WallpaperSet {
  readonly name: string;
  readonly wallpapers: readonly Wallpaper[];
}

export const SETS: readonly WallpaperSet[] = [
  {
    name: "Shinjuku",
    wallpapers: [
      { name: "kabukicho rain", picture: kabukicho },
      { name: "purple hour", picture: purple },
      { name: "shibuya rainy corner", picture: rain },
    ],
  },
  { name: "Tokyo", wallpapers: [{ name: "tokyo tower blue hour", picture: tower }] },
  {
    name: "KubaOS",
    wallpapers: [
      { name: "neon sign", picture: sign, video: true },
      { name: "the desk", picture: hero },
    ],
  },
];

/** One step of the install, as the engine plans it, and how long it took on the sample laptop. */
export interface InstallStep {
  readonly title: string;
  readonly seconds: number;
  /** What it shows while it runs: how much of the system is copied. */
  readonly detail?: string;
}

export const STEPS: readonly InstallStep[] = [
  { title: "Checking the machine", seconds: 2 },
  { title: "Partitioning", seconds: 3 },
  { title: "Encrypting", seconds: 6 },
  { title: "Creating filesystems", seconds: 2 },
  { title: "Copying the system", seconds: 161, detail: "4.1 / 9.4 GB" },
  { title: "Configuring", seconds: 18 },
  { title: "Creating your account", seconds: 3 },
  { title: "Installing the bootloader", seconds: 9 },
  { title: "Activating your home", seconds: 24 },
  { title: "Taking the factory snapshot", seconds: 4 },
  { title: "Finishing", seconds: 2 },
];

/** The whole install's seconds. */
export const TOTAL = STEPS.reduce((sum, each) => sum + each.seconds, 0);

/** A duration as the log and the steps print it: 2.4s, 18s, 2m 41s. */
export function duration(seconds: number): string {
  if (seconds < 10) return `${seconds.toFixed(1)}s`;
  if (seconds < 60) return `${Math.round(seconds)}s`;
  const whole = Math.round(seconds);
  return `${Math.floor(whole / 60)}m ${String(whole % 60).padStart(2, "0")}s`;
}

/** The step running `share` of the way through the install, and how far through it. */
export function stepAt(share: number): { readonly index: number; readonly within: number } {
  let left = share * TOTAL;
  for (let index = 0; index < STEPS.length; index++) {
    const seconds = STEPS[index]!.seconds;
    if (left < seconds) return { index, within: left / seconds };
    left -= seconds;
  }
  return { index: STEPS.length - 1, within: 1 };
}

/** The install's log, as the engine writes it, up to the step running. */
export const LOG: readonly { readonly step: number; readonly level: "step" | "info" | "warn" | "error"; readonly text: string }[] = [
  { step: 0, level: "step", text: "==> Checking the machine" },
  { step: 0, level: "info", text: "UEFI, Secure Boot on · 32 GB RAM · AC power" },
  { step: 1, level: "step", text: "==> Partitioning" },
  { step: 1, level: "info", text: "sgdisk --new=5:2363392:+818G --typecode=5:8309 /dev/nvme0n1" },
  { step: 2, level: "step", text: "==> Encrypting" },
  { step: 2, level: "info", text: "cryptsetup luksFormat --type luks2 --pbkdf argon2id /dev/nvme0n1p5" },
  { step: 3, level: "step", text: "==> Creating filesystems" },
  { step: 3, level: "info", text: "mkfs.btrfs -L kubaos /dev/mapper/kubaos · subvolumes @ @home @nix @snapshots" },
  { step: 4, level: "step", text: "==> Copying the system" },
  { step: 4, level: "info", text: "unsquashfs -f -d /mnt/target /run/kubaos/payload/rootfs.squashfs" },
  { step: 4, level: "info", text: "41% · 61812 of 150233 files · 4.1 of 9.4 GB" },
  { step: 5, level: "step", text: "==> Configuring" },
  { step: 5, level: "info", text: "locale en_US.UTF-8 · keyboard us · zone America/New_York" },
  { step: 6, level: "step", text: "==> Creating your account" },
  { step: 7, level: "step", text: "==> Installing the bootloader" },
  { step: 7, level: "warn", text: "Windows Boot Manager found: added to the GRUB menu" },
  { step: 8, level: "step", text: "==> Activating your home" },
  { step: 9, level: "step", text: "==> Taking the factory snapshot" },
  { step: 10, level: "step", text: "==> Finishing" },
];

/** What a failed install says: the step, the cause and the next thing to try. */
export const FAILURE = {
  step: 4,
  after: 112,
  error: "No space left on device while copying /nix/store.",
  hint: "The free space was smaller than it looked. Erase the disk instead, or free more room in Windows and try again.",
  log: [
    { level: "info" as const, text: "unsquashfs: 87% · 130620 of 150233 files" },
    { level: "error" as const, text: "write_file: failed to write /mnt/target/nix/store/9x2k…-firefox-133.0: No space left on device" },
    { level: "error" as const, text: "!!! No space left on device while copying /nix/store." },
    { level: "info" as const, text: "hint: The free space was smaller than it looked." },
  ],
};

/** Things worth knowing on the first day, shown one at a time while the install runs. */
export const TIPS: readonly { readonly keys: string; readonly does: string }[] = [
  { keys: "Super+Space", does: "opens the Command menu: apps, settings, anything." },
  { keys: "Super+Return", does: "opens a terminal." },
  { keys: "Super+Q", does: "closes a window. Nothing ever closes on its own." },
  { keys: "Super+L", does: "locks the screen. Your password unlocks it." },
];

/** The welcome's big line, the lede under it, and the done screen's headline. */
export const HERO: Words = { short: "Let's fix this shit.", long: "Let's fix this shit, once and for all, properly this time." };
export const LEDE: Words = {
  short: "Whatever's on here, it's had its chance.",
  long: "Whatever's on here, it's had its chance. Kuba's own Linux: a fast, keyboard-\u2060driven desktop for writing Rust all day without the OS getting in the way.",
};
export const DONE_HEADLINE: Words = { short: "Fixed. You're welcome.", long: "Fixed, finally. You're very welcome, by the way." };
