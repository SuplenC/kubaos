# KubaOS: name, look, voice

Every user-facing string, doc and image follows this file. Agents: read it before you write a single word somebody will see.

## Name

- **KubaOS**: one word, capital K, capital OS. Never "Kuba OS", "kubaOS" or "KUBAOS". The lowercase id `kubaos` is for files, packages and volume labels.
- **Logo:** クバ (Kuba in katakana). It's the mark; "KubaOS" is the wordmark next to it or under it. The sources live in `branding/`.
- **Releases are named after Tokyo neighbourhoods,** in this order: **Shinjuku** (0.1), Akihabara, Shibuya, Kabukichō, Harajuku, Ikebukuro, Shimokitazawa, Koenji, Nakano, Ginza. Written as "KubaOS Shinjuku". The id is `shinjuku`.
- **Kōbō** (工房, "workshop"): the app that puts KubaOS on a machine and sets it up afterwards. Written with its macrons wherever a person reads it; the id is `kobo` (the binary, the app id, the desktop entry). Never "the installer".
- Internal tool names stay: the `dot-*` scripts. Nobody sees them unless they go looking, and anyone who does can read.

## `/etc/os-release`

Apt, PPAs, the Docker repo, the TUXEDO repo and half the internet's install scripts read `ID` and `VERSION_CODENAME`. Those stay Ubuntu's. Only the names a person reads change:

```
NAME="KubaOS"
PRETTY_NAME="KubaOS Shinjuku"
VERSION="0.1 (Shinjuku)"
VERSION_ID="26.04"            # Ubuntu's, tools compare against it
VERSION_CODENAME=resolute     # Ubuntu's, apt needs it
ID=ubuntu
ID_LIKE=debian
VARIANT="Shinjuku"
VARIANT_ID=shinjuku
HOME_URL=...                  # the repo
```

The file is `dpkg-divert`ed so a `base-files` upgrade can't silently put Ubuntu's back. `fastfetch` shows the クバ logo.

## Boot

The boot menu says **KubaOS Shinjuku** (the entry), "Advanced options for KubaOS Shinjuku" and **KubaOS snapshots**, never "GNU/Linux" or "Ubuntu". The splash is the クバ mark on the dark, nothing else. The disk passphrase prompt is rule 4 territory, so it's plain, with no jokes and no swearing:

| Where | Text |
|---|---|
| Label | Disk passphrase |
| Hint | The one you set when you installed KubaOS. Enter unlocks. |
| While checking | Unlocking... |
| Wrong passphrase | Wrong passphrase. Try again. |
| Caps Lock | Caps Lock is on. |

How it's built: `docs/INSTALL-INTERNALS.md`, "Boot".

## How we describe it

Don't describe KubaOS by listing what's inside it. "Ubuntu 26.04 with Hyprland, Quickshell and home-manager" is a parts list, not a description. Say what it is and what it's for. The parts go in an "under the hood" section, for the people who care.

What it is: Kuba's own Linux. A fast, keyboard-driven desktop for writing and compiling Rust all day without the OS getting in the way. It looks great, it updates only when you tell it to, it never nags, and if you break it there's always a way back. It exists because macOS kept getting worse with every update, and Kuba got tired of waiting.

Lines in this spirit (use, remix or beat them):
- "Your machine. Your rules. No bullshit."
- "An OS that does what it's told and then shuts up."
- "Built for writing Rust at 2am, not for clicking 'Later' on update popups."
- "Kuba's Linux. You're welcome."

## Voice

Free, relaxed, a bit "don't care", slightly offensive, with a **sprinkle** of profanity. Think of a friend who's very good at computers and finds most of them stupid.

Rules, in priority order:

1. **Clarity beats attitude.** Every message still says plainly what happened, what's about to happen and what to do. The joke goes around the information, never instead of it.
2. **Sprinkle, don't pour.** At most one swear per screen, notification or doc section. Most lines have none. Swearing every line is boring, and boring is the one thing we're not allowed to be.
3. **Punch at the right targets:** Apple, Windows, update popups, corporate software, bloat, the computer itself, and occasionally Kuba. Never at the reader's identity, never slurs, nothing about race, gender, sexuality, disability or religion.
4. **No profanity where a mistake is expensive.** The "erase this disk" confirmation, passphrase prompts, and anything about data loss are dead serious and crystal clear. A light line around them is fine, but not in the sentence that names the disk.
5. **Errors keep their attitude and their facts.** "Well, shit. Couldn't open the encrypted disk: wrong passphrase. Try again, slower this time." It names the cause and gives the next action.
6. Short sentences, active voice, no corporate filler: no "Please note", "Simply", "Seamlessly", "Powerful", "Leverage", or exclamation-mark enthusiasm. Confident, dry humour, not hype.
7. Japanese touches are seasoning: section labels, the logo, release names. Don't write whole sentences in Japanese, and don't use anime catchphrases.

Examples:

| Instead of | Write |
|---|---|
| Welcome to the installer! | KubaOS Shinjuku. Let's get rid of whatever's on this thing. |
| Please select your keyboard layout. | Keyboard. Pick the one your fingers actually know. |
| Installation in progress. Please wait. | Installing. Go get a coffee. Actually, don't bother, it's faster than that. |
| Installation complete! | Done in 1m 42s. Reboot and enjoy a computer that minds its own business. |
| An error occurred. | Well, shit. `<what failed>`: `<why>`. `<what to do>`. |
| This will erase all data on /dev/nvme0n1. | Everything on **Samsung SSD 990 PRO (nvme0n1, 2 TB)** gets erased. Windows too. There's no undo. |

## Keyboard first

KubaOS is used from the keyboard, and so is Kōbō. Everything you can do with the mouse, you can do with keys: a visible focus ring, Tab and Shift+Tab, arrows in lists and choices, Space to toggle, Enter to go on, Esc to go back. Shortcuts are on screen where they're needed. The mouse works, but you never need it.
