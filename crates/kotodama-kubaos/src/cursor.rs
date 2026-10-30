//! The cursor in the menu: one Item for the pointer, its style and size in the subtitle, and an
//! Action each style and size, the next style first. Each runs `kubaos cursor`, which applies it
//! at once and keeps it in `~/.config/kubaos/desktop/cursor/cursor.toml`.
//!
//! # Owns
//!
//! - The Item and what each of its Actions runs.
//!
//! # Not here
//!
//! - Reading the settings: [`crate::states`]. Applying them: `kubaos cursor` (`kubaos-cli`), with
//!   `kubaos_cursor`.

use std::collections::BTreeMap;
use std::time::Duration;

use kotodama_extension::Chain;
use kotodama_protocol::Action;
use kotodama_protocol::Argument;
use kotodama_protocol::Item;
use kotodama_protocol::ItemKind;
use kotodama_protocol::NamespacedKind;
use kubaos_cursor::settings::Style;

use crate::run::Runner;
use crate::states::CursorSettings;
use crate::toggles::Ran;

/// The Item kind of the cursor.
pub const KIND: &str = "kubaos.cursor";
/// The Item's id, which `hide` names it by.
pub const ID: &str = "cursor";
/// The Action that sets a size in pixels, its Argument.
pub const SIZE: &str = "size";

/// The sizes offered as Actions of their own, in pixels: the default, larger, and large.
const SIZES: [u32; 3] = [24, 32, 48];
/// The Argument of [`SIZE`].
const PIXELS: &str = "pixels";

/// The Item, as `cursor` has the settings; `None` while they can't be read.
pub fn item(cursor: Option<CursorSettings>) -> Item {
    let current = cursor.map(|cursor| cursor.style);
    let next = current.map(|style| Style::ALL[(index(style) + 1) % Style::ALL.len()]);
    let styles = next
        .into_iter()
        .chain(Style::ALL.into_iter().filter(|&style| Some(style) != next))
        .map(style_action);
    let sizes = SIZES.into_iter().map(size_action);
    let set = Action::new(SIZE, "Size…")
        .arguments(vec![Argument::new(PIXELS).placeholder("32")])
        .keywords(["set", "pixels"]);
    Item {
        id: ID.to_owned(),
        kind: ItemKind::Namespaced(kind()),
        identity: None,
        title: "Cursor".to_owned(),
        subtitle: Some(cursor.map_or_else(
            || "The pointer in the wallpaper's colours: its style and size".to_owned(),
            subtitle,
        )),
        icon: Some("input-mouse".to_owned()),
        keyword: None,
        keywords: [
            "cursor style",
            "cursor size",
            "pointer",
            "pointer size",
            "mouse",
            "small cursor",
            "big cursor",
            "bigger cursor",
            "large cursor",
            "theme",
        ]
        .map(str::to_owned)
        .to_vec(),
        hidden: false,
        app_only: false,
        weak_keywords: Vec::new(),
        yields: false,
        actions: styles.chain(sizes).chain([set]).collect(),
    }
}

/// Every id of an Item this module sends, which no action of the registry may take.
pub fn ids() -> [&'static str; 1] {
    [ID]
}

/// Runs `action` with `arguments`: sets the style or size it names.
///
/// # Errors
///
/// When there is no such Action, the size isn't a number, or `kubaos` can't start; the message is
/// what the user reads.
pub fn run(action: &str, arguments: &BTreeMap<String, String>, runner: &impl Runner) -> Result<Ran, String> {
    let (setting, value) = if let Some(style) = Style::ALL.into_iter().find(|style| style.as_str() == action) {
        ("style", style.as_str().to_owned())
    } else if let Some(size) = SIZES.into_iter().find(|size| size_id(*size) == action) {
        ("size", size.to_string())
    } else if action == SIZE {
        let pixels = arguments.get(PIXELS).map_or("", |pixels| pixels.trim());
        let size: u32 = pixels
            .parse()
            .map_err(|_| format!("A cursor size is a number of pixels, not {pixels:?}"))?;
        ("size", size.to_string())
    } else {
        return Err(format!("The cursor has no Action `{action}`"));
    };
    let argv = ["kubaos", "cursor", setting, &value].map(str::to_owned);
    runner
        .launch(&argv, Duration::ZERO)
        .map_err(|err| Chain(&err).to_string())?;
    Ok(Ran::Close)
}

fn kind() -> NamespacedKind {
    NamespacedKind::new(KIND).expect("the cursor kind is valid")
}

fn index(style: Style) -> usize {
    Style::ALL
        .iter()
        .position(|&each| each == style)
        .expect("every style is in Style::ALL")
}

fn style_action(style: Style) -> Action {
    let (title, keywords): (&str, &[&str]) = match style {
        Style::Dark => ("Dark, with a light outline", &["dark"]),
        Style::Accent => ("In the accent colour", &["accent", "colour", "color"]),
        Style::Classic => ("Classic black and white", &["classic", "bibata"]),
    };
    Action::new(style.as_str(), title).keywords(keywords.iter().copied())
}

fn size_id(size: u32) -> String {
    format!("size-{size}")
}

fn size_action(size: u32) -> Action {
    Action::new(size_id(size), format!("{size} px")).keywords([size.to_string()])
}

fn subtitle(cursor: CursorSettings) -> String {
    let size = cursor.size;
    match cursor.style {
        Style::Dark => format!("Dark, {size} px: a light outline, readable on any wallpaper"),
        Style::Accent => format!("Accent, {size} px: the pointer in the wallpaper's accent colour"),
        Style::Classic => format!("Classic, {size} px: Bibata black and white, whatever the wallpaper"),
    }
}

#[cfg(test)]
mod tests;
