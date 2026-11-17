//! The symbols an app's `symbol` can name: the Tabler icons of KubaOS Symbols, the list in
//! `design/tokens.toml`'s `[symbols]` that `kubaos-ui` and the shell draw from, read here so
//! the catalog can name only a symbol the shell has a drawing of.

use std::collections::BTreeMap;
use std::sync::LazyLock;

/// The design tokens, embedded as the build saw them.
const TOKENS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../design/tokens.toml"));

/// Each Tabler icon of `[symbols]`, by its Tabler name, to the KubaOS Symbol that draws it.
static BY_TABLER: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    let tokens: toml::Table = toml::from_str(TOKENS).expect("design/tokens.toml is TOML");
    let symbols = tokens
        .get("symbols")
        .and_then(toml::Value::as_table)
        .expect("design/tokens.toml has [symbols]");
    symbols
        .iter()
        .filter_map(|(name, symbol)| {
            let tabler = symbol.get("tabler")?.as_str()?;
            Some((tabler.to_owned(), name.clone()))
        })
        .collect()
});

/// The KubaOS Symbol, as `kubaos-ui` and the shell name it, that draws the Tabler icon `tabler`;
/// none for an icon KubaOS Symbols don't ship.
pub fn by_tabler(tabler: &str) -> Option<&'static str> {
    BY_TABLER.get(tabler).map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::by_tabler;

    #[test]
    fn a_tabler_name_of_kubaos_symbols_names_its_symbol_and_any_other_none() {
        assert_eq!(by_tabler("aspect-ratio"), Some("aspect_ratio"));
        assert_eq!(by_tabler("gauge"), Some("speed"));
        assert_eq!(by_tabler("device-gamepad-2"), Some("sports_esports"));
        assert_eq!(by_tabler("user-circle"), Some("account_circle"));
        assert_eq!(by_tabler("aspect_ratio"), None, "a KubaOS name isn't a Tabler one");
        assert_eq!(
            by_tabler("rocket-off"),
            None,
            "Tabler has it, KubaOS Symbols don't ship it"
        );
    }
}
