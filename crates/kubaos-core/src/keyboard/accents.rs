//! The Mac-style accents on Left Alt: Left Alt + e, `, u, i or n, then a letter, types it with
//! an acute, grave, diaeresis, circumflex or tilde (é à ö ô ñ), and Space after the chord types
//! the mark alone (´ ` ¨ ˆ ˜), as Option does on a Mac.
//!
//! keyd catches the chord and, for the letter that follows, sends a dead key and then the same
//! letter key. The dead keys sit on keycodes no xkeyboard-config layout uses, and the
//! `kubaos:mac-accents` XKB option gives them their symbols, so the app composes the accent with
//! whatever letter the active layout types. keyd's own unicode output can't do this: it types a
//! compose sequence with the US layout's keys, which breaks on AZERTY, QWERTZ or Cyrillic.
//!
//! # Owns
//!
//! - The table of marks, chords and spare keycodes, and the two files made from it:
//!   `system/files/keyd/kubaos.conf` and `system/files/xkb/symbols/kubaos`. A test keeps them
//!   equal to the table.
//!
//! # Not here
//!
//! - Installing them: `install_mac_accents` in `system/lib.sh`. The option in the keymap:
//!   `config/hypr/input.lua`. Turning them on and off: [`super::settings`] and
//!   `kubaos keyboard accents`.

use std::fmt::Write;

/// A keycode, by its keyd name and its XKB name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    /// keyd's name, as `keyd list-keys` prints it.
    pub keyd: &'static str,
    /// The keycode in XKB's evdev keycodes, `I183` for `<I183>`.
    pub xkb: &'static str,
}

/// The key keyd sends for Space after a chord, and whether it holds Shift for the second level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spacing {
    /// keyd's name of the key.
    pub keyd: &'static str,
    /// Shift picks the second of the key's two symbols.
    pub shifted: bool,
}

/// One mark, the chord that starts it and the keys keyd sends for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Accent {
    /// The mark's name, which is also its keyd layer's after `mac-`.
    pub name: &'static str,
    /// The key held with Left Alt, by keyd's name.
    pub chord: &'static str,
    /// The spare key that types the dead key.
    pub dead: Key,
    /// The XKB keysym of the dead key.
    pub dead_keysym: &'static str,
    /// The key that types the mark alone.
    pub spacing: Spacing,
    /// The XKB keysym of the mark alone, as a Mac types it after Space.
    pub spacing_keysym: &'static str,
}

/// The spare keys the marks alone sit on, two per key at most. Each takes a key apart from the
/// dead keys: with Shift on a dead key, keyd would send Shift with the letter too and break the
/// lowercase.
const SPACING_KEYS: [Key; 3] = [
    Key { keyd: "setup", xkb: "I149" },
    Key { keyd: "deletefile", xkb: "I154" },
    Key { keyd: "closecd", xkb: "I168" },
];

/// The five marks, in the order a Mac lists them. The spare keycodes have no symbol in any
/// xkeyboard-config layout; `macro` (I120), `unknown` (I248) and I219 do, so they are not here.
pub const ACCENTS: [Accent; 5] = [
    Accent {
        name: "acute",
        chord: "e",
        dead: Key { keyd: "move", xkb: "I183" },
        dead_keysym: "dead_acute",
        spacing: Spacing { keyd: "setup", shifted: false },
        spacing_keysym: "acute",
    },
    Accent {
        name: "grave",
        chord: "`",
        dead: Key { keyd: "edit", xkb: "I184" },
        dead_keysym: "dead_grave",
        spacing: Spacing { keyd: "setup", shifted: true },
        spacing_keysym: "grave",
    },
    Accent {
        name: "diaeresis",
        chord: "u",
        dead: Key { keyd: "bassboost", xkb: "I217" },
        dead_keysym: "dead_diaeresis",
        spacing: Spacing { keyd: "deletefile", shifted: false },
        spacing_keysym: "diaeresis",
    },
    Accent {
        name: "circumflex",
        chord: "i",
        dead: Key { keyd: "question", xkb: "I222" },
        dead_keysym: "dead_circumflex",
        spacing: Spacing { keyd: "deletefile", shifted: true },
        spacing_keysym: "U02C6",
    },
    Accent {
        name: "tilde",
        chord: "n",
        dead: Key { keyd: "iso", xkb: "I178" },
        dead_keysym: "dead_tilde",
        spacing: Spacing { keyd: "closecd", shifted: false },
        spacing_keysym: "U02DC",
    },
];

/// The top of the keyd config: what it does and why it sends spare keys.
const KEYD_HEADER: &str = "\
# Mac-style accents on Left Alt, installed as /etc/keyd/kubaos.conf by
# system/50-hyprland.sh. Left Alt + e, `, u, i or n, then a letter, types the
# letter with an acute, grave, diaeresis, circumflex or tilde, as Option does
# on a Mac: Left Alt + e, then e, is é. Space after the chord types the mark
# alone.
#
# keyd doesn't type the accented letter itself. Its unicode output goes
# through a compose sequence typed with the US layout's letter and digit
# keys, which breaks on AZERTY, QWERTZ or Cyrillic. For the letter after a
# chord it sends a dead key on a keycode no layout uses, then the letter's
# own key. The `kubaos:mac-accents` XKB option (system/files/xkb/symbols/kubaos)
# gives those keycodes their symbols in every layout, so the app composes
# the accent with whatever letter the layout types, and Shift gives the
# capital.
#
# Generated from crates/kubaos-core/src/keyboard/accents.rs; a test fails
# when the two differ. Turning the accents off stops keyd
# (`kubaos keyboard accents off`).
# https://github.com/rvaiya/keyd/blob/master/docs/keyd.scdoc

[ids]
*

# Right Alt keeps keyd's own `altgr` layer, so AltGr layouts still reach
# their third level.
[main]
leftalt = layer(macalt)

# A layer with `:A` is Alt for every key it doesn't bind, so ALT + Tab, Alt
# in terminals, tmux, nvim and browsers stay Alt. A chord latches its mark's
# layer for the next key only.
[macalt:A]
";

/// The comment above the marks' layers.
const KEYD_LAYERS: &str = "
# The key after a chord. keyd lets go of Alt while it sends a bound key, so
# the dead key and the letter reach the app without Alt; a held Shift stays,
# for the capital. Any other key ends the latch and types as it would.
";

/// The top of the XKB symbols.
const XKB_HEADER: &str = "\
// KubaOS's XKB symbols, installed as /etc/xkb/symbols/kubaos by
// system/50-hyprland.sh and named by the options in /etc/xkb/rules/evdev.
// Generated from crates/kubaos-core/src/keyboard/accents.rs; a test fails
// when the two differ.

// The keys keyd sends for the Mac-style accents on Left Alt
// (system/files/keyd/kubaos.conf): a dead key for each mark, and the marks
// alone for Space after a chord. No layout in xkeyboard-config puts a symbol
// on these keycodes, and a key with one group types the same in every
// layout, so they work whichever layout is active.
partial
xkb_symbols \"mac-accents\" {
";

/// The keyd config: Left Alt is Alt, but for the five chords, which latch a mark for the next
/// key.
pub fn keyd_config() -> String {
    let mut out = String::from(KEYD_HEADER);
    for accent in &ACCENTS {
        let _ = writeln!(out, "{} = oneshot(mac-{})", accent.chord, accent.name);
    }
    out.push_str(KEYD_LAYERS);
    for accent in &ACCENTS {
        let shift = if accent.spacing.shifted { "S-" } else { "" };
        let _ = write!(out, "\n[mac-{}]\nspace = {shift}{}\n", accent.name, accent.spacing.keyd);
        for letter in 'a'..='z' {
            let _ = writeln!(out, "{letter} = macro({} {letter})", accent.dead.keyd);
        }
    }
    out
}

/// The `kubaos(mac-accents)` XKB symbols: the dead keys, then the marks alone, two to a key.
pub fn xkb_symbols() -> String {
    let mut out = String::from(XKB_HEADER);
    for accent in &ACCENTS {
        let _ = writeln!(out, "    // Left Alt + {}: keyd's `{}`.", accent.chord, accent.dead.keyd);
        let _ = writeln!(out, "    key <{}> {{ [ {} ] }};", accent.dead.xkb, accent.dead_keysym);
    }
    for key in &SPACING_KEYS {
        let levels = |shifted: bool| {
            ACCENTS
                .iter()
                .find(|accent| accent.spacing.keyd == key.keyd && accent.spacing.shifted == shifted)
                .map(|accent| accent.spacing_keysym)
        };
        let symbols = match (levels(false), levels(true)) {
            (Some(plain), Some(shifted)) => format!("{plain}, {shifted}"),
            (Some(plain), None) => plain.to_owned(),
            _ => continue,
        };
        let _ = writeln!(out, "    // Space after a chord: keyd's `{}`.", key.keyd);
        let _ = writeln!(out, "    key <{}> {{ [ {symbols} ] }};", key.xkb);
    }
    out.push_str("};\n");
    out
}

#[cfg(test)]
mod tests;
