import QtQuick
import QtTest
import KubaOS.Components as K

// An icon tile with a photo: the symbol shows until the photo has loaded, and the photo's
// rounded mask is there from the start, so the photo draws in the same frame it loads. An app that
// names no symbol shows its initial in the symbol's place.
TestCase {
    id: suite

    name: "IconTile"
    when: windowShown
    // design-allow(size): a test's fixture, not a design value.
    width: 200
    visible: true
    // design-allow(size): a test's fixture, not a design value.
    height: 200

    // A picture for each test: Qt keeps one loaded, and a second load of it is ready at once.
    readonly property url photo: Qt.resolvedUrl("../../branding/png/kubaos-icon-1024.png")
    readonly property url other: Qt.resolvedUrl("../../branding/png/kubaos-horizontal.png")

    K.IconTile {
        id: tile

        size: "Sm"
    }

    K.IconTile {
        id: app

        icon: ""
        name: "Blender"
    }

    function cleanup() {
        tile.photo = "";
        app.name = "Blender";
        app.icon = "";
        app.source = "";
    }

    // Regression: #252.
    function test_a_photo_s_mask_is_there_before_the_photo_has_loaded() {
        tile.photo = suite.photo;
        const cover = findChild(tile, "cover");

        verify(cover.status !== Image.Ready, "still loading, as it loads asynchronously");
        verify(findChild(tile, "corners").layer.enabled);
        tryCompare(cover, "status", Image.Ready);
    }

    function test_the_symbol_shows_until_the_photo_has_loaded() {
        tile.photo = suite.other;

        verify(!tile.pictured);
        tryVerify(() => tile.pictured, 5000, "the photo takes the tile once it has loaded");
    }

    function test_an_app_with_no_symbol_shows_the_first_letter_of_its_name_upper_cased() {
        app.name = "  k9s";

        compare(app.letter, "K");
        compare(app.probe.variant, "Md initials");
    }

    function test_an_initial_is_one_character_in_whatever_script_the_name_is_in() {
        const cases = [["日本語入力", "日"], ["éclair", "É"], ["e\u0301clair", "E\u0301"], ["𝒜lpha", "𝒜"], ["ßeta", "SS"], ["", ""]];
        for (const [name, initial] of cases) {
            app.name = name;

            compare(app.letter, initial, name);
        }
    }

    function test_a_symbol_takes_the_initial_s_place() {
        app.icon = "terminal";

        compare(app.letter, "");
        compare(app.probe.variant, "Md");
    }

    function test_the_initial_shows_until_the_app_s_icon_has_loaded() {
        app.source = suite.photo;

        verify(!app.pictured);
        tryVerify(() => app.pictured, 5000, "the icon takes the tile once it has loaded");
        compare(app.letter, "B", "the initial stays, hidden, for when the icon goes");
    }

    function test_an_initial_sits_on_whole_device_pixels_inside_its_tile() {
        const letter = findChild(app, "initial");

        verify(letter !== null);
        verify(letter.x > 0 && letter.x + letter.width < app.width, "centred across");
        compare(letter.height, K.Tokens.type.subheading.line);
        compare(Math.round(letter.x * app.dpr), letter.x * app.dpr);
        compare(Math.round(letter.y * app.dpr), letter.y * app.dpr);
    }
}
