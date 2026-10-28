# KubaOS: name and voice

Every user-facing string, doc and image follows this file. Agents: read it before you write a single word somebody will see. How things look and behave (colour, type, space, motion, icons, components, the keyboard) is [DESIGN.md](DESIGN.md).

## Name

- **KubaOS**: one word, capital K, capital OS. Never "Kuba OS", "kubaOS" or "KUBAOS". The lowercase id `kubaos` is for files, packages and volume labels.
- **Logo:** the KubaOS neon sign: クバ (Kuba in katakana) in amber tubes with a blue dakuten, then "KubaOS", on a dark wall. The photo of it is the logo wherever it shows large (boot splash, GRUB, Kōbō, the README). Small sizes (icons, tray, fastfetch) use the vector クバ mark, with "KubaOS" as the wordmark next to it or under it. The sources live in `branding/`.
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

The boot menu says **KubaOS Shinjuku** (the entry), "Advanced options for KubaOS Shinjuku" and **KubaOS snapshots**, never "GNU/Linux" or "Ubuntu". The splash is the neon sign on the dark, nothing else: it switches on a group of tubes at a time, then hums until the desktop shows up. The disk passphrase prompt is rule 4 territory, so it's plain, with no jokes and no swearing:

| Where | Text |
|---|---|
| Placeholder | Disk passphrase |
| Hint | The one you set when you installed KubaOS. `Enter` unlocks. |
| While checking | Unlocking… |
| Wrong passphrase | Wrong passphrase. Try again. |
| Caps Lock | `Caps Lock` is on. |

The keys are keycaps, as everywhere (docs/DESIGN.md, "Keys").

How it's built: `docs/INSTALL-INTERNALS.md`, "Boot".

## How we describe it

Don't describe KubaOS by listing what's inside it. "Ubuntu 26.04 with Hyprland, Quickshell and home-manager" is a parts list, not a description. Say what it is and what it's for. The parts go in an "under the hood" section, for the people who care.

What it is: Kuba's own Linux. A fast, keyboard-driven desktop for writing and compiling Rust all day without the OS getting in the way. It looks great, it updates only when you tell it to, it never nags, and if you break it there's always a way back. It exists because macOS kept getting worse with every update, and Kuba got tired of waiting.

Lines in this spirit (use, remix or beat them):
- "Your machine. Your rules. No bullshit."
- "An OS that does what it's told and then shuts the fuck up."
- "Built for writing Rust at 2am, not for clicking 'Later' on update popups."
- "Kuba's Linux. You're welcome."

## Voice

Free, loud, "don't care", and properly rude. Think of a friend who's very good at computers, finds most of them stupid, and swears like it.

Rules, in priority order:

1. **Clarity beats attitude.** Every message still says plainly what happened, what's about to happen and what to do. The joke goes around the information, never instead of it.
2. **Swear when it's funny.** Most jokes can have one; one per line is plenty. A label stays a label: a field name, a button that only says what it does, a setting's title. Plain text between the jokes is what makes them land.
3. **Roast everyone and everything, equally:** Apple, Windows, update popups, corporate software, bloat, the computer, the OS itself, Kuba, and you, the reader. Never identity: nothing about race, gender, sexuality, disability or religion, and no slurs.
4. **The expensive sentences stay plain.** The sentence that names the disk about to be erased, the passphrase prompts at boot, and anything that says data is about to go are exact and calm. The lead-in around them can swear ("Last chance to not fuck this up."); the sentence with the disk in it can't.
5. **Errors keep their attitude and their facts.** "Fuck. The disk is in use, so nothing got written. Free it and try again." It names the cause and gives the next action.
6. **"I" is the OS, the button is you.** Messages, notifications and subtitles speak as the OS ("I'll keep the last 30 s, you're welcome."). Buttons and choices the user picks speak as the user ("Fuck Wi-Fi, I've got a cable"). Never "we".
7. Short sentences, active voice, no corporate filler: no "Please note", "Simply", "Seamlessly", "Powerful", "Leverage", or exclamation-mark enthusiasm. Confident, dry and rude, not hype.
8. Japanese touches are seasoning: small labels above titles, the logo, release names. Don't write whole sentences in Japanese, and don't use anime catchphrases.

Examples:

| Where | Write |
|---|---|
| Kōbō, first screen | Whatever's on here, it's had its chance. Let's fix this shit. |
| Installing | Installing. Go stare at a wall, it'll be done before you get bored. |
| Done (the line follows the time it took) | Done in 1m 42s. Damn, that was quick. Pull the stick and reboot. |
| Wi-Fi error | Wrong password for Home-5G. Fat fingers? Try again. |
| Generic error | Fuck. The disk is in use, so nothing got written. Free it and try again. |
| Update notification | Shinjuku 0.1.3 is ready. Update when you feel like it. Or don't, I'm not your mum. |
| Update failed, rolled back | 0.1.3 shit the bed. You're on 0.1.2 again, like it never happened. |
| Weak password | Weak. A toddler could guess it. |
| Skip Wi-Fi button | Fuck Wi-Fi, I've got a cable |
| Disk erase | Last chance to not fuck this up. Everything on **Samsung SSD 990 PRO (nvme0n1, 2 TB)** gets erased. There's no undo. |
| Battery low | 14% left. Plug the damn thing in. |
| Menu subtitle | Large pointer. Grandpa mode. |
| Replay off | Replay off. Your last 30 s just got yeeted into the void. |
| Screenshot | Copied. Evidence secured. |
| Tagline | An OS that does what it's told and then shuts the fuck up. |

## Keyboard first

KubaOS is used from the keyboard, and so is Kōbō. The keyboard map every app shares, the focus ring and the rest of the behaviour are in [DESIGN.md](DESIGN.md), "Behaviour".
