import QtQuick
import QtTest
import "../../config/quickshell/modules/menu/words.js" as Words

// What a Result row's tile and the detail pane draw from an Item: its picture, its Extension's
// symbol, or a desktop app's initial.
TestCase {
    name: "MenuWords"

    function test_a_desktop_app_names_no_symbol_so_its_tile_shows_its_initial() {
        compare(Words.tileGlyph("apps"), "");
    }

    function test_an_item_of_another_extension_keeps_its_extension_s_symbol() {
        compare(Words.tileGlyph("calculator"), "calculate");
        compare(Words.tileGlyph("kubaos"), Words.glyph("kubaos"));
    }

    function test_a_picture_is_a_path_or_a_theme_name_and_no_icon_is_none() {
        compare(Words.iconSource("/apps/gamescope/gamescope.svg", name => `theme:${name}`), "file:///apps/gamescope/gamescope.svg");
        compare(Words.iconSource("firefox", name => `theme:${name}`), "theme:firefox");
        compare(Words.iconSource("", name => `theme:${name}`), "");
    }
}
