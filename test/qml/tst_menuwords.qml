import QtQuick
import QtTest
import "../../config/quickshell/modules/menu/words.js" as Words

// What a Result row's tile draws from an Item's icon: a picture, the symbol it names, its
// Extension's symbol, or an app's initial.
TestCase {
    name: "MenuWords"

    function test_a_desktop_app_names_no_symbol_and_shows_its_title_s_initial() {
        compare(Words.tileGlyph("firefox", "apps"), "");
        compare(Words.rowName("firefox", "Firefox"), "Firefox");
    }

    function test_a_catalog_app_with_only_a_name_shows_its_initial_and_no_picture() {
        const icon = "initial:Claude Code";

        compare(Words.tileGlyph(icon, "kubaos"), "");
        compare(Words.rowName(icon, "Install Claude Code"), "Claude Code");
        compare(Words.iconSource(icon, name => `theme:${name}`), "");
    }

    function test_a_symbol_the_icon_names_is_drawn_and_an_empty_one_falls_back_to_the_extension_s() {
        compare(Words.tileGlyph("symbol:speed", "kubaos"), "speed");
        compare(Words.tileGlyph("symbol:", "kubaos"), Words.glyph("kubaos"));
        compare(Words.itemGlyph("symbol:", "calculator"), "calculate");
    }

    function test_an_item_of_another_extension_keeps_its_extension_s_symbol() {
        compare(Words.tileGlyph("", "calculator"), "calculate");
        compare(Words.itemGlyph("", "apps"), "desktop_windows", "the detail pane's big symbol");
    }
}
